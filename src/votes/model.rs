use crate::{
    auth::AuthenticatedUser,
    errors::AppError,
    quality::{Choice, QualityDefinedBy, QualityType},
};
use actix_web::{Error, HttpRequest, HttpResponse, Responder};
use anyhow::Result;
use futures::future::{ready, Ready};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

#[derive(Serialize, Deserialize)]
pub struct QualityChoiceVote {
    pub id: i32,
    pub quality_long: String,
    pub quality_short: String,
    pub quality_type: QualityType,
    pub sign_1_id: i32,
    pub sign_2_id: i32,
    pub defined_by: QualityDefinedBy,
    pub choice: Vec<Choice>,
    pub choice_value: String,
}

#[derive(Serialize, Deserialize, PartialEq, FromRow)]
pub struct ValidChoices {
    pub id: i32,
}

impl Responder for QualityChoiceVote {
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

impl QualityChoiceVote {
    // find all votes casted by a given user
    pub async fn find_all_by_user(
        user: AuthenticatedUser,
        pool: &PgPool,
    ) -> Result<Vec<QualityChoiceVote>> {
        let mut votes = vec![];

        let recs = sqlx::query!(r#"
                select
                    row_to_json(qual)::text "json"
                from
                    (
                    select
                        q.id, q.quality_long, q.quality_short, q.quality_type, q.defined_by, q.sign_1_id, q.sign_2_id, c2.choice_value , (
                        select
                            json_agg(alb)
                        from
                            (
                            select
                                id, choice_value
                            from
                                choice
                            where
                                quality_id = q.id ) alb ) as choice
                    from
                        quality as q
                    join choice c2 on
                        c2.quality_id = q.id
                    join votes v2 on
                        v2.quality_id = q.id
                        and v2.choice_id = c2.id
                    where
                        v2.user_id = $1
                        --	group by
                        --		q.id, q.quality_long, q.quality_short, q.quality_type, q.defined_by, q.sign_1_id, q.sign_2_id, c2.choice_value 
                ) qual
                "#,
                user.user.id
                )
                .fetch_all(pool)
                .await?;

        for rec in recs {
            match rec.json {
                Some(record) => {
                    let v: QualityChoiceVote = serde_json::from_str(&record)?;
                    votes.push(v)
                }
                _ => println!("Nada found"),
            }
        }

        Ok(votes)
    }

    // delete all votes casted by a user
    pub async fn delete_all_by_user(user: AuthenticatedUser, pool: &PgPool) -> Result<()> {
        let mut tx = pool.begin().await?;
        sqlx::query("DELETE FROM votes WHERE user_id = $1")
            .bind(&user.user.id)
            .execute(&mut tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    // delete a single vote casted by a user
    pub async fn delete_single_vote_by_user(
        user: AuthenticatedUser,
        q_id: i32,
        c_id: i32,
        pool: &PgPool,
    ) -> Result<(), AppError> {
        let mut tx = pool.begin().await?;
        sqlx::query("DELETE FROM votes WHERE user_id = $1 and quality_id = $2 and choice_id = $3")
            .bind(&user.user.id)
            .bind(&q_id)
            .bind(&c_id)
            .execute(&mut tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    // user wants to update their choice
    pub async fn update(
        user: AuthenticatedUser,
        q_id: i32,
        c_id: i32,
        pool: &PgPool,
    ) -> Result<(), AppError> {
        // first check if the incoming choice id (c_id) is a valid choice for the incoming quality
        let valid_choices: Vec<ValidChoices> =
            sqlx::query_as(r#"select id from choice where quality_id = $1"#)
                .bind(&q_id)
                .fetch_all(pool)
                .await?;

        let temp_vc = ValidChoices { id: c_id };
        if !valid_choices.contains(&temp_vc) {
            return Err(AppError::INVALID_CHOICE.into());
        }

        let mut tx = pool.begin().await?;

        sqlx::query(
            "UPDATE VOTES 
                set 
                    choice_id = $1
                where 
                    quality_id = $2
                    and user_id = $3",
        )
        .bind(&c_id)
        .bind(&q_id)
        .bind(&user.user.id)
        .fetch_optional(&mut tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    // user wants to cast a vote
    pub async fn create(
        user: AuthenticatedUser,
        q_id: i32,
        c_id: i32,
        pool: &PgPool,
    ) -> Result<(), AppError> {
        // first check if the incoming choice id (c_id) is a valid choice for the incoming quality
        let valid_choices: Vec<ValidChoices> =
            sqlx::query_as(r#"select id from choice where quality_id = $1"#)
                .bind(&q_id)
                .fetch_all(pool)
                .await?;

        let temp_vc = ValidChoices { id: c_id };
        if !valid_choices.contains(&temp_vc) {
            return Err(AppError::INVALID_CHOICE.into());
        }

        // ensure the user hasn't already casted this vote
        let valid_choices: Vec<ValidChoices> = sqlx::query_as(
            r#"select id from votes WHERE user_id = $1 and quality_id = $2 and choice_id = $3"#,
        )
        .bind(&user.user.id)
        .bind(&q_id)
        .bind(&c_id)
        .fetch_all(pool)
        .await?;

        if valid_choices.len() > 0 {
            return Err(AppError::ALREADY_VOTED.into());
        }

        // if we are here then that means its okay to open a transaction to start inserting votes
        let mut tx = pool.begin().await?;

        let _qu = sqlx::query(
            "
                INSERT INTO VOTES (
                    quality_id, 
                    choice_id, 
                    user_id)
                VALUES 
                    ($1, $2, $3)",
        )
        .bind(&q_id)
        .bind(&c_id)
        .bind(&user.user.id)
        .fetch_optional(&mut tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }
}
