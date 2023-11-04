use data_structures::metadata;
use sqlx::{
    query, query_as, sqlite::SqliteConnectOptions, sqlite::SqlitePool, sqlite::SqlitePoolOptions,
    Error, QueryBuilder, Row, Sqlite,
};
use std::path::Path;

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
    (title, author, link, avatar ,rule,created,updated,createdAt)
     VALUES (?, ?, ?,?, ?,?, ?, ?)";
    let q = query(sql)
        .bind(&post.meta.title)
        .bind(&post.author)
        .bind(&post.meta.link)
        .bind(&post.avatar)
        .bind(&post.meta.rule)
        .bind(&post.meta.created)
        .bind(&post.meta.updated)
        .bind(&post.createdAt);
    // println!("sql: {},{:?}",q.sql(),q.take_arguments());
    q.execute(pool).await?;
    Ok(())
}

pub async fn insert_friend_table(
    friends: &metadata::Friends,
    pool: &SqlitePool,
) -> Result<(), Error> {
    let sql = "INSERT INTO friends (name, link, avatar, error,createdAt) VALUES (?, ?, ?, ?, ?)";
    let q = query(sql)
        .bind(&friends.name)
        .bind(&friends.link)
        .bind(&friends.avatar)
        .bind(&friends.error)
        .bind(&friends.createdAt);
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
        "INSERT INTO posts (title, author, link, avatar ,rule,created,updated,createdAt) ",
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
            .push_bind(post.createdAt);
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
        "INSERT INTO friends (name, link, avatar, error,createdAt) ",
    );

    query_builder.push_values(tuples, |mut b, friends| {
        // If you wanted to bind these by-reference instead of by-value,
        // you'd need an iterator that yields references that live as long as `query_builder`,
        // e.g. collect it to a `Vec` first.
        b.push_bind(friends.name)
            .push_bind(friends.link)
            .push_bind(friends.avatar)
            .push_bind(friends.createdAt);
    });
    let query = query_builder.build();
    query.execute(pool).await?;
    Ok(())
}

pub async fn delete_post_table(
    tuples: impl Iterator<Item = metadata::Posts>,
    pool: &SqlitePool,
) -> Result<(), Error> {
    let sql = "DELETE FROM posts WHERE link= ? and author = ? ";
    for posts in tuples {
        query(sql)
            .bind(posts.meta.link)
            .bind(posts.author)
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn truncate_friend_table(pool: &SqlitePool) -> Result<(), Error> {
    let sql = "DELETE FROM friends";
    query(sql).execute(pool).await?;
    Ok(())
}

/// 查询`posts`表
///
/// 按照`sort_rule`排序；
///
/// 如果`start`和`end`同时为0，则查询全部；
///
/// 否则只查询`start-end`条数据，如果`start>end`，会报错
pub async fn select_all_from_posts(
    pool: &SqlitePool,
    start: usize,
    end: usize,
    sort_rule: &str,
) -> Result<Vec<metadata::Posts>, Error> {
    let sql;
    if start == 0 && end == 0 {
        sql = format!("SELECT * FROM posts ORDER BY {sort_rule} DESC");
    } else {
        sql = format!(
            "
        SELECT * FROM posts
        ORDER BY {sort_rule} DESC
        LIMIT {limit} OFFSET {start}
        ",
            limit = end - start
        );
    }
    // println!("{}",sql);
    let posts = query_as::<_, metadata::Posts>(&sql).fetch_all(pool).await?;
    Ok(posts)
}

pub async fn select_latest_time_from_posts(pool: &SqlitePool) -> Result<String, Error> {
    let sql = "SELECT createdAt from posts ORDER BY createdAt DESC";
    let result = query(sql).fetch_one(pool).await?;
    let created_at: String = result.get("createdAt");
    Ok(created_at)
}

/// 查询`friends`表的所有数据
pub async fn select_all_from_friends(pool: &SqlitePool) -> Result<Vec<metadata::Friends>, Error> {
    let sql = String::from("SELECT * FROM friends");
    let friends = query_as::<_, metadata::Friends>(&sql)
        .fetch_all(pool)
        .await?;
    Ok(friends)
}
