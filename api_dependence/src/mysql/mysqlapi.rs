use crate::format_response::PYQError;
use axum::{
    extract::{Query, State},
    Json,
};
use data_structures::{
    metadata::{Friends, Posts},
    response::{AllPostData, AllPostDataSomeFriend},
};
use db::{mysql, MySqlPool};
use rand::seq::SliceRandom;
use serde::Deserialize;
use url::Url;

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

pub async fn get_friend(State(pool): State<MySqlPool>) -> Result<Json<Vec<Friends>>, PYQError> {
    let friends = match mysql::select_all_from_friends(&pool).await {
        Ok(v) => v,
        Err(e) => return Err(PYQError::QueryDataBaseError(e.to_string())),
    };

    Ok(Json(friends))
}

#[derive(Debug, Deserialize)]
pub struct PostParams {
    link: Option<String>,
    num: Option<i32>,
    #[serde(rename(deserialize = "rule"))]
    sort_rule: Option<String>,
}

pub async fn get_post(
    State(pool): State<MySqlPool>,
    Query(params): Query<PostParams>,
) -> Result<Json<AllPostDataSomeFriend>, PYQError> {
    let friend = match params.link {
        Some(link) => {
            let domain_str = match Url::parse(&link) {
                Ok(v) => match v.host_str() {
                    Some(host) => host.to_string(),
                    None => return Err(PYQError::QueryParamsError(String::from("无法解析出host"))),
                },
                Err(e) => return Err(PYQError::QueryParamsError(e.to_string())),
            };
            // println!("{}", domain_str);

            match mysql::select_one_from_friends_with_linklike(&pool, &domain_str).await {
                Ok(v) => v,
                Err(e) => return Err(PYQError::QueryDataBaseError(e.to_string())),
            }
        }
        None => {
            /// 没有提供link，则随机获取一个friend
            let friends = match mysql::select_all_from_friends(&pool).await {
                Ok(v) => v,
                Err(e) => return Err(PYQError::QueryDataBaseError(e.to_string())),
            };
            let rng = &mut rand::thread_rng();
            let friend = match friends.choose(rng).cloned() {
                Some(f) => f,
                None => {
                    return Err(PYQError::QueryDataBaseError(String::from(
                        "friends表数据为空",
                    )))
                }
            };
            friend
        }
    };
    let posts = match mysql::select_all_from_posts_with_linklike(
        &pool,
        &friend.link,
        params.num.unwrap_or(-1),
        &params.sort_rule.unwrap_or(String::from("created")),
    )
    .await
    {
        Ok(v) => v,
        Err(e) => return Err(PYQError::QueryDataBaseError(e.to_string())),
    };
    let data = AllPostDataSomeFriend::new(
        friend.name,
        friend.link,
        friend.avatar,
        posts.len(),
        posts,
        0,
    );
    Ok(Json(data))
}

#[derive(Debug, Deserialize)]
pub struct RandomQueryParams {
    num: Option<usize>,
}
pub async fn get_randomfriend(
    State(pool): State<MySqlPool>,
    Query(params): Query<RandomQueryParams>,
) -> Result<Json<Vec<Friends>>, PYQError> {
    let friends = match mysql::select_all_from_friends(&pool).await {
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
    State(pool): State<MySqlPool>,
    Query(params): Query<RandomQueryParams>,
) -> Result<Json<Vec<Posts>>, PYQError> {
    let posts = match mysql::select_all_from_posts(&pool, 0, 0, "updated").await {
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
