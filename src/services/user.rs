use actix_web::web::Data;
use actix_web::web::Json;
use actix_web::web::Path;
use actix_web::HttpResponse;
use uuid::Uuid;

use crate::errors::user::UserServiceError;
use crate::errors::user::UserServiceResult;
use crate::models::user::User;
use crate::models::user::UserCreateReqDto;
use crate::repositories::user::UserRepo;

pub async fn get_user_by_id<T>(user_repo: Data<T>, user_id: Path<String>) -> UserServiceResult
where
    T: UserRepo,
{
    let user_id_str = user_id.into_inner();
    let user_id = Uuid::try_parse(&user_id_str)
        .map_err(|_| UserServiceError::InvalidId(user_id_str.clone()))?;
    let user = user_repo
        .get_user_by_id(&user_id)
        .await
        .map_err(|_| UserServiceError::InvalidId(user_id_str))?;
    Ok(HttpResponse::Ok().json(user))
}

pub async fn post_user<T>(user_repo: Data<T>, user: Json<UserCreateReqDto>) -> UserServiceResult
where
    T: UserRepo,
{
    let user = User::from(user.0);
    let user_id = user_repo
        .create_user(&user)
        .await
        .map_err(|_| UserServiceError::UnknownInternal)?;
    Ok(HttpResponse::Ok().json(user_id))
}

pub async fn delete_user<T>(user_repo: Data<T>, user_id: Path<String>) -> UserServiceResult
where
    T: UserRepo,
{
    let user_id_str = user_id.into_inner();
    let user_id = Uuid::try_parse(&user_id_str)
        .map_err(|_| UserServiceError::InvalidId(user_id_str.clone()))?;
    user_repo
        .delete_user_by_id(&user_id)
        .await
        .map_err(|_| UserServiceError::InvalidId(user_id_str))?;

    Ok(HttpResponse::Ok().into())
}
