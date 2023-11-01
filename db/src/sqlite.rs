use data_structures::metadata;
use std::borrow::BorrowMut;
use std::{future::Future, path::Path};
use sqlx::Execute;
use sqlx::{
    query, sqlite::SqliteConnectOptions, sqlite::SqlitePool, sqlite::SqlitePoolOptions, Error,
};
// use sqlx::mysql::MySqlPoolOptions;
// etc.

pub async fn connect_sqlite_dbpool(filename: impl AsRef<Path>) -> Result<SqlitePool, Error> {
    let options = SqliteConnectOptions::new()
        .filename(filename)
        .create_if_missing(true);
    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
}

pub async fn insert_post_table(post: &metadata::Posts, pool: &SqlitePool) -> Result<(), Error> {
    let sql = "INSERT INTO posts 
    (title, author, link, avatar ,rule,created,updated,createAt)
     VALUES (?, ?, ?,?, ?,?, ?, ?)";
    let q = query(sql)
        .bind(&post.meta.title)
        .bind(&post.author)
        .bind(&post.meta.link)
        .bind(&post.avatar)
        .bind(&post.meta.rule)
        .bind(&post.meta.created)
        .bind(&post.meta.updated)
        .bind(&post.createAt);
    // println!("sql: {},{:?}",q.sql(),q.take_arguments());
    q.execute(pool).await?;
    Ok(())
}
