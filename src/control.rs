use crate::state::SharedState;
use anyhow::Result;
use bytes::Bytes;
use http_body_util::{BodyExt, Full, combinators::BoxBody};
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info};

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type BoxedBody = BoxBody<Bytes, GenericError>;

pub struct ControlServer {
    listen_addr: String,
    state: Arc<SharedState>,
}

impl ControlServer {
    pub fn new(listen_addr: String, state: Arc<SharedState>) -> Self {
        Self { listen_addr, state }
    }

    pub async fn run(self) -> Result<()> {
        let addr: SocketAddr = self.listen_addr.parse()?;
        let listener = TcpListener::bind(addr).await?;
        info!("Control server listening on {}", addr);

        let server = Arc::new(self);

        loop {
            let (stream, _remote_addr) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let server_clone = Arc::clone(&server);

            tokio::task::spawn(async move {
                let service = service_fn(move |req| {
                    let server = Arc::clone(&server_clone);
                    async move { server.handle_request(req).await }
                });

                if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                    error!("Error serving control connection: {}", err);
                }
            });
        }
    }

    async fn handle_request(
        &self,
        req: Request<Incoming>,
    ) -> Result<Response<BoxedBody>, std::convert::Infallible> {
        let path = req.uri().path();
        let method = req.method();

        info!("Control API request: {} {}", method, path);

        match (method, path) {
            (&Method::GET, "/health") => Ok(self.health_response()),
            (&Method::GET, "/metrics") => Ok(self.metrics_response()),
            _ => Ok(self.not_found_response()),
        }
    }

    fn health_response(&self) -> Response<BoxedBody> {
        let body = r#"{"status":"ok"}"#;
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(
                Full::new(Bytes::from(body))
                    .map_err(|never| match never {})
                    .boxed(),
            )
            .unwrap()
    }

    fn metrics_response(&self) -> Response<BoxedBody> {
        let metrics = self.state.get_metrics();
        let body = format!(
            r#"{{"total_requests":{},"active_requests":{},"total_errors":{},"upstreams":[]}}"#,
            metrics.total_requests, metrics.active_requests, metrics.total_errors
        );
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(
                Full::new(Bytes::from(body))
                    .map_err(|never| match never {})
                    .boxed(),
            )
            .unwrap()
    }

    fn not_found_response(&self) -> Response<BoxedBody> {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(
                Full::new(Bytes::from("Not Found"))
                    .map_err(|never| match never {})
                    .boxed(),
            )
            .unwrap()
    }
}
