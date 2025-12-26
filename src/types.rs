use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Maximum number of requests to keep in history
const MAX_HISTORY: usize = 1000;

/// Represents a single HTTP request/response transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: usize,
    pub timestamp: DateTime<Utc>,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub duration_ms: u64,
    pub request_size: usize,
    pub response_size: usize,
}

/// Proxy statistics
#[derive(Debug, Clone, Default)]
pub struct ProxyStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub active_connections: usize,
}

/// Shared state for the proxy
pub struct ProxyState {
    pub transactions: RwLock<VecDeque<Transaction>>,
    pub stats: RwLock<ProxyStats>,
}

impl ProxyState {
    pub fn new() -> Self {
        Self {
            transactions: RwLock::new(VecDeque::with_capacity(MAX_HISTORY)),
            stats: RwLock::new(ProxyStats::default()),
        }
    }

    pub fn add_transaction(&self, transaction: Transaction) {
        let mut transactions = self.transactions.write();
        if transactions.len() >= MAX_HISTORY {
            transactions.pop_front();
        }
        transactions.push_back(transaction);
    }

    pub fn update_stats<F>(&self, updater: F)
    where
        F: FnOnce(&mut ProxyStats),
    {
        let mut stats = self.stats.write();
        updater(&mut stats);
    }

    pub fn get_stats(&self) -> ProxyStats {
        self.stats.read().clone()
    }

    pub fn get_recent_transactions(&self, count: usize) -> Vec<Transaction> {
        let transactions = self.transactions.read();
        transactions
            .iter()
            .rev()
            .take(count)
            .cloned()
            .collect()
    }
}

impl Default for ProxyState {
    fn default() -> Self {
        Self::new()
    }
}
