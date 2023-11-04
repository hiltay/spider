use api_dependence::sqlite::sqliteapi;
use axum::{routing::get, Router};
use db::sqlite;
#[tokio::main]
async fn main() {
    let dbpool = sqlite::connect_sqlite_dbpool("data.db").await.unwrap();

    let app = Router::new()
        .route("/all", get(sqliteapi::get_all))
        .with_state(dbpool);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
