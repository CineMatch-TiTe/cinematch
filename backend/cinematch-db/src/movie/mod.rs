pub mod insert;

use crate::Database;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::DbError;
use crate::DbResult;

use crate::vector::models::{MovieData, CastMember};

impl Database {
    pub async fn get_movie_directors(&self, movie_id: i64) -> DbResult<Vec<String>> {
        use crate::schema::{movie_directors, directors};
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        let mut conn = self.conn().await?;
        let names = movie_directors::table
            .inner_join(directors::table)
            .filter(movie_directors::movie_id.eq(movie_id))
            .select(directors::name)
            .load::<String>(&mut conn)
            .await
            .map_err(DbError::from)?;
        Ok(names)
    }

    pub async fn get_movie_genres(&self, movie_id: i64) -> DbResult<Vec<String>> {
        use crate::schema::{movie_genres, genres};
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        let mut conn = self.conn().await?;
        match movie_genres::table
            .inner_join(genres::table)
            .filter(movie_genres::movie_id.eq(movie_id))
            .select(genres::name)
            .load::<String>(&mut conn)
            .await {
            Ok(genres_vec) => Ok(genres_vec),
            Err(e) => {
                Err(DbError::from(e))
            }
        }
    }

    pub async fn get_movie_keywords(&self, movie_id: i64) -> DbResult<Vec<String>> {
        use crate::schema::{movie_keywords, keywords};
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        let mut conn = self.conn().await?;
        let keywords_vec = movie_keywords::table
            .inner_join(keywords::table)
            .filter(movie_keywords::movie_id.eq(movie_id))
            .select(keywords::name)
            .load::<String>(&mut conn)
            .await
            .map_err(DbError::from)?;
        Ok(keywords_vec)
    }

    pub async fn get_movie_cast(&self, movie_id: i64) -> DbResult<Vec<crate::vector::models::CastMember>> {
        use crate::schema::{movie_cast, cast_members};
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        let mut conn = self.conn().await?;
        let cast_vec = movie_cast::table
            .inner_join(cast_members::table)
            .filter(movie_cast::movie_id.eq(movie_id))
            .select((cast_members::name, cast_members::profile_url))
            .load::<(String, Option<String>)>(&mut conn)
            .await
            .map_err(DbError::from)?
            .into_iter()
            .map(|(name, profile_url)| CastMember { name, profile_url })
            .collect();
        Ok(cast_vec)
    }

    /// empty currently
    pub async fn get_movie_production_countries(&self, movie_id: i64) -> DbResult<Vec<String>> {
        use crate::schema::{movie_production_countries, production_countries};
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        let mut conn = self.conn().await?;
        let prod_countries = movie_production_countries::table
            .inner_join(production_countries::table)
            .filter(movie_production_countries::movie_id.eq(movie_id))
            .select(production_countries::country_code)
            .load::<String>(&mut conn)
            .await
            .map_err(DbError::from)?;
        Ok(prod_countries)
    }

    pub async fn get_trailers(&self, movie_id: i64) -> DbResult<Vec<String>> {
        use crate::schema::{movie_trailers, trailers};
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        let mut conn = self.conn().await?;
        let video_keys = movie_trailers::table
            .inner_join(trailers::table)
            .filter(movie_trailers::movie_id.eq(movie_id))
            .select(trailers::video_key)
            .load::<String>(&mut conn)
            .await
            .map_err(DbError::from)?;
        Ok(video_keys)
    }

    pub async fn get_genres(&self) -> DbResult<Vec<String>> {
        use diesel_async::RunQueryDsl;
        use crate::schema::genres;

        let mut conn = self.conn().await?;
        let mut names = genres::table
            .select(genres::name)
            .load::<String>(&mut conn)
            .await
            .map_err(|e| {
                log::error!("DB error in get_genres: {}", e);
                DbError::Query(e)
            })?;
        names.sort();
        Ok(names)
    }

    pub async fn get_movie_by_id(&self, movie_id: i64) -> DbResult<Option<MovieData>> {
        use crate::schema::movies;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        let mut conn = self.conn().await?;

        use crate::models::Movie;
        // Main movie row
        let movie_row: Option<Movie> = movies::table
            .filter(movies::movie_id.eq(movie_id))
            .first::<Movie>(&mut conn)
            .await
            .optional()
            .map_err(DbError::from)?;

        let movie_row = match movie_row {
            Some(row) => row,
            None => return Ok(None),
        };

        let director_names = self.get_movie_directors(movie_id).await?;
        let genres_vec = self.get_movie_genres(movie_id).await?;
        let keywords_vec = self.get_movie_keywords(movie_id).await?;
        let cast_vec = self.get_movie_cast(movie_id).await?;
        let prod_countries = self.get_movie_production_countries(movie_id).await?;
        let video_keys = self.get_trailers(movie_id).await?;

        let movie_data = MovieData {
            movie_id: movie_row.movie_id,
            title: movie_row.title,
            runtime: movie_row.runtime as i64,
            popularity: movie_row.popularity,
            imdb_id: movie_row.imdb_id,
            mediawiki_id: movie_row.mediawiki_id,
            rating: movie_row.rating,
            release_date: movie_row.release_date.and_utc().timestamp(),
            original_language: movie_row.original_language,
            poster_url: movie_row.poster_url,
            overview: movie_row.overview,
            tagline: movie_row.tagline,
            director: director_names,
            genres: genres_vec,
            keywords: keywords_vec,
            cast: cast_vec,
            production_countries: prod_countries,
            reviews: vec![], // TODO: fill if you have reviews
            video_keys,
        };
        Ok(Some(movie_data))
    }
}