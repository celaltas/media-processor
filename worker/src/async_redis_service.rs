use crate::job::JobMetadata;
use redis::{AsyncCommands, RedisError, aio::MultiplexedConnection};

#[derive(Clone)]
pub struct AsyncRedisService {
    connection: MultiplexedConnection,
}

impl AsyncRedisService {
    pub async fn new() -> Self {
        let client = redis::Client::open("redis://default:secret_passwd@localhost:6379/0").unwrap();
        let conn = client.get_multiplexed_async_connection().await.unwrap();
        AsyncRedisService { connection: conn }
    }

    pub async fn enqueue_job(&mut self, job_id: &str) -> Result<(), RedisError> {
        self.connection.lpush("image:process", job_id).await
    }

    pub async fn dequeue_job(&mut self) -> Result<String, RedisError> {
        self.connection.lpop("image:process", None).await
    }

    pub async fn get_job_metadata(
        &mut self,
        job_id: &str,
    ) -> Result<Option<JobMetadata>, RedisError> {
        self.connection.hgetall(job_id).await
    }

    pub async fn update_job_status(
        &mut self,
        job_id: &str,
        new_status: &str,
    ) -> Result<(), RedisError> {
        self.connection.hset(job_id, "status", new_status).await
    }
}

#[cfg(test)]
mod tests {
    use crate::async_redis_service::AsyncRedisService;

    #[tokio::test]
    async fn test_async_redis() {
        let mut redis = AsyncRedisService::new().await;
        // let job_id = redis.dequeue_job();
        let job_id = "cd5429a1-0005-4a84-bb88-78dd06bd9165";
        let _ = redis.enqueue_job(job_id).await;
        // println!("job_id: {:#?}", job_id);
        // let job_id = job_id.unwrap();
        // let data = redis.get_job_metadata(job_id);
        // println!("data: {:#?}", data);
        // let _ = redis.update_job_status(job_id, "new_status");
        // let data = redis.get_job_metadata(job_id);
        // println!("data: {:#?}", data);
    }
}
