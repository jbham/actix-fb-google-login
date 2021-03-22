use actix::prelude::*;
use actix_redis::{Command, RedisActor};
// use actix_web::{error, Error};
use redis_async::{resp::RespValue, resp_array};

use crate::errors::AppError;

use serde::de::DeserializeOwned;

pub async fn get_redis_key<T>(
    key: &str,
    redis: &actix_web::web::Data<Addr<RedisActor>>,
) -> Result<Option<T>, AppError>
where
    T: DeserializeOwned,
{
    // let aed = redis.send(Command(resp_array!["GET", key])).await;
    return match redis.send(Command(resp_array!["GET", key])).await {
        Err(e) => {
            debug!("Redis error #1 from get_redis_key function: {:?}", e);
            Err(AppError::NOT_FOUND.into())
        }
        Ok(res) => match res {
            Ok(val) => {
                // debug!("VAL: {:?}", &val);
                match val {
                    RespValue::Error(err) => {
                        debug!("Redis error #2 from get_redis_key function: {:?}", err);
                        return Err(AppError::NOT_FOUND.into());
                    }
                    RespValue::SimpleString(s) => {
                        if let Ok(val) = serde_json::from_str(&s) {
                            return Ok(Some(val));
                        }
                    }

                    // usage of "ref" was found here:
                    // https://stackoverflow.com/questions/57797677/borrowed-value-does-not-live-long-enough-when-use-generic-lifecycle
                    RespValue::BulkString(ref s) => {
                        // let asd = serde_json::from_slice::<T>(s).unwrap();
                        if let Ok(val1) = serde_json::from_slice::<T>(s) {
                            return Ok(Some(val1));
                        }
                    }
                    _ => (),
                }
                Ok(None)
            }
            Err(err) => {
                debug!("Redis error #3 from get_redis_key function: {:?}", err);
                Err(AppError::NOT_FOUND.into())
            }
        },
    };
}

pub async fn set_redis_key_with_expiration(
    key: String,
    body: String,
    ttl_in_seconds: String,
    redis: &actix_web::web::Data<Addr<RedisActor>>,
) -> Result<(), AppError> {
    match redis
        .send(Command(resp_array![
            "SET",
            key,
            body,
            "EX",
            &ttl_in_seconds
        ]))
        .await
    {
        Err(e) => {
            debug!(
                "Redis error #1 from set_redis_key_with_expiration function: {:?}",
                e
            );
            Err(AppError::NOT_FOUND.into())
        }
        Ok(redis_result) => match redis_result {
            Ok(_) => Ok(()),
            Err(err) => {
                debug!(
                    "Redis error #2 from set_redis_key_with_expiration function: {:?}",
                    err
                );
                Err(AppError::NOT_FOUND.into())
            }
        },
    }
}
