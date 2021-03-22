// useful links:
// https://stackoverflow.com/questions/57892819/how-to-return-an-early-response-from-an-actix-web-middleware

use std::{
    fmt,
    time::{SystemTime, UNIX_EPOCH},
};

// use actix_session::*;

use actix::prelude::*;
use actix_redis::RedisActor;
// use redis_async::resp_array;

use crate::{
    errors::AppError,
    user::{User, UserExternalIDP},
    InternalAppData,
};
use actix_http::http::HeaderValue;
// use actix_web::error::ErrorUnauthorized;
use actix_web::{dev, web::Data, FromRequest, HttpRequest};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use futures::future::{ready, BoxFuture};
// use futures_util::future::{err, ok, Ready};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::redis::{get_redis_key, set_redis_key_with_expiration};

#[derive(Serialize, Deserialize)]
pub struct Facebook<T> {
    pub data: T,
}

// impl From<Facebook<FacebookResponseData>> for RespValue {
//     fn from(data: Facebook<FacebookResponseData>) -> Self {
//         let j = serde_json::to_string(&data).unwrap();
//         let asd = RespValue::SimpleString(j);
//         asd
//     }
// }
#[derive(Serialize, Deserialize)]
pub struct FacebookResponseData {
    pub app_id: String,
    pub r#type: String,
    pub application: String,
    pub data_access_expires_at: u64,
    pub error: Option<FacebookError>,
    pub expires_at: u64,
    pub is_valid: bool,
    pub scopes: Vec<String>,
    pub user_id: String,
}
#[derive(Serialize, Deserialize)]
pub struct FacebookError {
    pub code: u32,
    pub message: String,
    pub subcode: u32,
}

#[derive(Debug)]
pub struct AuthenticatedUser {
    pub user: User,
}

impl fmt::Display for AuthenticatedUser {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AuthenticatedUser: \n: {}", self.user)
    }
}

