use crate::format_response::PYQError;
use axum::{
    extract::{Query, State},
    Json,
};
use data_structures::{
    metadata::{Friends, Posts},
    response::AllPostData,
};
use db::{sqlite, SqlitePool};
use rand::seq::SliceRandom;
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct AllQueryParams {
    start: Option<usize>,
    end: Option<usize>,
    #[serde(rename(deserialize = "rule"))]
    sort_rule: Option<String>,
}

pub async fn get_all(
    State(pool): State<SqlitePool>,
    Query(params): Query<AllQueryParams>,
) -> Result<Json<AllPostData>, PYQError> {
    // println!("{:?}",params);
    let posts = match sqlite::select_all_from_posts(
        &pool,
        params.start.unwrap_or(0),
        params.end.unwrap_or(0),
        &params.sort_rule.unwrap_or(String::from("updated")),
    )
    .await
    {
        Ok(v) => v,
        Err(e) => return Err(PYQError::QueryDataBaseError(e.to_string())),
    };

    let last_updated_time = match sqlite::select_latest_time_from_posts(&pool).await {
        Ok(v) => v,
        Err(e) => return Err(PYQError::QueryDataBaseError(e.to_string())),
    };

    let friends = match sqlite::select_all_from_friends(&pool).await {
        Ok(v) => v,
        Err(e) => return Err(PYQError::QueryDataBaseError(e.to_string())),
    };
    let friends_num = friends.len();
    let mut active_num = 0;
    let mut lost_num = 0;
    for friend in friends {
        if friend.error {
            lost_num += 1;
        } else {
            active_num += 1;
        }
    }
    let data = AllPostData::new(
        friends_num,
        active_num,
        lost_num,
        posts.len(),
        last_updated_time,
        posts,
        params.start.unwrap_or(0),
    );
    Ok(Json(data))
}

pub async fn get_friend(State(pool): State<SqlitePool>) -> Result<Json<Vec<Friends>>, PYQError> {
    let friends = match sqlite::select_all_from_friends(&pool).await {
        Ok(v) => v,
        Err(e) => return Err(PYQError::QueryDataBaseError(e.to_string())),
    };

    Ok(Json(friends))
}

#[derive(Debug, Deserialize)]
pub struct RandomQueryParams {
    #[serde(default)]
    num: Option<usize>,
}

pub async fn get_randomfriend(
    State(pool): State<SqlitePool>,
    Query(params): Query<RandomQueryParams>,
) -> Result<Json<Vec<Friends>>, PYQError> {
    let friends = match sqlite::select_all_from_friends(&pool).await {
        Ok(v) => v,
        Err(e) => return Err(PYQError::QueryDataBaseError(e.to_string())),
    };
    // println!("{:?}",params);
    let rng = &mut rand::thread_rng();
    let result: Vec<Friends> = friends
        .choose_multiple(rng, params.num.unwrap_or(1))
        .cloned()
        .collect();
    Ok(Json(result))
}

pub async fn get_randompost(
    State(pool): State<SqlitePool>,
    Query(params): Query<RandomQueryParams>,
) -> Result<Json<Vec<Posts>>, PYQError> {
    let posts = match sqlite::select_all_from_posts(&pool, 0, 0, "updated").await {
        Ok(v) => v,
        Err(e) => return Err(PYQError::QueryDataBaseError(e.to_string())),
    };
    let rng = &mut rand::thread_rng();
    let result: Vec<Posts> = posts
        .choose_multiple(rng, params.num.unwrap_or(1))
        .cloned()
        .collect();
    Ok(Json(result))
}
