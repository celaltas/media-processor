use r2d2_redis::{
    RedisConnectionManager,
    r2d2::PooledConnection,
    redis::{Commands, RedisError},
};

use crate::job::JobMetadata;

pub struct RedisServicePooledCon<'a> {
    conn: &'a mut PooledConnection<RedisConnectionManager>,
}

impl<'a> RedisServicePooledCon<'a> {
    pub fn new(conn: &'a mut PooledConnection<RedisConnectionManager>) -> Self {
        RedisServicePooledCon { conn }
    }

    pub fn enqueue_job(&mut self, job_id: &str) -> Result<(), RedisError> {
        self.conn.lpush("image:process", job_id)
    }

    pub fn dequeue_job(&mut self) -> Result<String, RedisError> {
        self.conn.lpop("image:process")
    }

    pub fn get_job_metadata(&mut self, job_id: &str) -> Result<Option<JobMetadata>, RedisError> {
        self.conn.hgetall(job_id)
    }

    pub fn update_job_status(&mut self, job_id: &str, new_status: &str) -> Result<(), RedisError> {
        self.conn.hset(job_id, "status", new_status)
    }
}
