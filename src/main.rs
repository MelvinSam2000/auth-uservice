use actix_web::middleware::Logger;
use actix_web::App;
use actix_web::HttpServer;
use anyhow::Result;

const SERVER_URL: &str = "0.0.0.0:8000";

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    HttpServer::new(move || {
        App::new().wrap(Logger::default())
        //.route("/users/{user_id}", web::get().to(get_user_by_id))
        //.route("/users", web::post().to(post_user))
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
