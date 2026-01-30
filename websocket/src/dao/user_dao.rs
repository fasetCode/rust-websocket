use crate::common::dto::PageVo;
use crate::domain::user::{User, UserCreate, UserPageListVo, UserQuery, UserUpdate};
use sqlx::{PgPool, Postgres, QueryBuilder};

pub async fn find_page(
    pool: &PgPool,
    query: UserQuery,
) -> Result<PageVo<UserPageListVo>, sqlx::Error> {
    let (total,): (i64,) = sqlx::query_as(r#"select count(*) from "user""#)
        .fetch_one(pool)
        .await?;

    let mut qb: QueryBuilder<Postgres> =
        QueryBuilder::new("select id,username,nickname,email,phone from \"user\"");
    qb.push(" limit ");
    qb.push_bind(query.page_size);
    qb.push(" offset ");
    qb.push_bind((query.page - 1) * query.page_size);
    let users = qb
        .build_query_as::<UserPageListVo>()
        .fetch_all(pool)
        .await?;

    Ok(PageVo::new(total, users))
}

pub async fn create_user(pool: &PgPool, user: UserCreate) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        insert into "user" (username, password, nickname, email, phone)
        values ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(user.username)
    .bind(user.password)
    .bind(user.nickname)
    .bind(user.email)
    .bind(user.phone)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

pub async fn get_username(pool: &PgPool, username: &str) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as(
        r#"
        select id,password,username,nickname,email,phone from "user" where username = $1
        "#,
    )
    .bind(username)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn get_count_username(pool: &PgPool, username: &str) -> Result<u64, sqlx::Error> {
    let count: (i64,) = sqlx::query_as(
        r#"
            SELECT COUNT(*) FROM "user" WHERE username = $1
        "#,
    )
    .bind(username)
    .fetch_one(pool)
    .await?;

    Ok(count.0 as u64)
}

pub async fn update_user(p0: &PgPool, p1: UserUpdate) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        update "user" set username = $1, nickname = $2, email = $3, phone = $4 where id = $5
        "#,
    )
    .bind(p1.username)
    .bind(p1.nickname)
    .bind(p1.email)
    .bind(p1.phone)
    .bind(p1.id)
    .execute(p0)
    .await?;
    Ok(result.rows_affected())
}

pub async fn delete_user(p0: &PgPool, p1: i32) ->Result<u64, sqlx::Error>{

    let result = sqlx::query(
        r#"
        delete from "user" where id = $1
        "#,
    )
    .bind(p1)
    .execute(p0)
    .await?;
    Ok(result.rows_affected())
}