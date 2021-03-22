use crate::{auth::AuthenticatedUser, signs::Signs};

use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;

#[get("/signs")]
async fn find_all(_user: AuthenticatedUser, db_pool: web::Data<PgPool>) -> impl Responder {
    let result = Signs::find_all(db_pool.get_ref()).await;
    match result {
        Ok(signs) => HttpResponse::Ok()
            .header("Cache-Control", "public, max-age=3600000")
            .json(signs),
        Err(e) => {
            debug!("this is the all signs error {:?}", e);
            HttpResponse::BadRequest().body("Error trying to read all signs")
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all);
}
