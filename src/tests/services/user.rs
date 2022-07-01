use std::collections::HashMap;

use actix_web::http::StatusCode;
use actix_web::test;
use actix_web::web;
use actix_web::web::Data;
use actix_web::App;
use anyhow::Context;
use anyhow::Result;
use itertools::assert_equal;
use serde_json::json;
use serde_json::Value;
use uuid::Uuid;

use crate::models::user::User;
use crate::models::user::UserBuilder;
use crate::models::user::UserCreateReqDtoBuilder;
use crate::repositories::user::UserRepo;
use crate::services::user::delete_user;
use crate::services::user::get_user_by_id;
use crate::services::user::post_user;
use crate::tests::mock::user_repo::MockUserRepo;

#[actix_web::test]
async fn test_get_user_by_id() -> Result<()> {
    let (user_map, user_repo) = mock_user_repo().await?;
    let app = test::init_service(App::new().app_data(user_repo).route(
        "/users/{user_id}",
        web::get().to(get_user_by_id::<MockUserRepo>),
    ))
    .await;

    // Test successful requests from valid IDs
    for id in user_map.keys() {
        let uri = format!("/users/{}", id.simple());
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
            resp_json,
            json!(user_map.get(&id).context("No user with given ID")?),
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

#[actix_web::test]
async fn test_post_user() -> Result<()> {
    let (mut user_map, user_repo) = mock_user_repo().await?;
    let app = test::init_service(
        App::new()
            .app_data(user_repo.clone())
            .route("/users", web::post().to(post_user::<MockUserRepo>)),
    )
    .await;

    // Test a valid user creation
    let new_user = UserCreateReqDtoBuilder::default()
        .username("Derek")
        .password_raw("abc123")
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
    let mut new_user = User::from(new_user);
    new_user.id = Some(user_id);
    user_map.insert(user_id, new_user);

    assert_equal(user_map.keys(), user_repo.0.lock().await.keys());
    assert_equal(user_map.values(), user_repo.0.lock().await.values());

    Ok(())
}

#[actix_web::test]
async fn test_delete_user() -> Result<()> {
    let (mut user_map, user_repo) = mock_user_repo().await?;
    let app = test::init_service(App::new().app_data(user_repo.clone()).route(
        "/users/{user_id}",
        web::delete().to(delete_user::<MockUserRepo>),
    ))
    .await;

    // Test valid user deletion
    let id = user_map
        .keys()
        .next()
        .cloned()
        .context("No more users left...")?;
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
    user_map.remove(&id).context("No user with given ID")?;

    assert_equal(user_map.keys(), user_repo.0.lock().await.keys());
    assert_equal(user_map.values(), user_repo.0.lock().await.values());

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

async fn mock_user_repo() -> Result<(HashMap<Uuid, User>, Data<MockUserRepo>)> {
    let ids = [Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];

    let user_map: HashMap<Uuid, User> = HashMap::from([
        (
            ids[0],
            UserBuilder::default()
                .id(ids[0])
                .username("Alice")
                .password_hash("phash1234")
                .password_salt("psalt1234")
                .build()?,
        ),
        (
            ids[1],
            UserBuilder::default()
                .id(ids[1])
                .username("Bob")
                .password_hash("hunter2")
                .password_salt("salt")
                .email("bobmaster@email.com")
                .build()?,
        ),
        (
            ids[2],
            UserBuilder::default()
                .id(ids[2])
                .username("Carl")
                .password_hash("blahblah")
                .password_salt("foobar")
                .email("carl@email.com")
                .created_at("2012")
                .last_login("2013")
                .build()?,
        ),
    ]);

    let user_repo = MockUserRepo::from(user_map.clone());
    let user_repo: Data<MockUserRepo> = Data::new(user_repo);

    Ok((user_map, user_repo))
}
