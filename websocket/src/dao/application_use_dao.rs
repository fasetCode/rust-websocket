use crate::domain::application_use::ApplicationUse;
use sqlx::PgPool;

pub async fn find_app_id(pool: &PgPool, app_name: &str) -> Result<ApplicationUse, sqlx::Error> {
    let app_use = sqlx::query_as(
        r#"
        select id,app_id,token,app_auth_url,app_callback_message from application_use where app_id = $1
        "#,
    )
    .bind(app_name)
    .fetch_one(pool)
    .await?;
    Ok(app_use)
}
