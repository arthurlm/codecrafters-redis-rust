use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use tokio::{join, sync::RwLock};

use crate::rdb::RedisString;

#[derive(Debug, Default)]
pub struct Database {
    content: RwLock<HashMap<RedisString, RedisString>>,
    expiry_millis: RwLock<HashMap<RedisString, u64>>,
}

impl Database {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn set<K, V>(&self, key: K, value: V)
    where
        K: Into<RedisString>,
        V: Into<RedisString>,
    {
        let key = key.into();
        let value = value.into();

        self.content.write().await.insert(key, value);
    }

    pub async fn expire_at_millis<K>(&self, key: K, timestamp: u64)
    where
        K: Into<RedisString>,
    {
        let key = key.into();

        self.expiry_millis.write().await.insert(key, timestamp);
    }

    pub async fn expire_in_millis<K>(&self, key: K, delta: u64)
    where
        K: Into<RedisString>,
    {
        self.expire_at_millis(key, now_unix_millis() + delta).await;
    }

    pub async fn get<K>(&self, key: K) -> Option<RedisString>
    where
        K: Into<RedisString>,
    {
        let key = key.into();

        let content = self.content.read().await.get(&key).cloned();
        let expire_at = self.expiry_millis.read().await.get(&key).copied();

        // Check if key is expired
        if matches!(expire_at, Some(val) if val < now_unix_millis()) {
            // Do some cleanup
            self.content.write().await.remove(&key);
            self.expiry_millis.write().await.remove(&key);
            return None;
        }

        content
    }

    pub async fn keys(&self) -> Vec<RedisString> {
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
