use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use tokio::{join, sync::RwLock};

#[derive(Debug, Default)]
pub struct Database {
    content: RwLock<HashMap<Vec<u8>, Vec<u8>>>,
    expiry_millis: RwLock<HashMap<Vec<u8>, u64>>,
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

    pub async fn expire_at_millis(&self, key: &[u8], timestamp: u64) {
        let key = key.to_vec();
        self.expiry_millis.write().await.insert(key, timestamp);
    }

    pub async fn expire_in_millis(&self, key: &[u8], delta: u64) {
        self.expire_at_millis(key, now_unix_millis() + delta).await;
    }

    pub async fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let content = self.content.read().await.get(key).cloned();
        let expire_at = self.expiry_millis.read().await.get(key).copied();

        // Check if key is expired
        if matches!(expire_at, Some(val) if val < now_unix_millis()) {
            // Do some cleanup
            self.content.write().await.remove(key);
            self.expiry_millis.write().await.remove(key);
            return None;
        }

        content
    }

    pub async fn keys(&self) -> Vec<Vec<u8>> {
        let (content, expiry_millis) = join!(self.content.read(), self.expiry_millis.read());
        let now = now_unix_millis();

        let mut output = Vec::with_capacity(content.len());
        for key in content.keys() {
            // Ignore expired keys
            if matches!(expiry_millis.get(key), Some(val) if *val < now) {
                continue;
            }

            output.push(key.clone());
        }

        output
    }
}

fn now_unix_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time cannot go before 1970 with this implementation")
        .as_millis() as u64
}
