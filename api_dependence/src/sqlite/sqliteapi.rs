use axum::{
    extract::{Query, State},
    Json,
};
use data_structures::response::AllPostData;
use db::{sqlite, SqlitePool};
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
    State(pool): State<SqlitePool>,
    Query(params): Query<AllQueryParams>,
) -> Json<AllPostData> {
    println!("{:?}", params);
    let posts = sqlite::select_all_from_posts(
        &pool,
        params.start.unwrap_or(0),
        params.end.unwrap_or(0),
        &params.sort_rule.unwrap_or(String::from("updated")),
    )
    .await
    .expect("查询`posts`表失败，请检查：1、请求参数是否正确？2、数据库是否可以连接？3、posts表是否有数据？4、字段格式是否正确？");
    let last_updated_time = sqlite::select_latest_time_from_posts(&pool).await.expect("查询上次更新时间失败。请检查：请检查：1、请求参数是否正确？2、数据库是否可以连接？3、posts表是否有数据？4、字段格式是否正确？");

    let friends = sqlite::select_all_from_friends(&pool).await.expect("查询`friends`表失败，请检查：1、请求参数是否正确？2、数据库是否可以连接？3、friends表是否有数据？4、字段格式是否正确？");
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
    Json(data)
}
