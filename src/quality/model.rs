use actix_web::{Error, HttpRequest, HttpResponse, Responder};
use anyhow::Result;
use chrono::NaiveDateTime;
use futures::future::{ready, Ready};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::{FromRow, PgPool, Row};
use strum_macros;

use crate::auth::AuthenticatedUser;

#[derive(Serialize, Deserialize, std::fmt::Debug, strum_macros::ToString, Clone, sqlx::Type)]
#[sqlx(rename = "valid_quality_types")]
pub enum QualityType {
    Binary,
    Percent,
    Multiple,
}

#[derive(Serialize, Deserialize, std::fmt::Debug, strum_macros::ToString, Clone, sqlx::Type)]
#[sqlx(rename = "quality_defined_by")]
pub enum QualityDefinedBy {
    System,
    User,
}

#[derive(Serialize, Deserialize)]
pub struct ChoiceRequest {
    // its callers responsibility to provide choices
    // pub id: Option<Uuid>,
    pub choice_value: String,
}

// this struct will use to receive user input
#[derive(Serialize, Deserialize)]
pub struct QualityChoiceRequest {
    pub quality_long: String,
    pub quality_short: String,
    pub quality_type: QualityType,
    pub sign_1_id: i32,
    pub sign_2_id: i32,
    // pub defined_by: QualityDefinedBy,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    // pub created_by: Uuid,
    // pub updated_by: Uuid,
    pub choice: Vec<ChoiceRequest>,
}

#[derive(Serialize, Deserialize)]
pub struct QualityChoiceUpdateRequest {
    pub quality_long: String,
    pub quality_short: String,
    pub quality_type: Option<QualityType>,
    pub sign_1_id: i32,
    pub sign_2_id: i32,
    // pub defined_by: QualityDefinedBy,
    // pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    // pub created_by: Option<Uuid>,
    // pub updated_by: Uuid,
    pub choice: Option<Vec<ChoiceUpdateRequest>>,
}

#[derive(Serialize, Deserialize)]
pub struct ChoiceUpdateRequest {
    // its callers responsibility to provide choices
    pub id: i32,
    pub choice_value: String,
}

#[derive(Serialize, Deserialize, std::fmt::Debug, FromRow, Clone)]
pub struct Choice {
    pub id: i32,
    pub choice_value: String,
}

// this struct will be used to represent database record
#[derive(Serialize, Deserialize, std::fmt::Debug, FromRow, Clone)]
pub struct Quality {
    pub id: i32,
    pub quality_long: String,
    pub quality_short: String,
    pub quality_type: QualityType,
    pub sign_1_id: i32,
    pub sign_2_id: i32,
    pub defined_by: QualityDefinedBy,
    // pub created_by: Option<Uuid>,
    // pub updated_by: Option<Uuid>,
    // pub created_at: Option<NaiveDateTime>,
    // pub updated_at: Option<NaiveDateTime>,
    pub choice: Vec<Choice>,
}

#[derive(Serialize, Deserialize, std::fmt::Debug, FromRow)]
pub struct QualityCreatedOrModified {
    pub id: i32,
    pub quality_long: String,
    pub quality_short: String,
    pub quality_type: QualityType,
    pub sign_1_id: i32,
    pub sign_2_id: i32,
    pub defined_by: QualityDefinedBy,
    // pub created_at: Option<NaiveDateTime>,
    // pub updated_at: Option<NaiveDateTime>,
    // pub created_by: Option<Uuid>,
    // pub updated_by: Option<Uuid>,
}

// #[derive(Serialize, Deserialize, std::fmt::Debug, FromRow)]
// pub struct SuccessMessage<'a> {
//     message: &'a str,
//     created_count: i32,
// }

