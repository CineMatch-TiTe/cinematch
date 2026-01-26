// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "auth_provider"))]
    pub struct AuthProvider;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "party_state"))]
    pub struct PartyState;
}

diesel::table! {
    cast_members (cast_id) {
        cast_id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        profile_url -> Nullable<Text>,
    }
}

diesel::table! {
    directors (director_id) {
        director_id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::AuthProvider;

    external_accounts (id) {
        id -> Uuid,
        user_id -> Uuid,
        provider -> AuthProvider,
        #[max_length = 255]
        provider_user_id -> Varchar,
        #[max_length = 255]
        email -> Nullable<Varchar>,
        #[max_length = 255]
        display_name -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    genres (genre_id) {
        genre_id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
    }
}

diesel::table! {
    keywords (keyword_id) {
        keyword_id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
    }
}

diesel::table! {
    movie_cast (movie_id, cast_id) {
        movie_id -> Int8,
        cast_id -> Uuid,
    }
}

diesel::table! {
    movie_directors (movie_id, director_id) {
        movie_id -> Int8,
        director_id -> Uuid,
    }
}

diesel::table! {
    movie_genres (movie_id, genre_id) {
        movie_id -> Int8,
        genre_id -> Uuid,
    }
}

diesel::table! {
    movie_keywords (movie_id, keyword_id) {
        movie_id -> Int8,
        keyword_id -> Uuid,
    }
}

diesel::table! {
    movie_production_countries (movie_id, country_code) {
        movie_id -> Int8,
        #[max_length = 3]
        country_code -> Bpchar,
    }
}

diesel::table! {
    movie_trailers (movie_id, trailer_id) {
        movie_id -> Int8,
        trailer_id -> Uuid,
    }
}

diesel::table! {
    movies (movie_id) {
        movie_id -> Int8,
        title -> Text,
        runtime -> Int4,
        popularity -> Float4,
        imdb_id -> Nullable<Text>,
        mediawiki_id -> Nullable<Text>,
        rating -> Nullable<Text>,
        release_date -> Timestamptz,
        original_language -> Nullable<Text>,
        poster_url -> Nullable<Text>,
        overview -> Nullable<Text>,
        tagline -> Nullable<Text>,
        release_year -> Nullable<Int4>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::PartyState;

    parties (id) {
        id -> Uuid,
        party_leader_id -> Uuid,
        state -> PartyState,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        disbanded_at -> Nullable<Timestamptz>,
        selected_movie_id -> Nullable<Int8>,
        can_vote -> Bool,
        voting_round -> Nullable<Int2>,
        phase_entered_at -> Timestamptz,
    }
}

diesel::table! {
    party_codes (code) {
        #[max_length = 4]
        code -> Bpchar,
        party_id -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    party_members (user_id, party_id) {
        user_id -> Uuid,
        party_id -> Uuid,
        joined_at -> Timestamptz,
        is_ready -> Bool,
    }
}

diesel::table! {
    prefs_exclude_genre (user_id, genre_id) {
        user_id -> Uuid,
        genre_id -> Uuid,
    }
}

diesel::table! {
    prefs_include_genre (user_id, genre_id) {
        user_id -> Uuid,
        genre_id -> Uuid,
    }
}

diesel::table! {
    production_countries (country_code) {
        #[max_length = 3]
        country_code -> Bpchar,
        #[max_length = 255]
        name -> Varchar,
    }
}

diesel::table! {
    shown_movies (party_id, user_id, movie_id) {
        party_id -> Uuid,
        user_id -> Uuid,
        movie_id -> Int8,
        shown_at -> Timestamptz,
    }
}

diesel::table! {
    trailers (trailer_id) {
        trailer_id -> Uuid,
        video_key -> Text,
    }
}

diesel::table! {
    user_preferences (user_id) {
        user_id -> Uuid,
        target_release_year -> Nullable<Int4>,
        release_year_flex -> Int4,
        is_tite -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    user_tastes (taste_id) {
        taste_id -> Uuid,
        user_id -> Uuid,
        party_id -> Nullable<Uuid>,
        movie_id -> Int8,
        liked -> Bool,
        updated_at -> Timestamptz,
        review -> Nullable<Int4>,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 32]
        username -> Varchar,
        oneshot -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    votes (party_id, user_id, movie_id) {
        party_id -> Uuid,
        user_id -> Uuid,
        movie_id -> Int8,
        vote_value -> Bool,
        voted_at -> Timestamptz,
    }
}

diesel::joinable!(external_accounts -> users (user_id));
diesel::joinable!(movie_cast -> cast_members (cast_id));
diesel::joinable!(movie_cast -> movies (movie_id));
diesel::joinable!(movie_directors -> directors (director_id));
diesel::joinable!(movie_directors -> movies (movie_id));
diesel::joinable!(movie_genres -> genres (genre_id));
diesel::joinable!(movie_genres -> movies (movie_id));
diesel::joinable!(movie_keywords -> keywords (keyword_id));
diesel::joinable!(movie_keywords -> movies (movie_id));
diesel::joinable!(movie_production_countries -> movies (movie_id));
diesel::joinable!(movie_production_countries -> production_countries (country_code));
diesel::joinable!(movie_trailers -> movies (movie_id));
diesel::joinable!(movie_trailers -> trailers (trailer_id));
diesel::joinable!(parties -> movies (selected_movie_id));
diesel::joinable!(parties -> users (party_leader_id));
diesel::joinable!(party_codes -> parties (party_id));
diesel::joinable!(party_members -> parties (party_id));
diesel::joinable!(party_members -> users (user_id));
diesel::joinable!(prefs_exclude_genre -> genres (genre_id));
diesel::joinable!(prefs_exclude_genre -> users (user_id));
diesel::joinable!(prefs_include_genre -> genres (genre_id));
diesel::joinable!(prefs_include_genre -> users (user_id));
diesel::joinable!(shown_movies -> movies (movie_id));
diesel::joinable!(shown_movies -> parties (party_id));
diesel::joinable!(shown_movies -> users (user_id));
diesel::joinable!(user_preferences -> users (user_id));
diesel::joinable!(user_tastes -> movies (movie_id));

diesel::allow_tables_to_appear_in_same_query!(
    cast_members,
    directors,
    external_accounts,
    genres,
    keywords,
    movie_cast,
    movie_directors,
    movie_genres,
    movie_keywords,
    movie_production_countries,
    movie_trailers,
    movies,
    parties,
    party_codes,
    party_members,
    prefs_exclude_genre,
    prefs_include_genre,
    production_countries,
    shown_movies,
    trailers,
    user_preferences,
    user_tastes,
    users,
    votes,
);
