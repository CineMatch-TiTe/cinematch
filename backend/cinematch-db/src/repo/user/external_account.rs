//! External account database operations

use crate::Database;
use crate::DbError;
use crate::DbResult;
use crate::models::{ExternalAccount, NewExternalAccount};
use crate::schema::external_accounts;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

impl Database {
    /// Link an external account to a user.
    pub(crate) async fn link_external_account(
        &self,
        new_account: NewExternalAccount<'_>,
    ) -> DbResult<ExternalAccount> {
        let mut conn = self.conn().await?;
        diesel::insert_into(external_accounts::table)
            .values(&new_account)
            .returning(ExternalAccount::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(DbError::from)
    }

    /// Unlink an external account from a user.
    pub(crate) async fn unlink_external_account(
        &self,
        user_id: Uuid,
        provider: crate::models::AuthProvider,
    ) -> DbResult<usize> {
        let mut conn = self.conn().await?;
        diesel::delete(external_accounts::table)
            .filter(external_accounts::user_id.eq(user_id))
            .filter(external_accounts::provider.eq(provider))
            .execute(&mut conn)
            .await
            .map_err(DbError::from)
    }

    /// Get an external account by provider and provider's user ID.
    pub(crate) async fn get_external_account_by_id(
        &self,
        provider: crate::models::AuthProvider,
        provider_user_id: &str,
    ) -> DbResult<Option<ExternalAccount>> {
        let mut conn = self.conn().await?;
        external_accounts::table
            .filter(external_accounts::provider.eq(provider))
            .filter(external_accounts::provider_user_id.eq(provider_user_id))
            .select(ExternalAccount::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(DbError::from)
    }

    /// Get all external accounts for a user.
    pub(crate) async fn get_user_external_accounts(
        &self,
        user_id: Uuid,
    ) -> DbResult<Vec<ExternalAccount>> {
        let mut conn = self.conn().await?;
        external_accounts::table
            .filter(external_accounts::user_id.eq(user_id))
            .select(ExternalAccount::as_select())
            .load(&mut conn)
            .await
            .map_err(DbError::from)
    }
}
