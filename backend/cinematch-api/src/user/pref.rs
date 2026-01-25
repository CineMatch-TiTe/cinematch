use serde::Deserialize;

use actix_identity::Identity;
use actix_web::{HttpResponse, get, patch, web};
use log::error;
use uuid::Uuid;

use crate::AppState;
use cinematch_common::{ErrorResponse, FullUserPreferences, extract_user_id};
use cinematch_db::{DbError, UpdateUserPreferences, UserPreferences};

/// Get the current user's preferences
use super::{UpdateUserPreferencesRequest, UserPreferencesResponse};

#[utoipa::path(
	responses(
		(status = 200, description = "Get user preferences", body = UserPreferencesResponse),
		(status = 401, description = "Unauthorized", body = ErrorResponse),
		(status = 404, description = "Preferences not found", body = ErrorResponse),
		(status = 500, description = "Internal server error", body = ErrorResponse)
	),
	tags = ["user"],
	operation_id = "get_user_preferences"
)]
#[get("/pref")]
pub async fn get_user_pref(db: AppState, user: Option<Identity>) -> HttpResponse {
    let user_id = extract_user_id!(user);
    
    let prefs_result: Result<FullUserPreferences, DbError> = db.get_user_preferences(user_id).await;

    let genre_map = match db.get_genres().await {
        Ok(map) => map,
        Err(e) => {
            error!("Failed to fetch genres: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to fetch genres"));
        }
    };

    match prefs_result {
        Ok(prefs) => HttpResponse::Ok().json(UserPreferencesResponse {
            target_release_year: prefs.preferred_year,
            release_year_flex: prefs.year_flexibility,
            include_genres: prefs.included_genres.iter().filter_map(|gid| genre_map.iter().find_map(|(name, &id)| if id == *gid { Some(name.clone()) } else { None })).collect(),
            exclude_genres: prefs.excluded_genres.iter().filter_map(|gid| genre_map.iter().find_map(|(name, &id)| if id == *gid { Some(name.clone()) } else { None })).collect(),
            updated_at: prefs.updated_at,
            created_at: prefs.created_at,
        }),
        Err(e) => {
            error!("Failed to get user preferences for {}: {}", user_id, e);
            HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to retrieve user preferences"))
        }
    }

}
/// Edit the current user's preferences
#[utoipa::path(
	request_body = UpdateUserPreferencesRequest,
	responses(
		(status = 200, description = "Preferences updated", body = UserPreferencesResponse),
		(status = 400, description = "Invalid input", body = ErrorResponse),
		(status = 401, description = "Unauthorized", body = ErrorResponse),
		(status = 404, description = "Preferences not found", body = ErrorResponse),
		(status = 500, description = "Internal server error", body = ErrorResponse)
	),
	tags = ["user"],
	operation_id = "edit_user_preferences"
)]
#[patch("/pref")]
pub async fn edit_user_pref(
    db: AppState,
    user: Option<Identity>,
    update: web::Json<UpdateUserPreferencesRequest>,
) -> HttpResponse {
    let user_id = extract_user_id!(user);
    let genre_map = match db.get_genres().await {
        Ok(map) => map,
        Err(e) => {
            error!("Failed to fetch genres: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to fetch genres"));
        }
    };
    // Update year/flex
    let update_internal = UpdateUserPreferences {
        target_release_year: update.target_release_year,
        release_year_flex: update.release_year_flex,
    };
    if let Err(e) = db.update_user_preferences(user_id, update_internal).await {
        error!("Failed to update user preferences: {}", e);
        return HttpResponse::InternalServerError()
            .json(ErrorResponse::new("Failed to update preferences"));
    }
    // Update included genres
    if let Some(ref names) = update.include_genres {
        // Remove all current includes
        if let Ok(current) = db.get_user_include_genres(user_id).await {
            for gid in current {
                let _ = db.remove_user_include_genre(user_id, gid).await;
            }
        }
        // Add new includes
        for name in names {
            if let Some(&gid) = genre_map.get(name) {
                let _ = db.add_user_include_genre(user_id, gid).await;
            } else {
                return HttpResponse::BadRequest()
                    .json(ErrorResponse::new("Invalid genre name in include_genres"));
            }
        }
    }
    // Update excluded genres
    if let Some(ref names) = update.exclude_genres {
        // Remove all current excludes
        if let Ok(current) = db.get_user_exclude_genres(user_id).await {
            for gid in current {
                let _ = db.remove_user_exclude_genre(user_id, gid).await;
            }
        }
        // Add new excludes
        for name in names {
            if let Some(&gid) = genre_map.get(name) {
                let _ = db.add_user_exclude_genre(user_id, gid).await;
            } else {
                return HttpResponse::BadRequest()
                    .json(ErrorResponse::new("Invalid genre name in exclude_genres"));
            }
        }
    }

    let prefs_result: Result<FullUserPreferences, DbError> = db.get_user_preferences(user_id).await;

    let genre_map = match db.get_genres().await {
        Ok(map) => map,
        Err(e) => {
            error!("Failed to fetch genres: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to fetch genres"));
        }
    };

    match prefs_result {
        Ok(prefs) => HttpResponse::Ok().json(UserPreferencesResponse {
            target_release_year: prefs.preferred_year,
            release_year_flex: prefs.year_flexibility,
            include_genres: prefs.included_genres.iter().filter_map(|gid| genre_map.iter().find_map(|(name, &id)| if id == *gid { Some(name.clone()) } else { None })).collect(),
            exclude_genres: prefs.excluded_genres.iter().filter_map(|gid| genre_map.iter().find_map(|(name, &id)| if id == *gid { Some(name.clone()) } else { None })).collect(),
            updated_at: prefs.updated_at,
            created_at: prefs.created_at,
        }),
        Err(e) => {
            error!("Failed to get user preferences for {}: {}", user_id, e);
            HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to retrieve user preferences"))
        }
    }
}
