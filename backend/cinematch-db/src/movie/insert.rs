const BATCH_SIZE: usize = 20;
use crate::schema::{
    cast_members, genres, keywords, movie_cast, movie_genres, movie_keywords,
    movie_production_countries, movies, production_countries,
};
use crate::vector::models::MovieData;
use chrono::DateTime;
use diesel::result::Error as DieselError;

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
    chrono::NaiveDateTime::from_timestamp_opt(secs, 0)
        .ok_or(MovieCrudError::Diesel(DieselError::NotFound))
}

impl Database {
    /// Batch insert movies and all related data (genres, keywords, cast, countries)
    pub async fn insert_movie_data_batch(&self, movies: &[MovieData]) -> MovieCrudResult<()> {
        for movie in movies {
            self.insert_movie_data(movie).await?;
        }
        Ok(())
    }
    /// Insert a new movie and all related data (genres, keywords, cast, countries)
    pub async fn insert_movie_data(&self, movie: &MovieData) -> MovieCrudResult<()> {
        use diesel::insert_into;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        let mut conn = self.conn().await?;

        // Insert movie core
        let release_date = unix_to_naive_datetime(movie.release_date)?;
        if let Err(e) = insert_into(movies::table)
            .values((
                movies::movie_id.eq(movie.movie_id),
                movies::title.eq(&movie.title),
                movies::runtime.eq(movie.runtime as i32),
                movies::popularity.eq(movie.popularity),
                movies::imdb_id.eq(movie.imdb_id.clone()),
                movies::mediawiki_id.eq(movie.mediawiki_id.clone()),
                movies::rating.eq(movie.rating.clone()),
                movies::release_date.eq(release_date),
                movies::original_language.eq(movie.original_language.clone()),
                movies::poster_url.eq(movie.poster_url.clone()),
                movies::overview.eq(movie.overview.clone()),
                movies::tagline.eq(movie.tagline.clone()),
            ))
            .on_conflict_do_nothing()
            .execute(&mut conn)
            .await
        {
            eprintln!(
                "[DB ERROR] Failed to insert movie {}: {}",
                movie.movie_id, e
            );
            return Err(MovieCrudError::Diesel(e));
        }

        // Insert directors (if not exist) and join
        for director_name in &movie.director {
            let name = director_name.trim();
            if name.is_empty() || name == "null" {
                continue;
            }
            let director_id = match diesel::insert_into(crate::schema::directors::table)
                .values(crate::schema::directors::name.eq(name))
                .returning(crate::schema::directors::director_id)
                .get_result::<uuid::Uuid>(&mut conn)
                .await
            {
                Ok(id) => id,
                Err(e) => {
                    eprintln!(
                        "[DB ERROR] Failed to insert director '{}' for movie {}: {}",
                        name, movie.movie_id, e
                    );
                    return Err(MovieCrudError::Diesel(e));
                }
            };
            if let Err(e) = diesel::insert_into(crate::schema::movie_directors::table)
                .values((
                    crate::schema::movie_directors::movie_id.eq(movie.movie_id),
                    crate::schema::movie_directors::director_id.eq(director_id),
                ))
                .on_conflict_do_nothing()
                .execute(&mut conn)
                .await
            {
                eprintln!(
                    "[DB ERROR] Failed to insert movie_directors for movie {}: {}",
                    movie.movie_id, e
                );
                return Err(MovieCrudError::Diesel(e));
            }
        }
        // Insert genres (if not exist) and join
        for genre in &movie.genres {
            let genre_id = match diesel::insert_into(genres::table)
                .values(genres::name.eq(genre))
                .on_conflict(genres::name)
                .do_update()
                .set(genres::name.eq(genre))
                .returning(genres::genre_id)
                .get_result::<uuid::Uuid>(&mut conn)
                .await
            {
                Ok(id) => id,
                Err(e) => {
                    eprintln!(
                        "[DB ERROR] Failed to insert genre '{}' for movie {}: {}",
                        genre, movie.movie_id, e
                    );
                    return Err(MovieCrudError::Diesel(e));
                }
            };
            if let Err(e) = diesel::insert_into(movie_genres::table)
                .values((
                    movie_genres::movie_id.eq(movie.movie_id),
                    movie_genres::genre_id.eq(genre_id),
                ))
                .on_conflict_do_nothing()
                .execute(&mut conn)
                .await
            {
                eprintln!(
                    "[DB ERROR] Failed to insert movie_genres for movie {}: {}",
                    movie.movie_id, e
                );
                return Err(MovieCrudError::Diesel(e));
            }
        }

        // Insert keywords (if not exist) and join
        for keyword in &movie.keywords {
            let keyword_id = match diesel::insert_into(keywords::table)
                .values(keywords::name.eq(keyword))
                .on_conflict(keywords::name)
                .do_update()
                .set(keywords::name.eq(keyword))
                .returning(keywords::keyword_id)
                .get_result::<uuid::Uuid>(&mut conn)
                .await
            {
                Ok(id) => id,
                Err(e) => {
                    eprintln!(
                        "[DB ERROR] Failed to insert keyword '{}' for movie {}: {}",
                        keyword, movie.movie_id, e
                    );
                    return Err(MovieCrudError::Diesel(e));
                }
            };
            if let Err(e) = diesel::insert_into(movie_keywords::table)
                .values((
                    movie_keywords::movie_id.eq(movie.movie_id),
                    movie_keywords::keyword_id.eq(keyword_id),
                ))
                .on_conflict_do_nothing()
                .execute(&mut conn)
                .await
            {
                eprintln!(
                    "[DB ERROR] Failed to insert movie_keywords for movie {}: {}",
                    movie.movie_id, e
                );
                return Err(MovieCrudError::Diesel(e));
            }
        }

        // Insert cast members (if not exist) and join
        for cast in &movie.cast {
            let cast_id = match diesel::insert_into(cast_members::table)
                .values((
                    cast_members::name.eq(&cast.name),
                    cast_members::profile_url.eq(cast.profile_url.clone()),
                ))
                .on_conflict((cast_members::name, cast_members::profile_url))
                .do_update()
                .set(cast_members::name.eq(&cast.name))
                .returning(cast_members::cast_id)
                .get_result::<uuid::Uuid>(&mut conn)
                .await
            {
                Ok(id) => id,
                Err(e) => {
                    eprintln!(
                        "[DB ERROR] Failed to insert cast '{}' for movie {}: {}",
                        cast.name, movie.movie_id, e
                    );
                    return Err(MovieCrudError::Diesel(e));
                }
            };
            if let Err(e) = diesel::insert_into(movie_cast::table)
                .values((
                    movie_cast::movie_id.eq(movie.movie_id),
                    movie_cast::cast_id.eq(cast_id),
                ))
                .on_conflict_do_nothing()
                .execute(&mut conn)
                .await
            {
                eprintln!(
                    "[DB ERROR] Failed to insert movie_cast for movie {}: {}",
                    movie.movie_id, e
                );
                return Err(MovieCrudError::Diesel(e));
            }
        }

        // Insert production countries (if not exist) and join
        for country in &movie.production_countries {
            if let Err(e) = diesel::insert_into(production_countries::table)
                .values((
                    production_countries::country_code.eq(country),
                    production_countries::name.eq(country),
                ))
                .on_conflict(production_countries::country_code)
                .do_nothing()
                .execute(&mut conn)
                .await
            {
                eprintln!(
                    "[DB ERROR] Failed to insert production_country '{}' for movie {}: {}",
                    country, movie.movie_id, e
                );
                return Err(MovieCrudError::Diesel(e));
            }
            if let Err(e) = diesel::insert_into(movie_production_countries::table)
                .values((
                    movie_production_countries::movie_id.eq(movie.movie_id),
                    movie_production_countries::country_code.eq(country),
                ))
                .on_conflict_do_nothing()
                .execute(&mut conn)
                .await
            {
                eprintln!(
                    "[DB ERROR] Failed to insert movie_production_countries for movie {}: {}",
                    movie.movie_id, e
                );
                return Err(MovieCrudError::Diesel(e));
            }
        }
        // Insert video keys for this movie
        for vk in &movie.video_keys {
            // Insert trailer (video_key) if not exist, then join in movie_trailers
            let trailer_id = match diesel::insert_into(crate::schema::trailers::table)
                .values(crate::schema::trailers::video_key.eq(vk))
                .on_conflict(crate::schema::trailers::video_key)
                .do_update()
                .set(crate::schema::trailers::video_key.eq(vk))
                .returning(crate::schema::trailers::trailer_id)
                .get_result::<uuid::Uuid>(&mut conn)
                .await
            {
                Ok(id) => id,
                Err(e) => {
                    eprintln!(
                        "[DB ERROR] Failed to insert trailer '{}' for movie {}: {}",
                        vk, movie.movie_id, e
                    );
                    return Err(MovieCrudError::Diesel(e));
                }
            };
            if let Err(e) = diesel::insert_into(crate::schema::movie_trailers::table)
                .values((
                    crate::schema::movie_trailers::movie_id.eq(movie.movie_id),
                    crate::schema::movie_trailers::trailer_id.eq(trailer_id),
                ))
                .on_conflict_do_nothing()
                .execute(&mut conn)
                .await
            {
                eprintln!(
                    "[DB ERROR] Failed to insert movie_trailers for movie {}: {}",
                    movie.movie_id, e
                );
                return Err(MovieCrudError::Diesel(e));
            }
        }

        Ok(())
    }
}
