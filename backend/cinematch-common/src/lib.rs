pub const NAME_MIN_LENGTH: usize = 3;
pub const NAME_MAX_LENGTH: usize = 32;

#[macro_export]
macro_rules! extract_and_validate_username {
    ($input:expr) => {{
        let name = $input
            .trim()
            .chars()
            .filter(|c| !c.is_control())
            .collect::<String>();
        if name.len() < cinematch_common::NAME_MIN_LENGTH
            || name.len() > cinematch_common::NAME_MAX_LENGTH
        {
            use ::actix_web::HttpResponse;
            return HttpResponse::BadRequest().json($crate::models::ErrorResponse::new(format!(
                "Username must be between {} and {} characters",
                cinematch_common::NAME_MIN_LENGTH,
                cinematch_common::NAME_MAX_LENGTH
            )));
        }
        name
    }};
}

pub mod models;

// Re-export models for easier access
pub use models::*;

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
                        return HttpResponse::InternalServerError().json(
                            cinematch_common::models::ErrorResponse::new("Invalid user ID"),
                        );
                    }
                },
                Err(_) => {
                    trace!("No user ID found in identity");
                    return HttpResponse::Unauthorized().json(
                        cinematch_common::models::ErrorResponse::new("No user ID found"),
                    );
                }
            },
            None => {
                trace!("No identity provided");
                return HttpResponse::Unauthorized().json(
                    cinematch_common::models::ErrorResponse::new("No user ID found"),
                );
            }
        }
    }};
}
