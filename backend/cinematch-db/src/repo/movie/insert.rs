#![allow(clippy::collapsible_if)]
use std::collections::{HashMap, HashSet};

use crate::conn::qdrant::models::MovieData;
use crate::schema::{
    cast_members, directors, genres, keywords, movie_cast, movie_directors, movie_genres,
    movie_keywords, movie_production_countries, movie_trailers, movies, production_countries,
    trailers,
};
use diesel::result::Error as DieselError;
use log::error;

use crate::Database;
use crate::DbError;

#[derive(thiserror::Error, Debug)]
pub enum MovieCrudError {
    #[error("Qdrant point ID must be set before inserting movie")]
    MissingQdrantPointId,
    #[error(transparent)]
    Diesel(#[from] DieselError),
    #[error(transparent)]
    Db(#[from] DbError),
}

pub type MovieCrudResult<T> = Result<T, MovieCrudError>;

fn unix_to_naive_datetime(secs: i64) -> Result<chrono::NaiveDateTime, MovieCrudError> {
    chrono::DateTime::from_timestamp(secs, 0)
        .map(|dt| dt.naive_utc())
        .ok_or(MovieCrudError::Diesel(DieselError::NotFound))
}

impl Database {
    /// True batch insert: inserts all movies and their related data using
    /// multi-row INSERT statements instead of one-at-a-time loops.
    ///
    /// Strategy per table:
    /// 1. Movies: multi-row INSERT ON CONFLICT DO NOTHING
    /// 2. Lookup tables (genres, keywords, cast, directors, trailers, countries):
    ///    collect unique values → bulk upsert → bulk SELECT IDs
    /// 3. Join tables (movie_genres, movie_cast, etc.):
    ///    multi-row INSERT ON CONFLICT DO NOTHING
    pub(crate) async fn insert_movie_data_batch(&self, batch: &[MovieData]) -> MovieCrudResult<()> {
        use diesel::insert_into;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        if batch.is_empty() {
            return Ok(());
        }

        let mut conn = self.conn().await?;

        // ================================================================
        // 1. Bulk insert movies
        // ================================================================
        {
            let movie_rows: Vec<_> = batch
                .iter()
                .filter_map(|m| {
                    let release_date = unix_to_naive_datetime(m.release_date).ok()?;
                    Some((
                        movies::movie_id.eq(m.movie_id),
                        movies::title.eq(&m.title),
                        movies::runtime.eq(m.runtime as i32),
                        movies::popularity.eq(m.popularity),
                        movies::imdb_id.eq(m.imdb_id.clone()),
                        movies::mediawiki_id.eq(m.mediawiki_id.clone()),
                        movies::rating.eq(m.rating.clone()),
                        movies::release_date.eq(release_date),
                        movies::original_language.eq(m.original_language.clone()),
                        movies::poster_url.eq(m.poster_url.clone()),
                        movies::overview.eq(m.overview.clone()),
                        movies::tagline.eq(m.tagline.clone()),
                    ))
                })
                .collect();

            if !movie_rows.is_empty() {
                if let Err(e) = insert_into(movies::table)
                    .values(&movie_rows)
                    .on_conflict_do_nothing()
                    .execute(&mut conn)
                    .await
                {
                    error!(
                        "[BATCH] Failed to bulk insert {} movies: {}",
                        movie_rows.len(),
                        e
                    );
                    return Err(MovieCrudError::Diesel(e));
                }
            }
        }

        // ================================================================
        // 2. Delete stale join-table rows for this batch
        //    This ensures removed genres/cast/etc. are cleaned up on re-import
        // ================================================================
        {
            let batch_ids: Vec<i64> = batch.iter().map(|m| m.movie_id).collect();

            diesel::delete(movie_directors::table)
                .filter(movie_directors::movie_id.eq_any(&batch_ids))
                .execute(&mut conn)
                .await?;
            diesel::delete(movie_genres::table)
                .filter(movie_genres::movie_id.eq_any(&batch_ids))
                .execute(&mut conn)
                .await?;
            diesel::delete(movie_keywords::table)
                .filter(movie_keywords::movie_id.eq_any(&batch_ids))
                .execute(&mut conn)
                .await?;
            diesel::delete(movie_cast::table)
                .filter(movie_cast::movie_id.eq_any(&batch_ids))
                .execute(&mut conn)
                .await?;
            diesel::delete(movie_production_countries::table)
                .filter(movie_production_countries::movie_id.eq_any(&batch_ids))
                .execute(&mut conn)
                .await?;
            diesel::delete(movie_trailers::table)
                .filter(movie_trailers::movie_id.eq_any(&batch_ids))
                .execute(&mut conn)
                .await?;
        }

        // ================================================================
        // 3. Directors: collect unique names → bulk upsert → get IDs → join
        // ================================================================
        {
            let unique_directors: HashSet<String> = batch
                .iter()
                .flat_map(|m| m.director.iter())
                .map(|d| d.trim().to_string())
                .filter(|d| !d.is_empty() && d != "null")
                .collect();

            if !unique_directors.is_empty() {
                // Bulk upsert directors
                let dir_values: Vec<_> = unique_directors
                    .iter()
                    .map(|name| directors::name.eq(name.as_str()))
                    .collect();

                if let Err(e) = insert_into(directors::table)
                    .values(&dir_values)
                    .on_conflict(directors::name)
                    .do_nothing()
                    .execute(&mut conn)
                    .await
                {
                    error!("[BATCH] Failed to bulk insert directors: {}", e);
                    return Err(MovieCrudError::Diesel(e));
                }

                // Fetch all director IDs
                let dir_names: Vec<&str> = unique_directors.iter().map(|s| s.as_str()).collect();
                let dir_map: HashMap<String, uuid::Uuid> = directors::table
                    .filter(directors::name.eq_any(&dir_names))
                    .select((directors::name, directors::director_id))
                    .load::<(String, uuid::Uuid)>(&mut conn)
                    .await?
                    .into_iter()
                    .collect();

                // Bulk join movies → directors
                let mut join_rows = Vec::new();
                for movie in batch {
                    for d in &movie.director {
                        let d = d.trim();
                        if d.is_empty() || d == "null" {
                            continue;
                        }
                        if let Some(dir_id) = dir_map.get(d) {
                            join_rows.push((
                                movie_directors::movie_id.eq(movie.movie_id),
                                movie_directors::director_id.eq(*dir_id),
                            ));
                        }
                    }
                }
                if !join_rows.is_empty() {
                    if let Err(e) = insert_into(movie_directors::table)
                        .values(&join_rows)
                        .on_conflict_do_nothing()
                        .execute(&mut conn)
                        .await
                    {
                        error!("[BATCH] Failed to bulk insert movie_directors: {}", e);
                        return Err(MovieCrudError::Diesel(e));
                    }
                }
            }
        }

        // ================================================================
        // 3. Genres: collect unique → bulk upsert → get IDs → join
        // ================================================================
        {
            let unique_genres: HashSet<String> = batch
                .iter()
                .flat_map(|m| m.genres.iter())
                .cloned()
                .collect();

            if !unique_genres.is_empty() {
                let genre_values: Vec<_> = unique_genres
                    .iter()
                    .map(|name| genres::name.eq(name.as_str()))
                    .collect();

                if let Err(e) = insert_into(genres::table)
                    .values(&genre_values)
                    .on_conflict(genres::name)
                    .do_nothing()
                    .execute(&mut conn)
                    .await
                {
                    error!("[BATCH] Failed to bulk insert genres: {}", e);
                    return Err(MovieCrudError::Diesel(e));
                }

                let genre_names: Vec<&str> = unique_genres.iter().map(|s| s.as_str()).collect();
                let genre_map: HashMap<String, uuid::Uuid> = genres::table
                    .filter(genres::name.eq_any(&genre_names))
                    .select((genres::name, genres::genre_id))
                    .load::<(String, uuid::Uuid)>(&mut conn)
                    .await?
                    .into_iter()
                    .collect();

                let mut join_rows = Vec::new();
                for movie in batch {
                    for genre in &movie.genres {
                        if let Some(genre_id) = genre_map.get(genre) {
                            join_rows.push((
                                movie_genres::movie_id.eq(movie.movie_id),
                                movie_genres::genre_id.eq(*genre_id),
                            ));
                        }
                    }
                }
                if !join_rows.is_empty() {
                    if let Err(e) = insert_into(movie_genres::table)
                        .values(&join_rows)
                        .on_conflict_do_nothing()
                        .execute(&mut conn)
                        .await
                    {
                        error!("[BATCH] Failed to bulk insert movie_genres: {}", e);
                        return Err(MovieCrudError::Diesel(e));
                    }
                }
            }
        }

        // ================================================================
        // 4. Keywords: collect unique → bulk upsert → get IDs → join
        // ================================================================
        {
            let unique_keywords: HashSet<String> = batch
                .iter()
                .flat_map(|m| m.keywords.iter())
                .cloned()
                .collect();

            if !unique_keywords.is_empty() {
                let kw_values: Vec<_> = unique_keywords
                    .iter()
                    .map(|name| keywords::name.eq(name.as_str()))
                    .collect();

                if let Err(e) = insert_into(keywords::table)
                    .values(&kw_values)
                    .on_conflict(keywords::name)
                    .do_nothing()
                    .execute(&mut conn)
                    .await
                {
                    error!("[BATCH] Failed to bulk insert keywords: {}", e);
                    return Err(MovieCrudError::Diesel(e));
                }

                let kw_names: Vec<&str> = unique_keywords.iter().map(|s| s.as_str()).collect();
                let kw_map: HashMap<String, uuid::Uuid> = keywords::table
                    .filter(keywords::name.eq_any(&kw_names))
                    .select((keywords::name, keywords::keyword_id))
                    .load::<(String, uuid::Uuid)>(&mut conn)
                    .await?
                    .into_iter()
                    .collect();

                let mut join_rows = Vec::new();
                for movie in batch {
                    for kw in &movie.keywords {
                        if let Some(kw_id) = kw_map.get(kw) {
                            join_rows.push((
                                movie_keywords::movie_id.eq(movie.movie_id),
                                movie_keywords::keyword_id.eq(*kw_id),
                            ));
                        }
                    }
                }
                if !join_rows.is_empty() {
                    if let Err(e) = insert_into(movie_keywords::table)
                        .values(&join_rows)
                        .on_conflict_do_nothing()
                        .execute(&mut conn)
                        .await
                    {
                        error!("[BATCH] Failed to bulk insert movie_keywords: {}", e);
                        return Err(MovieCrudError::Diesel(e));
                    }
                }
            }
        }

        // ================================================================
        // 5. Cast members: collect unique (name, profile_url) → bulk upsert → get IDs → join
        // ================================================================
        {
            // Use (name, profile_url) as unique key
            let unique_cast: HashSet<(String, Option<String>)> = batch
                .iter()
                .flat_map(|m| m.cast.iter())
                .map(|c| (c.name.clone(), c.profile_url.clone()))
                .collect();

            if !unique_cast.is_empty() {
                let cast_values: Vec<_> = unique_cast
                    .iter()
                    .map(|(name, profile_url)| {
                        (
                            cast_members::name.eq(name.as_str()),
                            cast_members::profile_url.eq(profile_url.clone()),
                        )
                    })
                    .collect();

                if let Err(e) = insert_into(cast_members::table)
                    .values(&cast_values)
                    .on_conflict((cast_members::name, cast_members::profile_url))
                    .do_nothing()
                    .execute(&mut conn)
                    .await
                {
                    error!("[BATCH] Failed to bulk insert cast_members: {}", e);
                    return Err(MovieCrudError::Diesel(e));
                }

                // Fetch all cast member IDs by (name, profile_url)
                let cast_names: Vec<&str> = unique_cast.iter().map(|(n, _)| n.as_str()).collect();
                let cast_rows: Vec<(uuid::Uuid, String, Option<String>)> = cast_members::table
                    .filter(cast_members::name.eq_any(&cast_names))
                    .select((
                        cast_members::cast_id,
                        cast_members::name,
                        cast_members::profile_url,
                    ))
                    .load::<(uuid::Uuid, String, Option<String>)>(&mut conn)
                    .await?;

                let cast_map: HashMap<(String, Option<String>), uuid::Uuid> = cast_rows
                    .into_iter()
                    .map(|(id, name, url)| ((name, url), id))
                    .collect();

                let mut join_rows = Vec::new();
                for movie in batch {
                    for cast in &movie.cast {
                        let key = (cast.name.clone(), cast.profile_url.clone());
                        if let Some(cast_id) = cast_map.get(&key) {
                            join_rows.push((
                                movie_cast::movie_id.eq(movie.movie_id),
                                movie_cast::cast_id.eq(*cast_id),
                            ));
                        }
                    }
                }
                if !join_rows.is_empty() {
                    if let Err(e) = insert_into(movie_cast::table)
                        .values(&join_rows)
                        .on_conflict_do_nothing()
                        .execute(&mut conn)
                        .await
                    {
                        error!("[BATCH] Failed to bulk insert movie_cast: {}", e);
                        return Err(MovieCrudError::Diesel(e));
                    }
                }
            }
        }

        // ================================================================
        // 6. Production countries: bulk upsert → join
        // ================================================================
        {
            let unique_countries: HashSet<String> = batch
                .iter()
                .flat_map(|m| m.production_countries.iter())
                .cloned()
                .collect();

            if !unique_countries.is_empty() {
                let country_values: Vec<_> = unique_countries
                    .iter()
                    .map(|code| {
                        (
                            production_countries::country_code.eq(code.as_str()),
                            production_countries::name.eq(code.as_str()),
                        )
                    })
                    .collect();

                if let Err(e) = insert_into(production_countries::table)
                    .values(&country_values)
                    .on_conflict(production_countries::country_code)
                    .do_nothing()
                    .execute(&mut conn)
                    .await
                {
                    error!("[BATCH] Failed to bulk insert production_countries: {}", e);
                    return Err(MovieCrudError::Diesel(e));
                }

                // Countries use country_code directly — no ID lookup needed
                let mut join_rows = Vec::new();
                for movie in batch {
                    for country in &movie.production_countries {
                        join_rows.push((
                            movie_production_countries::movie_id.eq(movie.movie_id),
                            movie_production_countries::country_code.eq(country.as_str()),
                        ));
                    }
                }
                if !join_rows.is_empty() {
                    if let Err(e) = insert_into(movie_production_countries::table)
                        .values(&join_rows)
                        .on_conflict_do_nothing()
                        .execute(&mut conn)
                        .await
                    {
                        error!(
                            "[BATCH] Failed to bulk insert movie_production_countries: {}",
                            e
                        );
                        return Err(MovieCrudError::Diesel(e));
                    }
                }
            }
        }

        // ================================================================
        // 7. Trailers: collect unique video_keys → bulk upsert → get IDs → join
        // ================================================================
        {
            let unique_keys: HashSet<String> = batch
                .iter()
                .flat_map(|m| m.video_keys.iter())
                .cloned()
                .collect();

            if !unique_keys.is_empty() {
                let trailer_values: Vec<_> = unique_keys
                    .iter()
                    .map(|vk| trailers::video_key.eq(vk.as_str()))
                    .collect();

                if let Err(e) = insert_into(trailers::table)
                    .values(&trailer_values)
                    .on_conflict(trailers::video_key)
                    .do_nothing()
                    .execute(&mut conn)
                    .await
                {
                    error!("[BATCH] Failed to bulk insert trailers: {}", e);
                    return Err(MovieCrudError::Diesel(e));
                }

                let vk_list: Vec<&str> = unique_keys.iter().map(|s| s.as_str()).collect();
                let trailer_map: HashMap<String, uuid::Uuid> = trailers::table
                    .filter(trailers::video_key.eq_any(&vk_list))
                    .select((trailers::video_key, trailers::trailer_id))
                    .load::<(String, uuid::Uuid)>(&mut conn)
                    .await?
                    .into_iter()
                    .collect();

                let mut join_rows = Vec::new();
                for movie in batch {
                    for vk in &movie.video_keys {
                        if let Some(trailer_id) = trailer_map.get(vk) {
                            join_rows.push((
                                movie_trailers::movie_id.eq(movie.movie_id),
                                movie_trailers::trailer_id.eq(*trailer_id),
                            ));
                        }
                    }
                }
                #[allow(clippy::collapsible_if)]
                if !join_rows.is_empty() {
                    if let Err(e) = insert_into(movie_trailers::table)
                        .values(&join_rows)
                        .on_conflict_do_nothing()
                        .execute(&mut conn)
                        .await
                    {
                        error!("[BATCH] Failed to bulk insert movie_trailers: {}", e);
                        return Err(MovieCrudError::Diesel(e));
                    }
                }
            }
        }

        Ok(())
    }
}
