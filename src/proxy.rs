use crate::config::{Config, Route, Upstream};
use crate::state::{RequestLog, SharedState};
use anyhow::Result;
use bytes::Bytes;
use chrono::Utc;
use http_body_util::{BodyExt, Full, combinators::BoxBody};
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpListener;
use tracing::{error, info, warn};

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type BoxedBody = BoxBody<Bytes, GenericError>;

pub struct ProxyServer {
    config: Config,
    state: Arc<SharedState>,
    route_matcher: Arc<RouteMatcher>,
}

struct RouteMatcher {
    routes: Vec<Route>,
}

impl RouteMatcher {
    fn new(routes: Vec<Route>) -> Self {
        Self { routes }
    }

    fn find_route(&self, host: &str, path: &str) -> Option<&Route> {
        for route in &self.routes {
            if self.matches_host(&route.hosts, host) && path.starts_with(&route.path_prefix) {
                return Some(route);
            }
        }
        None
    }

    fn matches_host(&self, patterns: &[String], host: &str) -> bool {
        for pattern in patterns {
            if pattern.starts_with("*.") {
                let suffix = &pattern[2..];
                if host.ends_with(suffix) || (suffix.len() > 1 && host == &suffix[1..]) {
                    return true;
                }
            } else if pattern == host {
                return true;
            }
        }
        false
    }
}

impl ProxyServer {
    pub fn new(config: Config, state: Arc<SharedState>) -> Self {
        let route_matcher = Arc::new(RouteMatcher::new(config.routes.clone()));
        Self {
            config,
            state,
            route_matcher,
        }
    }

    pub async fn run(self) -> Result<()> {
        let addr: SocketAddr = self.config.listen.parse()?;
        let listener = TcpListener::bind(addr).await?;
        info!("Proxy server listening on {}", addr);

        let server = Arc::new(self);

        loop {
            let (stream, remote_addr) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let server_clone = Arc::clone(&server);

            tokio::task::spawn(async move {
                let service = service_fn(move |req| {
                    let server = Arc::clone(&server_clone);
                    async move { server.handle_request(req, remote_addr).await }
                });

                if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                    error!("Error serving connection: {}", err);
                }
            });
        }
    }

    async fn handle_request(
        &self,
        req: Request<Incoming>,
        _remote_addr: SocketAddr,
    ) -> Result<Response<BoxedBody>, std::convert::Infallible> {
        let start = Instant::now();
        self.state.increment_total_requests();
        self.state.increment_active_requests();

        let host = req
            .headers()
            .get("host")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown")
            .to_string();
        let path = req.uri().path().to_string();
        let method = req.method().clone();

        info!("Received request: {} {} Host: {}", method, path, host);

        let result = match self.route_matcher.find_route(&host, &path) {
            Some(route) => {
                let upstream = self.select_upstream(&route.upstreams);
                match upstream {
                    Some(upstream_url) => {
                        let response = self.proxy_request(req, route, &upstream_url).await;
                        let status = response.status().as_u16();

                        self.log_request(
                            method,
                            path,
                            host,
                            status,
                            start.elapsed().as_millis() as u64,
                            upstream_url,
                        );

                        response
                    }
                    None => {
                        warn!("No healthy upstream available for route: {}", route.name);
                        self.state.increment_errors();
                        self.log_request(
                            method,
                            path,
                            host,
                            503,
                            start.elapsed().as_millis() as u64,
                            "none".to_string(),
                        );
                        self.error_response(
                            StatusCode::SERVICE_UNAVAILABLE,
                            "No upstream available",
                        )
                    }
                }
            }
            None => {
                warn!("No route found for: {} {}", host, path);
                self.state.increment_errors();
                self.log_request(
                    method,
                    path,
                    host,
                    404,
                    start.elapsed().as_millis() as u64,
                    "none".to_string(),
                );
                self.error_response(StatusCode::NOT_FOUND, "No route configured")
            }
        };

        self.state.decrement_active_requests();
        Ok(result)
    }

    fn select_upstream(&self, upstreams: &[Upstream]) -> Option<String> {
        // Simple round-robin selection of first available upstream
        // In production, this would implement weighted selection and health checking
        upstreams.first().map(|u| u.url.clone())
    }

    async fn proxy_request(
        &self,
        mut req: Request<Incoming>,
        route: &Route,
        upstream_url: &str,
    ) -> Response<BoxedBody> {
        // Rewrite path if needed
        let path = req.uri().path();
        let new_path = if route.strip_prefix {
            let stripped = path.strip_prefix(&route.path_prefix).unwrap_or(path);
            if let Some(rewrite) = &route.rewrite_prefix {
                format!("{}{}", rewrite, stripped)
            } else {
                stripped.to_string()
            }
        } else {
            path.to_string()
        };

        // Build new URI
        let query = req
            .uri()
            .query()
            .map(|q| format!("?{}", q))
            .unwrap_or_default();
        let new_uri = format!("{}{}{}", upstream_url, new_path, query);

        info!("Proxying to: {}", new_uri);

        // Create a new request to the upstream
        let client =
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                .build_http();

        match new_uri.parse() {
            Ok(uri) => {
                *req.uri_mut() = uri;

                match client.request(req).await {
                    Ok(response) => {
                        let (parts, body) = response.into_parts();
                        let boxed_body = body.map_err(|e| Box::new(e) as GenericError).boxed();
                        Response::from_parts(parts, boxed_body)
                    }
                    Err(e) => {
                        error!("Error proxying request: {}", e);
                        self.state.increment_errors();
                        self.error_response(StatusCode::BAD_GATEWAY, "Upstream error")
                    }
                }
            }
            Err(e) => {
                error!("Invalid URI: {}", e);
                self.state.increment_errors();
                self.error_response(StatusCode::BAD_GATEWAY, "Invalid upstream URI")
            }
        }
    }

    fn error_response(&self, status: StatusCode, message: &str) -> Response<BoxedBody> {
        Response::builder()
            .status(status)
            .body(
                Full::new(Bytes::from(message.to_string()))
                    .map_err(|never| match never {})
                    .boxed(),
            )
            .unwrap()
    }

    fn log_request(
        &self,
        method: Method,
        path: String,
        host: String,
        status: u16,
        duration_ms: u64,
        upstream: String,
    ) {
        self.state.add_request_log(RequestLog {
            timestamp: Utc::now(),
            method: method.to_string(),
            path,
            host,
            status,
            duration_ms,
            upstream,
        });
    }
}
