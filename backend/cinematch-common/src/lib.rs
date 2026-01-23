use actix_jwt_auth_middleware::FromRequest;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Clone, FromRequest, ToSchema)]
pub struct UserClaims {
    pub user_id: uuid::Uuid,
}

#[macro_export]
macro_rules! extract_user_id {
    // Pattern for Option<Identity> - returns 401 if None
    (option $identity:expr) => {{
        if let Some(identity) = $identity {
            match identity.id() {
                Ok(id_str) => match Uuid::parse_str(&id_str) {
                    Ok(uuid) => uuid,
                    Err(_) => {
                        error!("Invalid user ID in identity: {}", &id_str);
                        return HttpResponse::InternalServerError().json(ErrorResponse::new("Invalid user ID"));
                    }
                },
                Err(_) => {
                    trace!("No user ID found in identity");
                    return HttpResponse::Unauthorized().json(ErrorResponse::new("No user ID found"));
                }
            }
        } else {
            trace!("No identity provided");
            return HttpResponse::Unauthorized().json(ErrorResponse::new("No user ID found"));
        }
    }};
    // Pattern for Identity
    ($identity:expr) => {{
        match $identity.id() {
            Ok(id_str) => match Uuid::parse_str(&id_str) {
                Ok(uuid) => uuid,
                Err(_) => {
                    error!("Invalid user ID in identity: {}", &id_str);
                    return HttpResponse::InternalServerError().json(ErrorResponse::new("Invalid user ID"));
                }
            },
            Err(_) => {
                trace!("No user ID found in identity");
                return HttpResponse::Unauthorized().json(ErrorResponse::new("No user ID found"));
            }
        }
    }};
}