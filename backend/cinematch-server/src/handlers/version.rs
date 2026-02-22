use actix_web::{HttpResponse, get};
use serde::Serialize;

#[derive(Serialize, utoipa::ToSchema)]
pub struct VersionInfo {
    pub version: &'static str,
    pub git_hash: &'static str,
}

/// Get application version and git commit hash.
#[utoipa::path(
    responses(
        (status = 200, description = "Version information", body = VersionInfo)
    ),
    tags = ["System"],
    operation_id = "get_version"
)]
#[get("/version")]
pub async fn get_version() -> HttpResponse {
    HttpResponse::Ok().json(VersionInfo {
        version: env!("CARGO_PKG_VERSION"),
        git_hash: env!("GIT_HASH"),
    })
}
