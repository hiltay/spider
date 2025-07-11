use tracing::info;
use db::sqlite;
use tracing_subscriber::prelude::*;


#[tokio::main]
async fn main() {
    let _guard =tools::init_tracing("tmptest", Some("trace"));
    let dbpool = sqlite::connect_sqlite_dbpool("data.db").await.unwrap();
    match sqlx::migrate!("../db/schema/sqlite").run(&dbpool).await {
        Ok(()) => (),
        Err(e) => {
            info!("{}", e);
            return;
        }
    };
    
    sqlite::delete_outdated_posts(1, &dbpool).await.unwrap();
}