// implementation of Actix Responder for Quality struct so we can return Quality from action handler
impl Responder for Quality {
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

impl Quality {
    pub async fn find_all(
        sign_1_id: i32,
        sign_2_id: Option<i32>,
        pool: &PgPool,
    ) -> Result<Vec<Quality>> {
        let mut qualities = vec![];

        // Leaving this here for now and not using it because it kind of slower right now than postgres' JSON conversion
        /*
        let recprds1 = sqlx::query!("
            select
                    q.id, q.quality_long, q.quality_short, q.quality_type as \"quality_type: QualityType\", (
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
        ")
                .fetch_all(pool)
                .await?;

        let records = sqlx::query!("select q.id, q.quality_long, q.quality_short, q.quality_type as \"quality_type: QualityType\" from quality q"
                )
                .fetch_all(pool)
                .await?;
        debug!("RECORDS: {:?}", records);
        */

        let recs = sqlx::query!(r#"
                select
                    row_to_json(qual)::text "json"
                from
                    (
                    select
                        q.id, q.quality_long, q.quality_short, q.quality_type, q.defined_by, q.sign_1_id, q.sign_2_id, (
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
                    where
                        q.sign_1_id = $1
                        and q.sign_2_id = $2) qual
                "#,
                sign_1_id, sign_2_id
                )
                .fetch_all(pool)
                .await?;

        for rec in recs {
            match rec.json {
                Some(record) => {
                    let v: Quality = serde_json::from_str(&record)?;
                    qualities.push(v)
                }
                _ => println!("Nada found"),
            }
        }

        // println!("This is quality??? \n{:?}", qualities);

        Ok(qualities)
    }

    // Used instructions from here to implement ENUM here:
    // https://cetra3.github.io/blog/implementing-a-jobq-sqlx/
    pub async fn find_by_id(id: i32, pool: &PgPool) -> Result<Option<Quality>> {
        let rec = sqlx::query!(
            r#"select
                row_to_json(qual)::text "json"
            from
                (
                select
                    q.id, q.quality_long, q.quality_short, q.quality_type, (
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
                    quality as q where q.id = $1) qual"#,
            id
        )
        .fetch_one(pool)
        .await?;

        match rec.json {
            Some(record) => {
                let v: Quality = serde_json::from_str(&record)?;
                Ok(Some(v))
            }
            _ => Ok(None),
        }
    }

    pub async fn create(
        user: AuthenticatedUser,
        quality: Vec<QualityChoiceRequest>,
        pool: &PgPool,
    ) -> Result<Vec<Quality>, sqlx::Error> {
        let mut qualities_created = vec![];
        let mut tx = pool.begin().await?;
        for q in quality.iter() {
            let q_type = match q.quality_type {
                QualityType::Binary => QualityType::Binary,
                QualityType::Multiple => QualityType::Multiple,
                QualityType::Percent => QualityType::Percent,
            };

            let q_defined_by = match user.user.is_internal {
                Some(internal_user) => {
                    if internal_user {
                        QualityDefinedBy::System
                    } else {
                        QualityDefinedBy::User
                    }
                }
                None => QualityDefinedBy::User,
            };

            let qu = sqlx::query(
                "INSERT INTO QUALITY (
                    quality_long, 
                    quality_short, 
                    quality_type, 
                    sign_1_id,
                    sign_2_id,
                    defined_by,
                    created_by, 
                    updated_by) 
                values 
                    ($1, $2, $3, $4, $5, $6, $7, $8) 
                returning 
                    ID, 
                    QUALITY_LONG, 
                    QUALITY_SHORT, 
                    QUALITY_TYPE, 
                    SIGN_1_ID, 
                    SIGN_2_ID, 
                    DEFINED_BY",
            )
            .bind(&q.quality_long)
            .bind(&q.quality_short)
            .bind(q_type)
            .bind(q.sign_1_id)
            .bind(q.sign_2_id)
            .bind(q_defined_by)
            .bind(&user.user.id)
            .bind(&user.user.id)
            .map(|row: PgRow| {
                QualityCreatedOrModified {
                    id: row.get(0),
                    quality_long: row.get(1),
                    quality_short: row.get(2),
                    quality_type: row.get(3),
                    sign_1_id: row.get(4),
                    sign_2_id: row.get(5),
                    defined_by: row.get(6),
                    // created_at: row.get(4),
                    // updated_at: row.get(5),
                    // created_by: row.get(6),
                    // updated_by: row.get(7),
                }
            })
            .fetch_one(&mut tx)
            .await?;

            // debug!("This is qualities??? \n{}", q.choice.len());

            let mut choice_holder = vec![];

            for c in q.choice.iter() {
                let choice: Choice = sqlx::query(
                    "
                    INSERT INTO CHOICE (
                        quality_id, 
                        choice_value, 
                        created_by, 
                        updated_by)
                    values (
                        $1, $2, $3, $4)
                    returning 
                        id, 
                        choice_value",
                )
                .bind(qu.id)
                .bind(&c.choice_value)
                .bind(&user.user.id)
                .bind(&user.user.id)
                .map(|row: PgRow| Choice {
                    id: row.get(0),
                    choice_value: row.get(1),
                })
                .fetch_one(&mut tx)
                .await?;

                choice_holder.push(choice);
            }

            qualities_created.push(Quality {
                id: qu.id,
                quality_long: qu.quality_long,
                quality_short: qu.quality_short,
                quality_type: qu.quality_type,
                sign_1_id: qu.sign_1_id,
                sign_2_id: qu.sign_2_id,
                defined_by: qu.defined_by,
                // created_at: qu.created_at,
                // updated_at: qu.updated_at,
                // created_by: qu.created_by,
                // updated_by: qu.updated_by,
                choice: choice_holder,
            })
        }
        tx.commit().await?;

        Ok(qualities_created)
    }

    pub async fn update(
        user: AuthenticatedUser,
        id: i32,
        quality: QualityChoiceUpdateRequest,
        pool: &PgPool,
    ) -> Result<Quality> {
        // TODO:: HANDLE updating signs ONLY if there are no votes
        // let existing_quality = Quality::find_by_id(id, &pool).await;

        let mut tx = pool.begin().await?;

        let qu = sqlx::query(
            "UPDATE QUALITY 
                set 
                    quality_long = $1, 
                    quality_short = $2, 
                    sign_1_id = $3,
                    sign_2_id = $4,
                    updated_by= $5
                where 
                    id = $6
                    and created_by = $7
                    and defined_by = any(array['User']::quality_defined_by[]) -- ONLY update the ones created by users
                returning 
                    ID, 
                    QUALITY_LONG, 
                    QUALITY_SHORT, 
                    QUALITY_TYPE, 
                    SIGN_1_ID, 
                    SIGN_2_ID, 
                    DEFINED_BY",
        )
        .bind(&quality.quality_long)
        .bind(&quality.quality_short)
        .bind(&quality.sign_1_id)
        .bind(&quality.sign_2_id)
        .bind(&user.user.id)
        .bind(&id)
        .bind(&user.user.id)
        .map(|row: PgRow| {
            QualityCreatedOrModified {
                id: row.get(0),
                quality_long: row.get(1),
                quality_short: row.get(2),
                quality_type: row.get(3),
                sign_1_id: row.get(4),
                sign_2_id: row.get(5),
                defined_by: row.get(6),
                // created_at: row.get(4),
                // updated_at: row.get(5),
                // created_by: row.get(6),
                // updated_by: row.get(7),
            }
        })
        .fetch_one(&mut tx)
        .await?;

        // This is quality???!("This is qualities??? \n{}", q.choice.len());

        let mut choice_holder = vec![];

        // loop over Option of Vectors: https://stackoverflow.com/questions/40907897/iterator-on-optionvec
        for c in quality.choice.unwrap_or_else(Vec::new).iter() {
            // if c.id == None {
            //     return Err(anyhow!("Missing attribute: choice_id"));
            // }

            let choice: Choice = sqlx::query(
                "
                UPDATE CHOICE 
                    set choice_value = $1, 
                    updated_by = $2
                where 
                    quality_id = $3 
                    and id = $4 
                    and updated_by = $5 
                returning 
                    id, choice_value",
            )
            .bind(&c.choice_value)
            .bind(&user.user.id)
            .bind(qu.id)
            .bind(c.id)
            .bind(&user.user.id)
            .map(|row: PgRow| Choice {
                id: row.get(0),
                choice_value: row.get(1),
            })
            .fetch_one(&mut tx)
            .await?;

            choice_holder.push(choice);
        }
        tx.commit().await?;

        Ok(Quality {
            id: qu.id,
            quality_long: qu.quality_long,
            quality_short: qu.quality_short,
            quality_type: qu.quality_type,
            sign_1_id: qu.sign_1_id,
            sign_2_id: qu.sign_2_id,
            defined_by: qu.defined_by,
            // created_at: qu.created_at,
            // updated_at: qu.updated_at,
            // created_by: qu.created_by,
            // updated_by: qu.updated_by,
            choice: choice_holder,
        })
    }

    pub async fn delete(id: i32, pool: &PgPool) -> Result<()> {
        let mut tx = pool.begin().await?;
        sqlx::query("DELETE FROM quality WHERE id = $1")
            .bind(id)
            .execute(&mut tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }
}
