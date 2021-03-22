use crate::{
    auth::AuthenticatedUser,
    user::{User, UserRequest, UserRequestUpdate},
};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use sqlx::PgPool;

#[get("/users")]
async fn find_all(user: AuthenticatedUser, db_pool: web::Data<PgPool>) -> impl Responder {
    let result = User::find_all(user, db_pool.get_ref()).await;
    match result {
        Ok(todos) => HttpResponse::Ok().json(todos),
        _ => HttpResponse::BadRequest().body("Error trying to read all USERS from database"),
    }
}

#[get("/user/{id}")]
async fn find(
    user: AuthenticatedUser,
    // id: web::Path<Uuid>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    let result = User::find_by_id(user, db_pool.get_ref()).await;
    match result {
        Ok(user) => HttpResponse::Ok().json(user),
        _ => HttpResponse::BadRequest().body("User not found"),
    }
}

#[post("/user")]
async fn create(
    user: AuthenticatedUser,
    user_data: web::Json<UserRequest>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    let result = User::create(user, user_data.into_inner(), db_pool.get_ref()).await;
    match result {
        Ok(user) => HttpResponse::Ok().json(user),
        _ => HttpResponse::BadRequest().body("Error while creating user"),
    }
}

#[put("/user/{id}")]
async fn update(
    user: AuthenticatedUser,
    user_data: web::Json<UserRequestUpdate>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    let result = User::update(
        user,
        // id.into_inner(),
        user_data.into_inner(),
        db_pool.get_ref(),
    )
    .await;
    match result {
        Ok(user) => HttpResponse::Ok().json(user),
        _ => HttpResponse::BadRequest().body("Error while updating user"),
    }
}

#[delete("/user/{id}")]
async fn delete(user: AuthenticatedUser, db_pool: web::Data<PgPool>) -> impl Responder {
    let result = User::delete(user, db_pool.get_ref()).await;
    match result {
        Ok(rows) => {
            if rows > 0 {
                HttpResponse::Ok().body(format!("Successfully deleted {} record(s)", rows))
            } else {
                HttpResponse::BadRequest().body("User not found")
            }
        }
        _ => HttpResponse::BadRequest().body("User not found"),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all);
    cfg.service(find);
    cfg.service(create);
    cfg.service(update);
    cfg.service(delete);
}