impl FromRequest for AuthenticatedUser {
    type Error = AppError;
    // type Future = Ready<Result<Self, Self::Error>>;
    type Future = BoxFuture<'static, Result<Self, Self::Error>>;

    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut dev::Payload) -> Self::Future {
        let db_pool = req.app_data::<Data<PgPool>>().unwrap().clone();

        let internal_app_data = req.app_data::<Data<InternalAppData>>().unwrap().clone();
        let redis = req.app_data::<Data<Addr<RedisActor>>>().unwrap().clone();
        // debug!("{:?}", internal_app_data);

        // check header to identify IDP:
        let idp = req.headers().get("idp");

        // create a dummy header value to use as Err later on
        let none_header = HeaderValue::from_static("hello");
        let idp = idp.unwrap_or_else(|| &none_header);

        let idp = if idp.to_str().unwrap() == "Google" {
            Ok(UserExternalIDP::Google)
        } else if idp.to_str().unwrap() == "Facebook" {
            Ok(UserExternalIDP::Facebook)
        } else {
            Err("Invalid IDP provided")
        };

        // let user_data = req.app_data::<Json<UserTestPayload>>().unwrap().clone();
        let bearer_result = BearerAuth::from_request(req, payload).into_inner();

        let current_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        match (idp, bearer_result) {
            //handle Google authentication here
            (Ok(UserExternalIDP::Google), Ok(bearer)) => {
                let future = async move {
                    let key = format!("google-{}", bearer.token());

                    // debug!("google key: {}", key);

                    let google_key = get_redis_key::<User>(key.as_str(), &redis).await?;

                    match google_key {
                        // we have the user data in redis, let's use that return data. This data will expire in redis
                        // based on the token expiration data. (hopefully user isn't already deleted from database lol)
                        Some(user) => {
                            // let user: User = serde_json::from_str(&key)?;
                            Ok(AuthenticatedUser { user })
                        }

                        // we don't have key in redis, evaluate and store in redis
                        _ => {
                            let g_client = &internal_app_data.google_client;
                            let g_data = g_client.verify_id_token_async(bearer.token()).await;
                            match g_data {
                                Ok(token) => {
                                    let user_id = token.get_claims().get_subject();
                                    let user = User::find_by_idp_id(&user_id, &db_pool)
                                        .await?
                                        .ok_or_else(|| {
                                            debug!("User not found with IDP: {}", user_id);
                                            AppError::NOT_AUTHORIZED
                                        })?;

                                    // let's save this user info in REDIS
                                    let key_expire_at_in_seconds =
                                        token.get_claims().get_expires_at() - current_timestamp;
                                    let user_serialized = serde_json::to_string(&user).unwrap();

                                    let _key_set = set_redis_key_with_expiration(
                                        key,
                                        user_serialized,
                                        key_expire_at_in_seconds.to_string(),
                                        &redis,
                                    )
                                    .await?;

                                    Ok(AuthenticatedUser { user })
                                }
                                Err(e) => {
                                    debug!("Error while decoding Google token: {:?}", e);
                                    Err(AppError::NOT_AUTHORIZED.into())
                                }
                            }
                        }
                    }
                };

                Box::pin(future)
            }

            //handle Facebook authentication here
            (Ok(UserExternalIDP::Facebook), Ok(bearer)) => {
                let url = format!(
                    "https://graph.facebook.com/debug_token?input_token={}&access_token={}",
                    bearer.token(),
                    internal_app_data.facebook_data["facebook_access_token"]
                );

                let future = async move {
                    let key = format!("facebook-{}", bearer.token());

                    // return type should be a User
                    let facebook_user = get_redis_key::<User>(key.as_str(), &redis).await?;

                    match facebook_user {
                        // we have the user data in redis, let's use that return data. This data will expire in redis
                        // based on the token expiration data. (hopefully user isn't already deleted from database lol)
                        Some(user) => {
                            Ok(AuthenticatedUser { user })
                            /*if let Ok(data) =
                                serde_json::from_str::<Facebook<FacebookResponseData>>(&fb_user)
                            {
                                if data.data.expires_at < current_timestamp {
                                    // Token is NOT valid.
                                    Err(AppError::INVALID_CREDENTIALS.into())
                                } else {
                                    // Good....the token is valid
                                    let user = User::find_by_idp_id(&data.data.user_id, &db_pool)
                                        .await?
                                        .ok_or_else(|| {
                                            debug!(
                                                "User not found with IDP: {}",
                                                &data.data.user_id
                                            );
                                            AppError::NOT_AUTHORIZED
                                        })?;

                                    Ok(AuthenticatedUser { user })
                                }
                            } else {
                                Err(AppError::NOT_AUTHORIZED.into())
                            }
                            */
                        }
                        _ => {
                            let body = reqwest::get(&url).await;
                            match body {
                                Ok(response) => {
                                    let f_id = response.text().await.unwrap();
                                    if let Ok(data) = serde_json::from_str::<
                                        Facebook<FacebookResponseData>,
                                    >(&f_id)
                                    {
                                        // Facebook says the token is valid and it belongs to our specific APP in facebook
                                        if data.data.is_valid
                                            && data.data.app_id
                                                == internal_app_data.facebook_data
                                                    ["facebook_app_id"]
                                        {
                                            let user =
                                                User::find_by_idp_id(&data.data.user_id, &db_pool)
                                                    .await?
                                                    .ok_or_else(|| {
                                                        debug!(
                                                            "User not found with IDP: {}",
                                                            &data.data.user_id
                                                        );
                                                        AppError::NOT_AUTHORIZED
                                                    })?;

                                            // push into REDIS so we don't make request to facebook again to verify
                                            let key_expire_at_in_seconds =
                                                data.data.expires_at - current_timestamp;

                                            let user_serialized =
                                                serde_json::to_string(&user).unwrap();

                                            let _key_set = set_redis_key_with_expiration(
                                                key,
                                                user_serialized,
                                                key_expire_at_in_seconds.to_string(),
                                                &redis,
                                            )
                                            .await?;

                                            // let _one = redis.send(Command(resp_array!["SET", key.as_str(), f_id, "EX", key_expire_at_in_seconds.to_string() ])).await;
                                            // let _two = redis.send(Command(resp_array!["EXPIREAT", key.as_str(), key_expire_at_in_seconds.to_string() ])).await;

                                            Ok(AuthenticatedUser { user })
                                        } else {
                                            // Facebook says the token is INVALID
                                            Err(AppError::CREDENTIAL_EXPIRED.into())
                                        }
                                    } else {
                                        Err(AppError::NOT_AUTHORIZED.into())
                                    }
                                }
                                Err(e) => {
                                    debug!("Error while decoding facebook token: {:?}", e);
                                    Err(AppError::NOT_AUTHORIZED.into())
                                }
                            }
                        }
                    }
                };

                Box::pin(future)
            }
            _ => {
                debug!(
                    "Proper IDP not provided. But, GET requests will flow through. Following are the headers provide: \n {:?}",
                    req.headers()
                );

                let method = req.method();

                // Allow free users to GET anything as unauthenticated user
                if method == "GET" {
                    let future = async move {
                        let key = format!("free_user");
                        let free_user_key = get_redis_key::<User>(key.as_str(), &redis).await?;

                        match free_user_key {
                            Some(user) => Ok(AuthenticatedUser { user }),
                            _ => {
                                let user =
                                    User::find_by_email_id(&internal_app_data.dummy_user, &db_pool)
                                        .await?
                                        .ok_or_else(|| {
                                            debug!(
                                                "User '{}' not found",
                                                &internal_app_data.sysadmin
                                            );
                                            AppError::NOT_AUTHORIZED
                                        })?;

                                let key_expire_at_in_seconds: u64 = 86600;
                                let user_serialized = serde_json::to_string(&user).unwrap();

                                let _key_set = set_redis_key_with_expiration(
                                    key,
                                    user_serialized,
                                    key_expire_at_in_seconds.to_string(),
                                    &redis,
                                )
                                .await?;

                                Ok(AuthenticatedUser { user })
                            }
                        }
                    };
                    Box::pin(future)
                } else {
                    // however if the method is anything other than GET then we need to stop them here
                    let error = ready(Err(AppError::NOT_AUTHORIZED.into()));
                    Box::pin(error)
                }
            }
        }
    }
}
