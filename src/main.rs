#[macro_use]
extern crate log;
// extern crate google_signin;

// use actix_redis::RedisSession;
use actix_redis::RedisActor;

use actix_cors::Cors;
use actix_http::http::ContentEncoding;
use actix_web::{
    http::header, middleware::Compress, middleware::Logger, web, App, HttpResponse, HttpServer,
    Responder,
};
use anyhow::Result;
use dotenv::dotenv;
use listenfd::ListenFd;
use sqlx::PgPool;
use std::{collections::HashMap, env};

use google_jwt_verify::AsyncClient as GoogleAsyncClient;

// import todo module (routes and model)
mod auth;
mod errors;
mod quality;
mod redis;
mod signs;
mod todo;
mod user;
mod votes;

// default / handler
async fn index() -> impl Responder {
    HttpResponse::Ok().body(r#"
        Welcome to Actix-web with SQLx Todos example.
        Available routes:
        GET /todos -> list of all todos
        POST /todo -> create new todo, example: { "description": "learn actix and sqlx", "done": false }
        GET /todo/{id} -> show one todo with requested id
        PUT /todo/{id} -> update todo with requested id, example: { "description": "learn actix and sqlx", "done": true }
        DELETE /todo/{id} -> delete todo with requested id
    "#
    )
}

// https://auth0.com/blog/build-an-api-in-rust-with-jwt-authentication-using-actix-web/
/*async fn validator(&mut req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, Error> {
    let mut client = google_signin::Client::new();

    let client_id = env::var("CLIENT_ID").expect("CLIENT_ID is not set in .env file");

    client.audiences.push(client_id); // required
    // client.hosted_domains.push(YOUR_HOSTED_DOMAIN); // optional


    let config = req
        .app_data::<Config>()
        .map(|data| data.clone())
        .unwrap_or_else(Default::default);

    debug!("CREDENTIALS: {:?} \n {:?}", credentials, req);

    match client.verify(&credentials.token()) {
        Ok(res) => {

            // let res1 = res.clone
            // let asd = Payload::Stream(format!("res: {:?}", res).as_bytes());

            // let asd1 = Payload::Stream(serde_json::to_string(&res));
            // let w = req.into_parts();


            let mut payload = actix_http::h1::Payload::empty();



            let bytes =serde_json::to_string(&SomeStruct{ test: 12, test1: 23}).expect("Failed to serialize test data to json");


            payload.unread_data(bytes.into());
            req.set_payload(payload.into());
            Ok(req)
        },
        Err(err) => {
            debug!("Erro: {:?}", err);
            Err(AuthenticationError::from(config).into())
        },
    }
}
*/

#[derive(Clone)]
pub struct InternalAppData {
    google_client: GoogleAsyncClient,
    facebook_data: HashMap<String, String>,
    sysadmin: String,
    dummy_user: String,
}

// impl fmt::Display for InternalAppData {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         // Customize so only `x` and `y` are denoted.
//         write!(
//             f,
//             "google_client_id: {:?}, external_idp: {}, facebook_data: {:?}",
//             self.google_client, self.sysadmin, self.facebook_data
//         )
//     }
// }

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    // this will enable us to keep application running during recompile: systemfd --no-pid -s http::5000 -- cargo watch -x run
    let mut listenfd = ListenFd::from_env();

    let google_client_id =
        env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID is not set in .env file");
    let sysadmin = env::var("SYSADMIN").expect("SYSADMIN is not set in .env file");
    let dummy_user = env::var("DUMMY_USER").expect("DUMMY_USER is not set in .env file");
    let facebook_app_id =
        env::var("FACEBOOK_APP_ID").expect("FACEBOOK_APP_ID is not set in .env file");
    let facebook_secret =
        env::var("FACEBOOK_SECRET").expect("FACEBOOK_SECRET is not set in .env file");
    let facebook_access_token =
        env::var("FACEBOOK_ACCESS_TOKEN").expect("FACEBOOK_ACCESS_TOKEN is not set in .env file");

    let mut data = HashMap::new();

    data.insert("facebook_app_id".to_string(), facebook_app_id);
    data.insert("facebook_secret".to_string(), facebook_secret);
    data.insert("facebook_access_token".to_string(), facebook_access_token);

    let g_client = google_jwt_verify::AsyncClient::new(&google_client_id);

    let internal_app_data = InternalAppData {
        google_client: g_client,
        sysadmin,
        dummy_user,
        facebook_data: data,
    };

    // POSTGRES
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let db_pool = PgPool::connect(&database_url).await?;

    // REDIS
    let redis_host = env::var("REDIS_HOST").expect("HOST is not set in .env file");
    let redis_port = env::var("REDIS_PORT").expect("PORT is not set in .env file");

    let redis_addr = RedisActor::start(format!("{}:{}", redis_host, redis_port));

    let mut server = HttpServer::new(move || {
        // let auth = HttpAuthentication::bearer(validator);

        //TODO:: Add rate limiting
        App::new()
            // .wrap(auth)
            .wrap(Compress::new(ContentEncoding::Gzip))
            // .wrap(RedisSession::new(
            //     format!("{}:{}", redis_host, redis_port),
            //     &[0; 32],
            // ))
            .wrap(Logger::default())
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:3000")
                    .allowed_methods(vec!["GET", "POST", "DELETE", "PUT"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .supports_credentials()
                    .max_age(3600),
            )
            .data(db_pool.clone())
            .data(redis_addr.clone())
            .data(internal_app_data.clone()) // pass database pool to application so we can access it inside handlers
            .route("/", web::get().to(index))
            .configure(quality::init)
            .configure(signs::init)
            .configure(todo::init)
            .configure(user::init)
            .configure(votes::init)
    });

    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => {
            let host = env::var("HOST").expect("HOST is not set in .env file");
            let port = env::var("PORT").expect("PORT is not set in .env file");
            server.bind(format!("{}:{}", host, port))?
        }
    };

    info!("Starting server");
    server.run().await?;

    Ok(())
}
