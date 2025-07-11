
use db::sqlite;
use tracing::info;



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
}
