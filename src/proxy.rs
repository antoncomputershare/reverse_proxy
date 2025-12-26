use crate::types::{ProxyState, Transaction};
use anyhow::{Context, Result};
use chrono::Utc;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Request, Response, Server, StatusCode, Uri};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tracing::{error, info};

/// Start the reverse proxy server
pub async fn start_proxy(port: u16, target: String, state: Arc<ProxyState>) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let target = Arc::new(target);

    let make_svc = make_service_fn(move |_conn| {
        let target = target.clone();
        let state = state.clone();

        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                handle_request(req, target.clone(), state.clone())
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    info!("Reverse proxy listening on http://{}", addr);

    server.await.context("Proxy server error")?;

    Ok(())
}

/// Handle a single HTTP request
async fn handle_request(
    req: Request<Body>,
    target: Arc<String>,
    state: Arc<ProxyState>,
) -> Result<Response<Body>, Infallible> {
    let start_time = Instant::now();
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let query = req
        .uri()
        .query()
        .map(|q| format!("?{}", q))
        .unwrap_or_default();

    // Update active connections
    state.update_stats(|stats| {
        stats.active_connections += 1;
    });

    // Forward request to target
    let response = match forward_request(req, &target).await {
        Ok(resp) => resp,
        Err(e) => {
            error!("Failed to forward request: {}", e);
            state.update_stats(|stats| {
                stats.failed_requests += 1;
                stats.active_connections = stats.active_connections.saturating_sub(1);
            });

            return Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(Body::from(format!("Proxy error: {}", e)))
                .unwrap());
        }
    };

    let status = response.status();
    let duration = start_time.elapsed();

    // Create transaction record
    let transaction = Transaction {
        id: state.next_transaction_id(),
        timestamp: Utc::now(),
        method: method.to_string(),
        path: format!("{}{}", path, query),
        status: status.as_u16(),
        duration_ms: duration.as_millis() as u64,
        request_size: 0,  // Could be calculated from body
        response_size: 0, // Could be calculated from body
    };

    // Update stats
    state.update_stats(|stats| {
        stats.total_requests += 1;
        if status.is_success() {
            stats.successful_requests += 1;
        } else {
            stats.failed_requests += 1;
        }
        stats.active_connections = stats.active_connections.saturating_sub(1);
    });

    state.add_transaction(transaction);

    Ok(response)
}

/// Forward the request to the target server
async fn forward_request(req: Request<Body>, target: &str) -> Result<Response<Body>> {
    // Parse target URL
    let target_uri = if target.starts_with("http://") || target.starts_with("https://") {
        target.parse::<Uri>()?
    } else {
        format!("http://{}", target).parse::<Uri>()?
    };

    // Build new URI with target host
    let path_and_query = req
        .uri()
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/");

    let new_uri = format!(
        "{}://{}{}",
        target_uri.scheme_str().unwrap_or("http"),
        target_uri
            .authority()
            .map(|a| a.as_str())
            .unwrap_or("localhost"),
        path_and_query
    )
    .parse::<Uri>()?;

    // Deconstruct the original request
    let (parts, body) = req.into_parts();

    // Create new request with the target URI
    let mut new_req = Request::builder()
        .method(parts.method)
        .uri(new_uri)
        .version(parts.version);

    // Copy all headers from original request
    for (key, value) in parts.headers.iter() {
        new_req = new_req.header(key, value);
    }

    // Build final request with original body
    let final_req = new_req.body(body).context("Failed to build forwarded request")?;

    // Create HTTP client
    let client = Client::new();

    // Forward request with body and headers
    let response = client
        .request(final_req)
        .await
        .context("Failed to send request to target")?;

    Ok(response)
}
