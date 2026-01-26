pub mod insert;

use crate::Database;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use uuid::Uuid;

use crate::Movie;

use crate::DbError;
use crate::DbResult;

use crate::vector::models::{CastMember, MovieData};
use cinematch_common::SearchFilter;

impl Database {
    pub async fn get_movie_directors(&self, movie_id: i64) -> DbResult<Vec<String>> {
        use crate::schema::{directors, movie_directors};
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
        use crate::schema::{genres, movie_genres};
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        let mut conn = self.conn().await?;
        match movie_genres::table
            .inner_join(genres::table)
            .filter(movie_genres::movie_id.eq(movie_id))
            .select(genres::name)
            .load::<String>(&mut conn)
            .await
        {
            Ok(genres_vec) => Ok(genres_vec),
            Err(e) => Err(DbError::from(e)),
        }
    }

    pub async fn get_movie_keywords(&self, movie_id: i64) -> DbResult<Vec<String>> {
        use crate::schema::{keywords, movie_keywords};
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

    pub async fn get_movie_cast(
        &self,
        movie_id: i64,
    ) -> DbResult<Vec<crate::vector::models::CastMember>> {
        use crate::schema::{cast_members, movie_cast};
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

    /// Get all genres as a map: name -> id
    pub async fn get_genres(&self) -> DbResult<std::collections::HashMap<String, Uuid>> {
        use crate::schema::genres::dsl::*;
        let mut conn = self.conn().await?;
        let rows = genres
            .select((name, genre_id))
            .load::<(String, Uuid)>(&mut conn)
            .await
            .map_err(DbError::from)?;

        Ok(rows.into_iter().collect())
    }

    /// Get genre ID by name (case-insensitive)
    pub async fn get_genre_id_by_name(&self, genre_name: &str) -> DbResult<Option<Uuid>> {
        use crate::schema::genres::dsl::*;
        let mut conn = self.conn().await?;
        let result = genres
            .filter(name.ilike(genre_name))
            .select(genre_id)
            .first::<Uuid>(&mut conn)
            .await
            .optional()
            .map_err(DbError::from)?;
        Ok(result)
    }

    /// Get Documentary genre ID (helper for default preferences)
    pub async fn get_doc_genre_id(&self) -> DbResult<Option<Uuid>> {
        self.get_genre_id_by_name("Documentary").await
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

    /// Movie IDs by popularity desc, for ballot fallback when picks are insufficient.
    pub async fn get_popular_movie_ids(&self, limit: i64) -> DbResult<Vec<i64>> {
        use crate::schema::movies;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        let mut conn = self.conn().await?;
        let ids = movies::table
            .order(movies::popularity.desc())
            .limit(limit)
            .select(movies::movie_id)
            .load::<i64>(&mut conn)
            .await
            .map_err(DbError::from)?;
        Ok(ids)
    }

    pub async fn get_popular_movies(&self, limit: i64) -> DbResult<Vec<MovieData>> {
        use crate::schema::movies;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        let mut conn = self.conn().await?;

        let movie_rows: Vec<Movie> = movies::table
            .order(movies::popularity.desc())
            .limit(limit)
            .load::<Movie>(&mut conn)
            .await
            .map_err(DbError::from)?;

        let mut movies_data = Vec::with_capacity(movie_rows.len());
        for movie_row in movie_rows {
            if let Some(movie_data) = self.get_movie_by_id(movie_row.movie_id).await? {
                movies_data.push(movie_data);
            }
        }

        Ok(movies_data)
    }

    pub async fn search_movies(
        &self,
        name: &str,
        page: i64,
        filter_opt: Option<SearchFilter>,
    ) -> DbResult<Vec<MovieData>> {
        use crate::schema::{movie_genres, movies};
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        let mut conn = self.conn().await?;

        let pattern = format!("%{}", name);
        let pattern2 = format!("{}%", name);

        let mut query = movies::table
            .filter(
                movies::title
                    .ilike(pattern.clone())
                    .or(movies::title.ilike(pattern2.clone())),
            )
            .into_boxed();

        let genre_map = self.get_genres().await?;

        // Apply filters if provided
        if let Some(filter) = filter_opt {
            // Exclude genres - use subquery
            // Clone the vector to ensure it lives long enough for the query
            if !filter.exclude_genres.is_empty() {
                let exclude_genres: Vec<Uuid> = filter
                    .exclude_genres
                    .into_iter()
                    .filter_map(|name| genre_map.get(&name).copied())
                    .collect();
                use diesel::dsl::not;
                query = query.filter(not(movies::movie_id.eq_any(
                    movie_genres::table
                        .filter(movie_genres::genre_id.eq_any(exclude_genres))
                        .select(movie_genres::movie_id),
                )));
            }

            // Include genres (must have at least one of these) - use subquery
            // Clone the vector to ensure it lives long enough for the query
            if !filter.include_genres.is_empty() {
                let include_genres: Vec<Uuid> = filter
                    .include_genres
                    .into_iter()
                    .filter_map(|name| genre_map.get(&name).copied())
                    .collect();
                query = query.filter(
                    movies::movie_id.eq_any(
                        movie_genres::table
                            .filter(movie_genres::genre_id.eq_any(include_genres))
                            .select(movie_genres::movie_id),
                    ),
                );
            }

            // Year range filter
            if let Some(min_year) = filter.min_year {
                query = query.filter(movies::release_year.ge(min_year));
            }
            if let Some(max_year) = filter.max_year {
                query = query.filter(movies::release_year.le(max_year));
            }

            // Runtime range filter
            if let Some(min_runtime) = filter.min_runtime {
                query = query.filter(movies::runtime.ge(min_runtime));
            }
            if let Some(max_runtime) = filter.max_runtime {
                query = query.filter(movies::runtime.le(max_runtime));
            }
        }

        let movie_rows: Vec<Movie> = query
            .order(movies::popularity.desc())
            .limit(10)
            .offset((page - 1) * 10)
            .load::<Movie>(&mut conn)
            .await
            .map_err(DbError::from)?;

        let mut movies_data = Vec::with_capacity(movie_rows.len());
        for movie_row in movie_rows {
            if let Some(movie_data) = self.get_movie_by_id(movie_row.movie_id).await? {
                movies_data.push(movie_data);
            }
        }

        Ok(movies_data)
    }
}
