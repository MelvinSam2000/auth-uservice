use std::sync::Arc;

use actix_web::http::StatusCode;
use actix_web::test;
use actix_web::web;
use actix_web::web::Data;
use actix_web::App;
use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use rstest::*;
use serde_json::Value;
use uuid::Uuid;

use crate::crypto::PasswordHasher;
use crate::models::user::User;
use crate::models::user::UserBuilder;
use crate::models::user::UserCreateReqDtoBuilder;
use crate::repositories::psql::user::UserRepoDb;
use crate::repositories::user::UserRepo;
use crate::services::user::delete_user;
use crate::services::user::get_user_by_id;
use crate::services::user::post_user;
use crate::tests::mock::user_repo::MockUserRepo;

#[rstest]
#[case::no_db(Arc::new(MockUserRepoNoDb))]
#[case::psql_db(Arc::new(MockUserRepoPsqlDb))]
#[actix_web::test]
async fn test_get_user_by_id(#[case] testable_repo: Arc<dyn InjectableMockUserRepo>) -> Result<()> {
    let (user_vec, user_repo) = testable_repo.init(0).await?;
    let user_repo = Data::from(user_repo);
    let app = test::init_service(
        App::new()
            .app_data(user_repo)
            .route("/users/{user_id}", web::get().to(get_user_by_id)),
    )
    .await;

    // Test successful requests from valid IDs
    for user in user_vec {
        let uri = format!("/users/{}", user.id.simple());
        let req = test::TestRequest::get().uri(&uri).to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "GET {} status code was not OK",
            &uri
        );
        let resp_json: Value = test::read_body_json(resp).await;
        assert_eq!(
            resp_json
                .get("username")
                .context("No username for payload")?
                .as_str()
                .context("Cant parse to str")?,
            user.username,
            "GET {} response is invalid",
            &uri
        );
    }

    // Test invalid request from invalid ID
    let req = test::TestRequest::get().uri("/users/invalid").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        StatusCode::BAD_REQUEST,
        "GET /users/invalid status code was not BAD REQUEST"
    );

    Ok(())
}

#[rstest]
#[case::no_db(Arc::new(MockUserRepoNoDb))]
#[case::psql_db(Arc::new(MockUserRepoPsqlDb))]
#[actix_web::test]
async fn test_post_user(#[case] testable_repo: Arc<dyn InjectableMockUserRepo>) -> Result<()> {
    let (_, user_repo) = testable_repo.init(1).await?;
    let user_repo = Data::from(user_repo);
    let pwd_hasher = Data::new(PasswordHasher::default());
    let app = test::init_service(
        App::new()
            .app_data(user_repo.clone())
            .app_data(pwd_hasher.clone())
            .route("/users", web::post().to(post_user)),
    )
    .await;

    // Test a valid user creation
    let password_raw = "abc12345";
    let new_user = UserCreateReqDtoBuilder::default()
        .username("Derek")
        .password_raw(password_raw)
        .build()?;

    let req = test::TestRequest::post()
        .uri("/users")
        .set_json(new_user.clone())
        .to_request();
    let resp = test::call_service(&app, req).await;
    let resp_status = resp.status();
    let resp_json: Value = test::read_body_json(resp).await;
    assert_eq!(
        resp_status,
        StatusCode::OK,
        "POST /users status code was not OK. Response: {}",
        resp_json
    );
    let user_id = Uuid::try_parse(
        resp_json
            .as_str()
            .context("Response JSON cant be converted to UUID")?,
    )?;

    assert!(
        user_repo.get_user_by_id(&user_id).await.is_ok(),
        "UserRepo does not contain newly created user"
    );

    // Test validation failure
    let new_user = UserCreateReqDtoBuilder::default()
        .username("Eric")
        .password_raw("p")
        .build()?;
    let req = test::TestRequest::post()
        .uri("/users")
        .set_json(new_user)
        .to_request();
    let resp = test::call_service(&app, req).await;
    let resp_status = resp.status();
    let resp_json: Value = test::read_body_json(resp).await;
    assert_eq!(
        resp_status,
        StatusCode::BAD_REQUEST,
        "POST /users for validation error status code was not BAD REQUEST. Response: {}",
        resp_json
    );

    // Test failure on repeated username
    let new_user = UserCreateReqDtoBuilder::default()
        .username("Derek")
        .password_raw("p")
        .build()?;
    let req = test::TestRequest::post()
        .uri("/users")
        .set_json(new_user)
        .to_request();
    let resp = test::call_service(&app, req).await;
    let resp_status = resp.status();
    let resp_json: Value = test::read_body_json(resp).await;
    assert_eq!(
        resp_status,
        StatusCode::BAD_REQUEST,
        "POST /users for repeated username status code was not BAD REQUEST. Response: {}",
        resp_json
    );

    // Test password hash is valid
    let password_hash = &user_repo.get_password_by_id(&user_id).await?;
    assert!(
        pwd_hasher.verify_password(password_raw, password_hash)?,
        "Password validation failed..."
    );

    Ok(())
}

