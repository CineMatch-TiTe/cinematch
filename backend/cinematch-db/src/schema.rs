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
    use diesel::sql_types::*;
    use super::sql_types::PartyState;

    parties (id) {
        id -> Uuid,
        party_leader_id -> Uuid,
        state -> PartyState,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        disbanded_at -> Nullable<Timestamptz>,
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
    users (id) {
        id -> Uuid,
        #[max_length = 32]
        username -> Varchar,
        oneshot -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(external_accounts -> users (user_id));
diesel::joinable!(parties -> users (party_leader_id));
diesel::joinable!(party_codes -> parties (party_id));
diesel::joinable!(party_members -> parties (party_id));
diesel::joinable!(party_members -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    external_accounts,
    parties,
    party_codes,
    party_members,
    users,
);
