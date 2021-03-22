use crate::{
    auth::AuthenticatedUser,
    quality::{Quality, QualityChoiceRequest, QualityChoiceUpdateRequest},
};

use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use sqlx::PgPool;
// use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct StartEndOfPayload {
    pub start: i32,
    pub end: Option<i32>,
}

#[get("/qualities")]
async fn find_all(
    _user: AuthenticatedUser,
    db_pool: web::Data<PgPool>,
    // user_payload: web::Json<UserPayload>,
    paginate: web::Query<StartEndOfPayload>,
    sign_1_id: web::Path<i32>,
    sign_2_id: web::Path<Option<i32>>,
) -> impl Responder {
    debug!("UserTestPayload....: {}", paginate.start);

    let result = Quality::find_all(
        sign_1_id.into_inner(),
        sign_2_id.into_inner(),
        db_pool.get_ref(),
    )
    .await;
    match result {
        Ok(qualities) => HttpResponse::Ok().json(qualities),
        Err(e) => {
            debug!("this is the all qualities error {:?}", e);
            HttpResponse::BadRequest().body("Error trying to read all Qualities")
        }
    }
}

#[get("/quality/{id}")]
async fn find(id: web::Path<i32>, db_pool: web::Data<PgPool>) -> impl Responder {
    let result = Quality::find_by_id(id.into_inner(), db_pool.get_ref()).await;
    match result {
        Ok(user) => HttpResponse::Ok().json(user),
        _ => HttpResponse::BadRequest().body("Quality not found"),
    }
}

#[post("/qualities")]
async fn create(
    user: AuthenticatedUser,
    qualities: web::Json<Vec<QualityChoiceRequest>>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    let result = Quality::create(user, qualities.into_inner(), db_pool.get_ref()).await;
    match result {
        Ok(todo) => HttpResponse::Ok().json(todo),
        Err(e) => {
            debug!("CREATE qualities error {:?}", e);
            HttpResponse::BadRequest().body("Error occurred while creating quality")
        }
    }
}

#[put("/quality/{id}")]
async fn update(
    user: AuthenticatedUser,
    id: web::Path<i32>,
    quality: web::Json<QualityChoiceUpdateRequest>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    let result = Quality::update(
        user,
        id.into_inner(),
        quality.into_inner(),
        db_pool.get_ref(),
    )
    .await;
    match result {
        Ok(quality) => HttpResponse::Ok().json(quality),
        Err(e) => {
            debug!("UPDATE quality error {:?}", e);
            HttpResponse::BadRequest().body("Error occurred while updating quality")
        }
    }
}

#[delete("/quality/{id}")]
async fn delete(id: web::Path<i32>, db_pool: web::Data<PgPool>) -> impl Responder {
    let result = Quality::delete(id.into_inner(), db_pool.get_ref()).await;
    match result {
        Ok(()) => HttpResponse::Ok().body("uccessfully deleted"),
        _ => HttpResponse::BadRequest().body("Todo not found"),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all);
    cfg.service(find);
    cfg.service(create);
    cfg.service(update);
    cfg.service(delete);
}
