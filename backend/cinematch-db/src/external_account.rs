//! External account database operations

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::models::{AuthProvider, ExternalAccount, NewExternalAccount, User};
use crate::schema;
use crate::{Database, DbError, DbResult};

impl Database {
    /// Link an external account to a user
    pub async fn link_external_account(
        &self,
        user_id: Uuid,
        provider: AuthProvider,
        provider_user_id: &str,
        email: Option<&str>,
        display_name: Option<&str>,
    ) -> DbResult<ExternalAccount> {
        use schema::external_accounts;

        let new_account = NewExternalAccount {
            user_id,
            provider,
            provider_user_id,
            email,
            display_name,
        };

        let mut conn = self.conn().await?;
        diesel::insert_into(external_accounts::table)
            .values(&new_account)
            .returning(ExternalAccount::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(DbError::from)
    }

    /// Get external accounts for a user
    pub async fn get_user_external_accounts(&self, uid: Uuid) -> DbResult<Vec<ExternalAccount>> {
        use schema::external_accounts::dsl::*;

        let mut conn = self.conn().await?;
        external_accounts
            .filter(user_id.eq(uid))
            .select(ExternalAccount::as_select())
            .load(&mut conn)
            .await
            .map_err(DbError::from)
    }

    /// Find user by external provider account
    pub async fn find_user_by_provider(
        &self,
        prov: AuthProvider,
        provider_id: &str,
    ) -> DbResult<Option<(User, ExternalAccount)>> {
        use schema::{external_accounts, users};

        let mut conn = self.conn().await?;
        external_accounts::table
            .inner_join(users::table)
            .filter(external_accounts::provider.eq(prov))
            .filter(external_accounts::provider_user_id.eq(provider_id))
            .select((User::as_select(), ExternalAccount::as_select()))
            .first(&mut conn)
            .await
            .optional()
            .map_err(DbError::from)
    }

    /// Unlink an external account
    pub async fn unlink_external_account(&self, account_id: Uuid) -> DbResult<usize> {
        use schema::external_accounts::dsl::*;

        let mut conn = self.conn().await?;
        diesel::delete(external_accounts.find(account_id))
            .execute(&mut conn)
            .await
            .map_err(DbError::from)
    }
}
