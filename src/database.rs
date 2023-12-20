use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use tokio::sync::RwLock;

#[derive(Debug, Default)]
pub struct Database {
    content: RwLock<HashMap<Vec<u8>, Vec<u8>>>,
    expiry: RwLock<HashMap<Vec<u8>, Instant>>,
}

impl Database {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn set(&self, key: &[u8], value: &[u8]) {
        let key = key.to_vec();
        let value = value.to_vec();

        self.content.write().await.insert(key, value);
    }

    pub async fn expire_in(&self, key: &[u8], ms_delta: u64) {
        let key = key.to_vec();
        let expire_at = Instant::now() + Duration::from_millis(ms_delta);

        self.expiry.write().await.insert(key, expire_at);
    }

    pub async fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let content = self.content.read().await.get(key).cloned();
        let expire_at = self.expiry.read().await.get(key).copied();

        // Check if key is expired
        if matches!(expire_at, Some(val) if val < Instant::now()) {
            // Do some cleanup
            self.content.write().await.remove(key);
            self.expiry.write().await.remove(key);
            return None;
        }

        content
    }

    pub async fn keys(&self) -> Vec<Vec<u8>> {
        let content = self.content.read().await;
        content.keys().cloned().collect()
    }
}
