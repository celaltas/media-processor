use deadpool::managed::{Manager, RecycleError, RecycleResult};
use redis::{AsyncCommands, RedisError, aio::MultiplexedConnection};
use std::{
    borrow::Cow,
    ops::{Deref, DerefMut},
};

use crate::job::JobMetadata;

pub struct RedisConnectionManager {
    client: redis::Client,
}

pub struct RedisConnection {
    actual: MultiplexedConnection,
}

impl RedisConnectionManager {
    pub fn new(client: redis::Client) -> Self {
        Self { client }
    }
}

impl Manager for RedisConnectionManager {
    type Type = RedisConnection;

    type Error = RedisError;

    async fn create(&self) -> Result<RedisConnection, RedisError> {
        Ok(RedisConnection {
            actual: self.client.get_multiplexed_async_connection().await?,
        })
    }

    async fn recycle(
        &self,
        obj: &mut Self::Type,
        _metrics: &deadpool::managed::Metrics,
    ) -> RecycleResult<Self::Error> {
        let resp = obj.actual.ping::<String>().await;
        match resp {
            Ok(resp) if resp == "PONG" => Ok(()),
            Ok(_) | Err(_) => Err(RecycleError::Message(Cow::from("Recycling failed"))),
        }
    }
}

impl Deref for RedisConnection {
    type Target = MultiplexedConnection;

    fn deref(&self) -> &Self::Target {
        &self.actual
    }
}

impl DerefMut for RedisConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.actual
    }
}

impl RedisConnection {
    pub async fn enqueue_job(&mut self, job_id: &str) -> Result<(), RedisError> {
        self.lpush("image:process", job_id).await
    }

    pub async fn dequeue_job(&mut self) -> Result<String, RedisError> {
        self.lpop("image:process", None).await
    }

    pub async fn get_job_metadata(
        &mut self,
        job_id: &str,
    ) -> Result<Option<JobMetadata>, RedisError> {
        self.hgetall(job_id).await
    }

    pub async fn update_job_status(
        &mut self,
        job_id: &str,
        new_status: &str,
    ) -> Result<(), RedisError> {
        self.hset(job_id, "status", new_status).await
    }
}

#[cfg(test)]
mod tests {
    use deadpool::managed::{Pool, PoolConfig};
    use redis::AsyncCommands;

    use crate::connection_manager::RedisConnectionManager;

    #[tokio::test]
    async fn test_redis_connection_manager() {
        let client = redis::Client::open("redis://default:secret_passwd@localhost:6379/0").unwrap();
        let connection_pool =
            Pool::<RedisConnectionManager>::builder(RedisConnectionManager::new(client))
                .config(PoolConfig::default())
                .max_size(5)
                .build()
                .expect("Failed to create connection pool");
        let mut conn = connection_pool
            .get()
            .await
            .expect("failed to get connection from pool");

        let _: Option<u64> = conn
            .incr("async_pool_test_key", 1)
            .await
            .expect("Failed to increment key");
        let curr: Option<u64> = conn
            .get("async_pool_test_key")
            .await
            .expect("Failed to get key");
        assert_eq!(curr, Some(1));
    }
}
