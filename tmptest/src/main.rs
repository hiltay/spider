
use data_structures::response::AllPostData;
use db::{SqlitePool, sqlite};
use tracing::info;
use serde_json::json;
use std::fs::File;
use std::io;



#[tokio::main]
async fn main() {
    let _guard = tools::init_tracing("tmptest", Some("trace"));
    let dbpool = sqlite::connect_sqlite_dbpool("data.db").await.unwrap();
    match sqlx::migrate!("../db/schema/sqlite").run(&dbpool).await {
        Ok(()) => (),
        Err(e) => {
            info!("{}", e);
            return;
        }
    };

    // 调用get_all将数据写入json文件
    if let Err(e) = get_all(dbpool.clone()).await {
        info!("写入JSON数据失败: {}", e);
    }
}
