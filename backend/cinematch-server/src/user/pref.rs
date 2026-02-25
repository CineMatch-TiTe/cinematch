use crate::auth::guard::Auth;
use actix_web::{get, patch, web};
use log::error;
use uuid::Uuid;

use crate::AppState;
use crate::api_error::ApiError;
use cinematch_common::models::ErrorResponse;
use cinematch_db::UpdateUserPreferences;
use cinematch_db::domain::{Movie, User};

use super::{UpdateUserPreferencesRequest, UserPreferencesResponse};

/// Current user's preferences (genres, year, etc.).
#[utoipa::path(
    responses(
        (status = 200, description = "Preferences", body = UserPreferencesResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tags = ["User"],
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    operation_id = "get_user_preferences"
)]
#[get("/preferences")]
pub async fn get_user_pref(
    ctx: AppState,
    auth: Option<Auth>,
) -> Result<web::Json<UserPreferencesResponse>, ApiError> {
    let user_id = auth
        .ok_or_else(|| ApiError::Unauthorized("No identity provided".to_string()))?
        .user_id();
    let user_obj = User::from_id(&ctx, user_id).await?;
    let prefs_handle = user_obj.preferences(&ctx).await?;
    let prefs_record = prefs_handle.record(&ctx).await?;

    let genre_map = match Movie::all_genres(&ctx).await {
        Ok(map) => map,
        Err(e) => {
            error!("Failed to fetch genres: {}", e);
            return Err(ApiError::InternalServerError(
                "Failed to fetch genres".to_string(),
            ));
        }
    };

    Ok(web::Json(UserPreferencesResponse {
        target_release_year: prefs_record.preferred_year,
        release_year_flex: prefs_record.year_flexibility,
        include_genres: prefs_record
            .included_genres
            .iter()
            .filter_map(|gid| {
                genre_map.iter().find_map(
                    |(name, &id): (&String, &Uuid)| {
                        if id == *gid { Some(name.clone()) } else { None }
                    },
                )
            })
            .collect(),
        exclude_genres: prefs_record
            .excluded_genres
            .iter()
            .filter_map(|gid| {
                genre_map.iter().find_map(
                    |(name, &id): (&String, &Uuid)| {
                        if id == *gid { Some(name.clone()) } else { None }
                    },
                )
            })
            .collect(),
        updated_at: prefs_record.updated_at,
        created_at: prefs_record.created_at,
    }))
}
/// Update current user's preferences. Invalid genre names return 400.
#[utoipa::path(
    request_body = UpdateUserPreferencesRequest,
    responses(
        (status = 200, description = "Updated preferences", body = UserPreferencesResponse),
        (status = 400, description = "Invalid genre name in include/exclude", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tags = ["User"],
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    operation_id = "edit_user_preferences"
)]
#[patch("/preferences")]
pub async fn edit_user_pref(
    ctx: AppState,
    auth: Option<Auth>,
    update: web::Json<UpdateUserPreferencesRequest>,
) -> Result<web::Json<UserPreferencesResponse>, ApiError> {
    let user_id = auth
        .ok_or_else(|| ApiError::Unauthorized("No identity provided".to_string()))?
        .user_id();
    let user_obj = User::from_id(&ctx, user_id).await?;
    let prefs_handle = user_obj.preferences(&ctx).await?;

    let genre_map = match Movie::all_genres(&ctx).await {
        Ok(map) => map,
        Err(e) => {
            error!("Failed to fetch genres: {}", e);
            return Err(ApiError::InternalServerError(
                "Failed to fetch genres".to_string(),
            ));
        }
    };
    // Update year/flex
    let update_internal = UpdateUserPreferences {
        target_release_year: update.target_release_year,
        release_year_flex: update.release_year_flex,
        is_tite: update.is_tite,
    };
    if let Err(e) = user_obj.update_preferences(&ctx, update_internal).await {
        error!("Failed to update user preferences: {}", e);
        return Err(ApiError::InternalServerError(
            "Failed to update preferences".to_string(),
        ));
    }
    // Update included genres
    if let Some(ref names) = update.include_genres {
        // Remove all current includes
        if let Ok(current) = user_obj.included_genres(&ctx).await {
            for gid in current {
                let _ = user_obj.remove_included_genre(&ctx, gid).await;
            }
        }
        // Add new includes
        for name in names {
            if let Some(&gid) = genre_map.get(name) {
                let _ = user_obj.add_included_genre(&ctx, gid).await;
            } else {
                return Err(ApiError::BadRequest(
                    "Invalid genre name in include_genres".to_string(),
                ));
            }
        }
    }
    // Update excluded genres
    if let Some(ref names) = update.exclude_genres {
        // Remove all current excludes
        if let Ok(current) = user_obj.excluded_genres(&ctx).await {
            for gid in current {
                let _ = user_obj.remove_excluded_genre(&ctx, gid).await;
            }
        }
        // Add new excludes
        for name in names {
            if let Some(&gid) = genre_map.get(name) {
                let _ = user_obj.add_excluded_genre(&ctx, gid).await;
            } else {
                return Err(ApiError::BadRequest(
                    "Invalid genre name in exclude_genres".to_string(),
                ));
            }
        }
    }

    let prefs = prefs_handle.record(&ctx).await?;

    Ok(web::Json(UserPreferencesResponse {
        target_release_year: prefs.preferred_year,
        release_year_flex: prefs.year_flexibility,
        include_genres: prefs
            .included_genres
            .iter()
            .filter_map(|gid| {
                genre_map.iter().find_map(
                    |(name, &id): (&String, &Uuid)| {
                        if id == *gid { Some(name.clone()) } else { None }
                    },
                )
            })
            .collect(),
        exclude_genres: prefs
            .excluded_genres
            .iter()
            .filter_map(|gid| {
                genre_map.iter().find_map(
                    |(name, &id): (&String, &Uuid)| {
                        if id == *gid { Some(name.clone()) } else { None }
                    },
                )
            })
            .collect(),
        updated_at: prefs.updated_at,
        created_at: prefs.created_at,
    }))
}
