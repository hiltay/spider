use crate::format_response::PYQError;
use axum::{
    extract::{Query, State},
    Json,
};
use data_structures::response::AllPostData;
use db::{mysql, MySqlPool};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AllQueryParams {
    #[serde(default)]
    start: Option<usize>,
    end: Option<usize>,
    #[serde(rename(deserialize = "rule"))]
    sort_rule: Option<String>,
}

pub async fn get_all(
    State(pool): State<MySqlPool>,
    Query(params): Query<AllQueryParams>,
) -> Result<Json<AllPostData>, PYQError> {
    // println!("{:?}",params);
    let posts = match mysql::select_all_from_posts(
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

    let last_updated_time = match mysql::select_latest_time_from_posts(&pool).await {
        Ok(v) => v,
        Err(e) => return Err(PYQError::QueryDataBaseError(e.to_string())),
    };

    let friends = match mysql::select_all_from_friends(&pool).await {
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
