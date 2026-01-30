use sqlx::PgPool;

pub struct DbState {
    pub db: PgPool,
}