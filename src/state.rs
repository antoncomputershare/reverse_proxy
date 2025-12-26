use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct RequestLog {
    pub timestamp: DateTime<Utc>,
    pub method: String,
    pub path: String,
    pub host: String,
    pub status: u16,
    pub duration_ms: u64,
    pub upstream: String,
}

#[derive(Debug, Clone)]
pub struct ProxyMetrics {
    pub total_requests: u64,
    pub active_requests: u64,
    pub total_errors: u64,
    pub upstreams_status: Vec<UpstreamStatus>,
}

#[derive(Debug, Clone)]
pub struct UpstreamStatus {
    pub url: String,
    pub healthy: bool,
    pub failures: u32,
}

pub struct SharedState {
    pub request_logs: RwLock<Vec<RequestLog>>,
    pub metrics: RwLock<ProxyMetrics>,
}

impl SharedState {
    pub fn new() -> Arc<Self> {
        Arc::new(SharedState {
            request_logs: RwLock::new(Vec::new()),
            metrics: RwLock::new(ProxyMetrics {
                total_requests: 0,
                active_requests: 0,
                total_errors: 0,
                upstreams_status: Vec::new(),
            }),
        })
    }

    pub fn add_request_log(&self, log: RequestLog) {
        let mut logs = self.request_logs.write();
        logs.push(log);
        // Keep only last 1000 requests
        if logs.len() > 1000 {
            logs.remove(0);
        }
    }

    pub fn increment_total_requests(&self) {
        let mut metrics = self.metrics.write();
        metrics.total_requests += 1;
    }

    pub fn increment_active_requests(&self) {
        let mut metrics = self.metrics.write();
        metrics.active_requests += 1;
    }

    pub fn decrement_active_requests(&self) {
        let mut metrics = self.metrics.write();
        if metrics.active_requests > 0 {
            metrics.active_requests -= 1;
        }
    }

    pub fn increment_errors(&self) {
        let mut metrics = self.metrics.write();
        metrics.total_errors += 1;
    }

    pub fn update_upstream_status(&self, statuses: Vec<UpstreamStatus>) {
        let mut metrics = self.metrics.write();
        metrics.upstreams_status = statuses;
    }

    pub fn get_request_logs(&self) -> Vec<RequestLog> {
        self.request_logs.read().clone()
    }

    pub fn get_metrics(&self) -> ProxyMetrics {
        self.metrics.read().clone()
    }
}

impl Default for SharedState {
    fn default() -> Self {
        SharedState {
            request_logs: RwLock::new(Vec::new()),
            metrics: RwLock::new(ProxyMetrics {
                total_requests: 0,
                active_requests: 0,
                total_errors: 0,
                upstreams_status: Vec::new(),
            }),
        }
    }
}
