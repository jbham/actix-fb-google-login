use crate::votes::QualityChoiceVote;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use sqlx::PgPool;

use crate::auth::AuthenticatedUser;

// find all votes casted by a given user
#[get("/votes")]
async fn find_all_by_user(user: AuthenticatedUser, db_pool: web::Data<PgPool>) -> impl Responder {
    let result = QualityChoiceVote::find_all_by_user(user, db_pool.get_ref()).await;

    match result {
        Ok(votes) => HttpResponse::Ok().json(votes),
        Err(e) => {
            debug!(
                "Error occurred in Votes >> routes.rs find_all_by_user function: \n{}",
                e
            );
            HttpResponse::BadRequest().body("Error trying to read all votes by user")
        }
    }
}

// delete all votes casted by a user
#[delete("/votes")]
async fn delete_all_by_user(user: AuthenticatedUser, db_pool: web::Data<PgPool>) -> impl Responder {
    let result = QualityChoiceVote::delete_all_by_user(user, db_pool.get_ref()).await;

    match result {
        Ok(votes) => HttpResponse::Ok().json(votes),
        Err(e) => {
            debug!(
                "Error occurred in Votes >> routes.rs delete_all_by_user function: \n{}",
                e
            );
            HttpResponse::BadRequest().body("Error trying to delete all votes by user")
        }
    }
}

// delete a single vote casted by a user
#[delete("/votes/{q_id}/{c_id}")]
async fn delete_single_vote_by_user(
    user: AuthenticatedUser,
    param: web::Path<(i32, i32)>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    let q_id = (param.0).0;
    let c_id = (param.0).1;

    let result =
        QualityChoiceVote::delete_single_vote_by_user(user, q_id, c_id, db_pool.get_ref()).await;

    match result {
        Ok(votes) => HttpResponse::Ok().json(votes),
        Err(e) => {
            debug!(
                "Error occurred in Votes >> routes.rs delete_single_vote_by_user function: \n{}",
                e
            );
            HttpResponse::BadRequest().body("Error trying to delete a vote by user")
        }
    }
}

// user wants to update their choice
#[put("/votes/{q_id}/{c_id}")]
async fn update(
    user: AuthenticatedUser,
    param: web::Path<(i32, i32)>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    let q_id = (param.0).0;
    let c_id = (param.0).1;

    let result = QualityChoiceVote::update(user, q_id, c_id, db_pool.get_ref()).await;

    match result {
        Ok(votes) => HttpResponse::Ok().json(votes),
        Err(e) => {
            debug!(
                "Error occurred in Votes >> routes.rs update function: \n{}",
                e
            );
            HttpResponse::BadRequest().json(e)
        }
    }
}

// user wants to cast a vote
#[post("/votes/{q_id}/{c_id}")]
async fn create(
    user: AuthenticatedUser,
    param: web::Path<(i32, i32)>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    let q_id = (param.0).0;
    let c_id = (param.0).1;

    let result = QualityChoiceVote::create(user, q_id, c_id, db_pool.get_ref()).await;

    match result {
        Ok(votes) => HttpResponse::Ok().json(votes),
        Err(e) => {
            debug!(
                "Error occurred in Votes >> routes.rs create function: \n{}",
                e
            );
            HttpResponse::BadRequest().json(e)
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all_by_user);
    cfg.service(delete_single_vote_by_user);
    cfg.service(delete_all_by_user);
    cfg.service(update);
}
