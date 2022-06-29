use actix_web::get;
use actix_web::middleware::Logger;
use actix_web::App;
use actix_web::HttpServer;
use anyhow::Result;

const SERVER_URL: &str = "0.0.0.0:8000";

#[get("/hello")]
async fn get_hello() -> String {
    "Hello World!".to_string()
}

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    HttpServer::new(move || App::new().wrap(Logger::default()).service(get_hello))
        .bind(SERVER_URL)?
        .run()
        .await?;
    Ok(())
}

#[actix_web::test]
async fn test_hello() -> Result<()> {
    use actix_web::body::MessageBody;
    use actix_web::test;
    use anyhow::anyhow;

    let app = test::init_service(App::new().service(get_hello)).await;
    let req = test::TestRequest::get().uri("/hello").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "GET /hello was not 200");
    assert_eq!(
        resp.into_body()
            .try_into_bytes()
            .map_err(|_| anyhow!("Could not convert response into bytes..."))?,
        "Hello World!".to_string(),
        "GET /hello response was not as expected."
    );
    Ok(())
}
