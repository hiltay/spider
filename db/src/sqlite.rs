use data_structures::metadata;
use sqlx::{
    query, sqlite::SqliteConnectOptions, sqlite::SqlitePool, sqlite::SqlitePoolOptions, Error,
    Execute, QueryBuilder, Sqlite,
};
use std::{future::Future, path::Path};
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

pub async fn insert_friend_table(
    friends: &metadata::Friends,
    pool: &SqlitePool,
) -> Result<(), Error> {
    let sql = "INSERT INTO friends (name, link, avatar, error,createAt) VALUES (?, ?, ?, ?, ?)";
    let q = query(sql)
        .bind(&friends.name)
        .bind(&friends.link)
        .bind(&friends.avatar)
        .bind(&friends.error)
        .bind(&friends.createAt);
    // println!("sql: {},{:?}",q.sql(),q.take_arguments());
    q.execute(pool).await?;
    Ok(())
}

pub async fn bulk_insert_post_table(
    tuples: impl Iterator<Item = metadata::Posts>,
    pool: &SqlitePool,
) -> Result<(), Error> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
        // Note the trailing space; most calls to `QueryBuilder` don't automatically insert
        // spaces as that might interfere with identifiers or quoted strings where exact
        // values may matter.
        "INSERT INTO posts (title, author, link, avatar ,rule,created,updated,createAt) ",
    );

    query_builder.push_values(tuples, |mut b, post| {
        // If you wanted to bind these by-reference instead of by-value,
        // you'd need an iterator that yields references that live as long as `query_builder`,
        // e.g. collect it to a `Vec` first.
        b.push_bind(post.meta.title)
            .push_bind(post.author)
            .push_bind(post.meta.link)
            .push_bind(post.avatar)
            .push_bind(post.meta.rule)
            .push_bind(post.meta.created)
            .push_bind(post.meta.updated)
            .push_bind(post.createAt);
    });
    let query = query_builder.build();
    query.execute(pool).await?;
    Ok(())
}

pub async fn bulk_insert_friend_table(
    tuples: impl Iterator<Item = metadata::Friends>,
    pool: &SqlitePool,
) -> Result<(), Error> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
        // Note the trailing space; most calls to `QueryBuilder` don't automatically insert
        // spaces as that might interfere with identifiers or quoted strings where exact
        // values may matter.
        "INSERT INTO friends (name, link, avatar, error,createAt) ",
    );

    query_builder.push_values(tuples, |mut b, friends| {
        // If you wanted to bind these by-reference instead of by-value,
        // you'd need an iterator that yields references that live as long as `query_builder`,
        // e.g. collect it to a `Vec` first.
        b.push_bind(friends.name)
            .push_bind(friends.link)
            .push_bind(friends.avatar)
            .push_bind(friends.createAt);
    });
    let query = query_builder.build();
    query.execute(pool).await?;
    Ok(())
}
