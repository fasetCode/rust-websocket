use crate::dao::application_use_dao::find_app_id;
use crate::domain::application_use::ApplicationUse;
use actix::fut::ok;
use sqlx::PgPool;

pub(crate) async fn get_app_id(
    pool: &PgPool,
    app_id: String,
) -> Result<ApplicationUse, sqlx::Error> {
    match find_app_id(pool, &app_id).await {
        Ok(app_use) => Ok(app_use),
        Err(e) => Err(e),
    }
}
