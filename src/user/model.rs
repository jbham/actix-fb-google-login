use actix_web::{Error, HttpRequest, HttpResponse, Responder};
use anyhow::Result;
use chrono::NaiveDateTime;
use futures::future::{ready, Ready};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::Done;
use sqlx::{FromRow, PgPool, Row};
use std::fmt;
// use strum_macros;
use uuid::Uuid;

use crate::auth::AuthenticatedUser;

#[derive(Serialize, Deserialize, std::fmt::Debug, sqlx::Type)]
#[sqlx(rename = "user_external_idp")]
pub enum UserExternalIDP {
    Google,
    Facebook,
    // Twitter,
    Free,
}

impl fmt::Display for UserExternalIDP {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize)]
pub struct UserPayload {
    pub external_idp_id: f64,
    pub external_idp: UserExternalIDP,
}

impl fmt::Display for UserPayload {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Customize so only `x` and `y` are denoted.
        write!(
            f,
            "external_idp_id: {}, external_idp: {}",
            self.external_idp_id,
            self.external_idp.to_string()
        )
    }
}

// this struct will use to receive user input
#[derive(Serialize, Deserialize)]
pub struct UserRequest {
    pub email: Option<String>,
    pub external_idp_id: f64,
    pub external_idp: UserExternalIDP,
    pub display_name: String,
    pub sign_id: i32,
    pub active: Option<bool>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// this struct will use to receive user input
#[derive(Serialize, Deserialize)]
pub struct UserRequestUpdate {
    pub display_name: Option<String>,
    pub sign_id: i32,
}

// this struct will be used to represent database record
#[derive(Serialize, Deserialize, FromRow, Debug)]
pub struct User {
    pub id: Uuid,
    pub email: Option<String>,
    pub external_idp_id: String,
    pub external_idp: UserExternalIDP,
    pub display_name: Option<String>,
    pub sign_id: i32,
    pub is_internal: Option<bool>,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "User id: {}, email: {:?}, external_idp_id: {:?}, external_idp: {}, display_name: {:?}, sign_id: {}", self.id, self.email, self.external_idp_id, self.external_idp, self.display_name, self.sign_id)
    }
}

// implementation of Actix Responder for User struct so we can return User from action handler
impl Responder for User {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();

        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)))
    }
}

impl User {
    pub async fn find_all(user: AuthenticatedUser, pool: &PgPool) -> Result<Vec<User>> {
        // let mut users = vec![];
        let users: Vec<User> = sqlx::query_as(
            r#"
                select
                    id,
                    external_idp,
                    external_idp_id,
                    display_name,
                    sign_id,
                    email,
                    is_internal
                from
                    USERS
                where email = $1
                order by
                    ID
            "#,
        )
        .bind(user.user.email)
        .fetch_all(pool)
        .await?;

        // for rec in recs {
        //     users.push(User {
        //         id: rec.id,
        //         external_idp: rec.external_idp,
        //         external_idp_id: rec.external_idp_id,
        //         display_name: rec.display_name,
        //         sign_id: rec.sign_id,
        //         email: rec.email,
        //         is_internal: rec.is_internal,
        //     });
        // }

        Ok(users)
    }

