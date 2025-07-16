use data_structures::metadata;
use sqlx::{
    Error, MySql, QueryBuilder, Row, mysql::MySqlPool, mysql::MySqlPoolOptions, query, query_as,
};

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
        .bind(&post.created_at);
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
        .bind(friends.error)
        .bind(&friends.created_at);
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
            .push_bind(post.created_at);
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
            .push_bind(friends.created_at);
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

pub async fn truncate_friend_table(pool: &MySqlPool) -> Result<(), Error> {
    let sql = "TRUNCATE table friends";
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
    pool: &MySqlPool,
    start: usize,
    end: usize,
    sort_rule: &str,
) -> Result<Vec<metadata::Posts>, Error> {
    let sql = if start == 0 && end == 0 {
        format!("SELECT * FROM posts ORDER BY {sort_rule} DESC")
    } else {
        format!(
            "
        SELECT * FROM posts
        ORDER BY {sort_rule} DESC
        LIMIT {limit} OFFSET {start}
        ",
            limit = end - start
        )
    };
    // println!("{}",sql);
    let posts = query_as::<_, metadata::Posts>(&sql).fetch_all(pool).await?;
    Ok(posts)
}
/// 查询`posts`表中`link`包含`domain_str`的数据
///
/// 当num<0时，返回所有数据
pub async fn select_all_from_posts_with_linklike(
    pool: &MySqlPool,
    link: &str,
    num: i32,
    sort_rule: &str,
) -> Result<Vec<metadata::Posts>, Error> {
    let sql = if num >= 0 {
        format!(
            "SELECT * FROM posts WHERE link like '%{link}%' ORDER BY {sort_rule} DESC LIMIT {num}"
        )
    } else {
        format!("SELECT * FROM posts WHERE link like '%{link}%' ORDER BY {sort_rule} DESC")
    };
    // println!("{}",sql);
    let posts = query_as::<_, metadata::Posts>(&sql).fetch_all(pool).await?;
    Ok(posts)
}

/// 查询`friends`表中`link`包含`domain_str`的一条数据
pub async fn select_one_from_friends_with_linklike(
    pool: &MySqlPool,
    domain_str: &str,
) -> Result<metadata::Friends, Error> {
    let sql = format!("SELECT * from friends WHERE link like '%{domain_str}%'");
    // println!("{}", sql);

    let friend = query_as::<_, metadata::Friends>(&sql)
        .fetch_one(pool)
        .await?;
    Ok(friend)
}

/// 获取`posts`表中最近一次更新（`createdAt`最新）的时间
pub async fn select_latest_time_from_posts(pool: &MySqlPool) -> Result<String, Error> {
    let sql = "SELECT createdAt from posts ORDER BY createdAt DESC";
    let result = query(sql).fetch_one(pool).await?;
    let created_at: String = result.get("createdAt");
    Ok(created_at)
}

/// 查询`friends`表的所有数据
pub async fn select_all_from_friends(pool: &MySqlPool) -> Result<Vec<metadata::Friends>, Error> {
    let sql = String::from("SELECT * FROM friends");
    let friends = query_as::<_, metadata::Friends>(&sql)
        .fetch_all(pool)
        .await?;
    Ok(friends)
}

pub async fn delete_outdated_posts(days: usize, dbpool: &MySqlPool) -> Result<usize, Error> {
    let sql = "DELETE FROM posts WHERE DATE(updated) < DATE_SUB(CURDATE(), INTERVAL ? DAY)";
    let affected_rows = query(sql).bind(days as i64).execute(dbpool).await?;

    Ok(affected_rows.rows_affected() as usize)
}
