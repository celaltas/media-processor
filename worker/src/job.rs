use std::collections::HashMap;

use redis::{FromRedisValue, RedisResult, Value};

#[derive(Debug)]
pub struct JobMetadata {
    pub id: String,
    pub status: String,
    pub path: String,
}

impl FromRedisValue for JobMetadata {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        let hash: HashMap<String, String> = FromRedisValue::from_redis_value(v)?;

        Ok(JobMetadata {
            id: hash.get("id").cloned().ok_or_else(|| {
                redis::RedisError::from((redis::ErrorKind::TypeError, "Missing field 'id'"))
            })?,
            status: hash.get("status").cloned().ok_or_else(|| {
                redis::RedisError::from((redis::ErrorKind::TypeError, "Missing field 'status'"))
            })?,
            path: hash.get("path").cloned().ok_or_else(|| {
                redis::RedisError::from((redis::ErrorKind::TypeError, "Missing field 'path'"))
            })?,
        })
    }
}