    // find by our in house UUID
    pub async fn find_by_id(user: AuthenticatedUser, pool: &PgPool) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "select
                    s.id,
                    s.external_idp_id ,
                    s.external_idp ,
                    s.display_name ,
                    s.sign_id,
                    s.email,
                    is_internal
                from
                    users s
                join sign s2 on
                    s2.id = s.sign_id
                where
                    id = $1",
        )
        .bind(user.user.id)
        .fetch_optional(pool)
        .await?;

        Ok(user)

        // Ok(User {
        //     id: rec.id,
        //     external_idp: rec.external_idp,
        //     external_idp_id: rec.external_idp_id,
        //     display_name: rec.display_name,
        //     sign_id: rec.sign_id,
        // })
    }

    // find by google, facebook, twitter user ID
    pub async fn find_by_idp_id(id: &String, pool: &PgPool) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "select
                    s.id,
                    s.external_idp_id ,
                    s.external_idp ,
                    s.display_name ,
                    s.sign_id,
                    s.email,
                    is_internal
                from
                    users s
                join sign s2 on
                    s2.id = s.sign_id
                where
                    s.external_idp_id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(user)
        // Ok(User {
        //     id: rec.id,
        //     external_idp: rec.external_idp,
        //     external_idp_id: rec.external_idp_id,
        //     display_name: rec.display_name,
        //     sign_id: rec.sign_id,
        // })
    }

    // find by email id
    pub async fn find_by_email_id(email: &str, pool: &PgPool) -> Result<Option<User>> {
        let rec = sqlx::query_as::<_, User>(
            "select
                    s.id,
                    s.external_idp_id ,
                    s.external_idp ,
                    s.display_name ,
                    s.sign_id,
                    s.email,
                    is_internal
                from
                    users s
                join sign s2 on
                    s2.id = s.sign_id
                where
                    s.email = $1",
        )
        .bind(&email)
        .fetch_optional(pool)
        .await?;

        Ok(rec)
    }

    pub async fn create(
        _user: AuthenticatedUser,
        user_data: UserRequest,
        pool: &PgPool,
    ) -> Result<User> {
        let mut tx = pool.begin().await?;
        let user = sqlx::query("INSERT INTO USERS (EMAIL, EXTERNAL_IDP, EXTERNAL_IDP_ID, DISPLAY_NAME, SIGN_ID, ACTIVE, CREATED_AT, UPDATED_AT) 
                VALUES ($1, $2, $3, $4, $5, $6) returning ID, EXTERNAL_IDP, EXTERNAL_IDP_ID, DISPLAY_NAME, SIGN_ID, EMAIL, IS_INTERNAL")    
                .bind(user_data.email)
                .bind(user_data.external_idp)
                .bind(user_data.external_idp_id)
                .bind(user_data.display_name)
                .bind(user_data.sign_id)
                .bind(user_data.active)
                .bind(user_data.created_at)
                .bind(user_data.updated_at)
                .map(|row: PgRow| {
                    User {
                        id: row.get(0),
                        external_idp: row.get(1),
                        external_idp_id: row.get(2),
                        display_name: row.get(3),
                        sign_id: row.get(4),
                        email: row.get(5),
                        is_internal: row.get(6)
                    }
                })
                .fetch_one(&mut tx)
                .await?;

        tx.commit().await?;
        Ok(user)
    }

    pub async fn update(
        user: AuthenticatedUser,
        user_data: UserRequestUpdate,
        pool: &PgPool,
    ) -> Result<User> {
        let mut tx = pool.begin().await.unwrap();
        let user = sqlx::query("UPDATE USERS set DISPLAY_NAME = $1, SIGN_ID = $2 where ID = $3 
                                    RETURNING ID, EXTERNAL_IDP, EXTERNAL_IDP_ID, DISPLAY_NAME, SIGN_ID, EMAIL, IS_INTERNAL")
            .bind(user_data.display_name)
            .bind(user_data.sign_id)
            .bind(user.user.id)
            .map(|row: PgRow| {
                User {
                    id: row.get(0),
                    external_idp: row.get(1),
                    external_idp_id: row.get(2),
                    display_name: row.get(3),
                    sign_id: row.get(4),
                    email: row.get(5),
                    is_internal: row.get(6)
                }
            })
            .fetch_one(&mut tx)
            .await?;

        tx.commit().await.unwrap();
        Ok(user)
    }

    pub async fn delete(user: AuthenticatedUser, pool: &PgPool) -> Result<u64> {
        // TODO:: delete from REDIS as well

        let mut tx = pool.begin().await?;
        let deleted = sqlx::query("DELETE FROM USERS where ID = $1")
            .bind(user.user.id)
            .execute(&mut tx)
            .await?;

        tx.commit().await?;
        Ok(deleted.rows_affected())
    }
}
