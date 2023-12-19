use std::collections::HashMap;

use tokio::sync::RwLock;

#[derive(Debug, Default)]
pub struct Database {
    content: RwLock<HashMap<Vec<u8>, Vec<u8>>>,
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

    pub async fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.content.read().await.get(key).cloned()
    }
}
