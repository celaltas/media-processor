use r2d2_redis::redis::{
    ErrorKind as R2D2ErrorKind, FromRedisValue as R2D2FromRedisValue, RedisError as R2D2RedisError,
    RedisResult as R2D2RedisResult, Value as R2D2Value,
};
use redis::{ErrorKind, FromRedisValue, RedisError, RedisResult, Value};
use std::collections::HashMap;

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
            id: hash
                .get("id")
                .cloned()
                .ok_or_else(|| RedisError::from((ErrorKind::TypeError, "Missing field 'id'")))?,
            status: hash.get("status").cloned().ok_or_else(|| {
                RedisError::from((ErrorKind::TypeError, "Missing field 'status'"))
            })?,
            path: hash
                .get("path")
                .cloned()
                .ok_or_else(|| RedisError::from((ErrorKind::TypeError, "Missing field 'path'")))?,
        })
    }
}

impl R2D2FromRedisValue for JobMetadata {
    fn from_redis_value(v: &R2D2Value) -> R2D2RedisResult<Self> {
        let hash: HashMap<String, String> = R2D2FromRedisValue::from_redis_value(v)?;

        Ok(JobMetadata {
            id: hash.get("id").cloned().ok_or_else(|| {
                R2D2RedisError::from((R2D2ErrorKind::TypeError, "Missing field 'id'"))
            })?,
            status: hash.get("status").cloned().ok_or_else(|| {
                R2D2RedisError::from((R2D2ErrorKind::TypeError, "Missing field 'status'"))
            })?,
            path: hash.get("path").cloned().ok_or_else(|| {
                R2D2RedisError::from((R2D2ErrorKind::TypeError, "Missing field 'path'"))
            })?,
        })
    }
}
