use data_structures::metadata;
use sqlx::{mysql::MySqlPool, mysql::MySqlPoolOptions, query, Error, MySql, QueryBuilder};

pub async fn connect_mysql_dbpool(url: &str) -> Result<MySqlPool, Error> {
    MySqlPoolOptions::new()
        .max_connections(5)
        .connect(url)
        .await
}

pub async fn insert_post_table(post: &metadata::Posts, pool: &MySqlPool) -> Result<(), Error> {
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
    pool: &MySqlPool,
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
    pool: &MySqlPool,
) -> Result<(), Error> {
    let mut query_builder: QueryBuilder<MySql> = QueryBuilder::new(
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
    pool: &MySqlPool,
) -> Result<(), Error> {
    let mut query_builder: QueryBuilder<MySql> = QueryBuilder::new(
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
    pool: &MySqlPool,
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
