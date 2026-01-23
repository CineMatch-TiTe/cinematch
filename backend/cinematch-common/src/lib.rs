use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub mod vote_store;

/// Standard error response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
        }
    }
}

#[macro_export]
macro_rules! extract_user_id {
    ($identity:expr) => {{
        use ::actix_web::HttpResponse;
        use ::log::{error, trace};
        use ::std::convert::Into;
        use ::uuid::Uuid;

        // Convert to Option<&Identity> to handle both cases
        let identity_opt: Option<::actix_identity::Identity> = $identity.into();

        match identity_opt {
            Some(identity) => match identity.id() {
                Ok(id_str) => match Uuid::parse_str(&id_str) {
                    Ok(uuid) => uuid,
                    Err(_) => {
                        error!("Invalid user ID in identity: {}", &id_str);
                        return HttpResponse::InternalServerError()
                            .json(cinematch_common::ErrorResponse::new("Invalid user ID"));
                    }
                },
                Err(_) => {
                    trace!("No user ID found in identity");
                    return HttpResponse::Unauthorized()
                        .json(cinematch_common::ErrorResponse::new("No user ID found"));
                }
            },
            None => {
                trace!("No identity provided");
                return HttpResponse::Unauthorized()
                    .json(cinematch_common::ErrorResponse::new("No user ID found"));
            }
        }
    }};
}
