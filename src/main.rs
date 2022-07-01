use actix_web::middleware::Logger;
use actix_web::web;
use actix_web::web::Data;
use actix_web::App;
use actix_web::HttpServer;
use anyhow::Result;
use repositories::psql::user::UserRepoDb;
use services::user::delete_user;
use services::user::get_user_by_id;
use services::user::post_user;

const SERVER_URL: &str = "0.0.0.0:8000";
const DB_URL: &str = "postgres://postgres:postgres@127.0.0.1:5432/users_db";

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let user_repo = Data::new(UserRepoDb::init(DB_URL).await?);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(user_repo.clone())
            .route(
                "/users/{user_id}",
                web::get().to(get_user_by_id::<UserRepoDb>),
            )
            .route("/users", web::post().to(post_user::<UserRepoDb>))
            .route(
                "/users/{user_id}",
                web::delete().to(delete_user::<UserRepoDb>),
            )
    })
    .bind(SERVER_URL)?
    .run()
    .await?;
    Ok(())
}

pub mod models {
    pub mod user;
}
pub mod repositories {
    pub mod user;
    pub mod psql {
        pub mod user;
    }
}

pub mod services {
    pub mod user;
}

pub mod errors {
    pub mod user;
}

#[cfg(test)]
mod tests {
    pub mod services {
        pub mod user;
    }
    pub mod mock {
        pub mod user_repo;
    }
}
