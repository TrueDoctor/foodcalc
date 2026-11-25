use sqlx::PgPool;
use std::sync::Arc;
use time::OffsetDateTime;
use crate::entities::user::{CreateUser, User};
use crate::exports::{default_pdf_exporter, PdfExporter};
use crate::ops::FoodLib;
use crate::types::ids::UserId;
use crate::{AppCtx, Channel, Result};

#[derive(Clone)]
pub struct AppState {
    pub(crate) db: FoodLib,
    pub pdf: Arc<dyn PdfExporter>,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self {
            db: FoodLib::new(pool),
            pdf: default_pdf_exporter(),
        }
    }

    pub fn from_pool(pool: Arc<PgPool>) -> Self {
        Self {
            db: FoodLib::from_shared(pool),
            pdf: default_pdf_exporter(),
        }
    }

    pub(crate) fn db(&self) -> &FoodLib {
        &self.db
    }

    pub fn with_pdf_exporter(mut self, pdf: Arc<dyn PdfExporter>) -> Self {
        self.pdf = pdf;
        self
    }

    pub fn pdf(&self) -> Arc<dyn PdfExporter> {
        Arc::clone(&self.pdf)
    }

    pub async fn build_ctx(&self, user: User, channel: Channel) -> Result<AppCtx> {
        let groups = self.db.users().get_user_groups(user.id).await?;
        let group_ids = groups.into_iter().map(|g| g.id).collect();
        Ok(AppCtx::new(user, group_ids, channel, self.db.pool_arc()))
    }

    pub fn status_api_ctx(&self) -> AppCtx {
        AppCtx::new(
            User {
                id: UserId(-1),
                username: "status-api".into(),
                email: String::new(),
                password_hash: String::new(),
                is_admin: false,
                created_at: OffsetDateTime::now_utc(),
            },
            Vec::new(),
            Channel::StatusApi,
            self.db.pool_arc(),
        )
    }

    // Auth helpers
    pub async fn get_user_by_login(&self, identifier: &str) -> Option<User> {
        self.db
            .users()
            .get_user_by_string_reference(identifier.to_string())
            .await
    }

    pub async fn get_user(&self, user_id: UserId) -> Result<User> {
        self.db.users().get_user(user_id).await
    }

    pub async fn register_user(&self, new_user: CreateUser) -> Result<User> {
        self.db
            .users()
            .create_user_with_personal_group(new_user)
            .await
            .map(|(user, _group)| user)
    }

    pub async fn update_user_password(
        &self,
        user_id: UserId,
        new_password: String,
    ) -> Result<User> {
        self.db.users().update_password(user_id, new_password).await
    }
}