#[rstest]
#[case::no_db(Arc::new(MockUserRepoNoDb))]
#[case::psql_db(Arc::new(MockUserRepoPsqlDb))]
#[actix_web::test]
async fn test_delete_user(#[case] testable_repo: Arc<dyn InjectableMockUserRepo>) -> Result<()> {
    let (user_vec, user_repo) = testable_repo.init(2).await?;
    let user_repo = Data::from(user_repo);
    let app = test::init_service(
        App::new()
            .app_data(user_repo.clone())
            .route("/users/{user_id}", web::delete().to(delete_user)),
    )
    .await;

    // Test valid user deletion
    let id = user_vec[0].id;
    let uri = &format!("/users/{}", id.simple());
    let req = test::TestRequest::delete().uri(uri).to_request();
    let resp = test::call_service(&app, req).await;
    let resp_status = resp.status();
    assert_eq!(
        resp_status,
        StatusCode::OK,
        "DELETE {} status code was not OK",
        &uri,
    );
    assert!(user_repo.get_user_by_id(&id).await.is_err());

    // Test invalid request from invalid ID
    let req = test::TestRequest::delete()
        .uri("/users/invalid")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        StatusCode::BAD_REQUEST,
        "DELETE /users/invalid status code was not BAD REQUEST"
    );

    Ok(())
}

#[async_trait]
trait InjectableMockUserRepo {
    async fn init(&self, test_id: u8) -> Result<(Vec<User>, Arc<dyn UserRepo>)>;
}

struct MockUserRepoNoDb;

#[async_trait]
impl InjectableMockUserRepo for MockUserRepoNoDb {
    async fn init(&self, _test_id: u8) -> Result<(Vec<User>, Arc<dyn UserRepo>)> {
        let user_vec = vec![
            UserBuilder::default()
                .id(Uuid::new_v4())
                .username("Alice")
                .password_hash("phash1234")
                .build()?,
            UserBuilder::default()
                .id(Uuid::new_v4())
                .username("Bob")
                .password_hash("hunter2")
                .email("bobmaster@email.com")
                .build()?,
            UserBuilder::default()
                .id(Uuid::new_v4())
                .username("Carl")
                .password_hash("blahblah")
                .email("carl@email.com")
                .created_at(Utc::now())
                .last_login(Utc::now())
                .build()?,
        ];

        let user_repo = Arc::new(MockUserRepo::from(user_vec.clone()));

        Ok((user_vec, user_repo))
    }
}

struct MockUserRepoPsqlDb;

#[async_trait]
impl InjectableMockUserRepo for MockUserRepoPsqlDb {
    async fn init(&self, test_id: u8) -> Result<(Vec<User>, Arc<dyn UserRepo>)> {
        const TEST_DB_URL: &str = "postgres://postgres:postgres@127.0.0.1:5432/test_users_db";
        let user_repo = UserRepoDb::init(&format!("{}_{}", TEST_DB_URL, test_id)).await?;
        let user_vec = vec![
            UserBuilder::default()
                .id(Uuid::new_v4())
                .username("Alice")
                .password_hash("phash1234")
                .build()?,
            UserBuilder::default()
                .id(Uuid::new_v4())
                .username("Bob")
                .password_hash("hunter2")
                .email("bobmaster@email.com")
                .build()?,
            UserBuilder::default()
                .id(Uuid::new_v4())
                .username("Carl")
                .password_hash("blahblah")
                .email("carl@email.com")
                .created_at(Utc::now())
                .last_login(Utc::now())
                .build()?,
        ];
        user_repo.drop_table().await?;
        user_repo.create_table().await?;
        for user in user_vec.iter() {
            user_repo.create_user(user).await?;
        }

        let user_repo = Arc::new(user_repo);
        Ok((user_vec, user_repo))
    }
}
