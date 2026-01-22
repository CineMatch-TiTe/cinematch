use actix_jwt_auth_middleware::FromRequest;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Clone, FromRequest, ToSchema)]
pub struct UserClaims {
    pub user_id: uuid::Uuid,
}