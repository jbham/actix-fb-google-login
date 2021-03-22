use actix_web::{Error, HttpRequest, HttpResponse, Responder};
use anyhow::Result;
use futures::future::{ready, Ready};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

#[derive(Deserialize, Serialize, FromRow, Debug)]
pub struct Signs {
    pub id: i32,
    pub sign_name: String,
}

// implementation of Actix Responder for Quality struct so we can return Quality from action handler
impl Responder for Signs {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();

        // debug!("here second");

        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)))
    }
}

impl Signs {
    pub async fn find_all(pool: &PgPool) -> Result<Vec<Signs>> {
        let signs: Vec<Signs> = sqlx::query_as(
            r#"select
                        id,
                        sign_name
                    from
                        sign"#,
        )
        .fetch_all(pool)
        .await?;

        Ok(signs)
    }
}
