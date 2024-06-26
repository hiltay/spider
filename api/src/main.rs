use api_dependence::{mysql::mysqlapi, sqlite::sqliteapi};
use axum::{
    http,
    routing::{get, post},
    Router,
};
use db::{mysql, sqlite};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
#[tokio::main]
async fn main() {
    let fc_settings = tools::get_yaml_settings("./fc_settings.yaml").unwrap();
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers(Any);
    let service = ServiceBuilder::new().layer(cors);

    let app;
    match fc_settings.DATABASE.as_str() {
        "sqlite" => {
            let dbpool = sqlite::connect_sqlite_dbpool("data.db").await.unwrap();
            app = Router::new()
                .route("/all", get(sqliteapi::get_all))
                .route("/friend", get(sqliteapi::get_friend))
                .route("/post", get(sqliteapi::get_post))
                .route("/randomfriend", get(sqliteapi::get_randomfriend))
                .route("/randompost", get(sqliteapi::get_randompost))
                .route("/login", post(sqliteapi::login))
                .route("/login_with_token", get(sqliteapi::login_with_token))
                .with_state(dbpool)
                .layer(service);
        }
        "mysql" => {
            // get mysql conn pool
            let mysqlconnstr = tools::load_mysql_conn_env().unwrap();
            let dbpool = mysql::connect_mysql_dbpool(&mysqlconnstr).await.unwrap();
            app = Router::new()
                .route("/all", get(mysqlapi::get_all))
                .route("/friend", get(mysqlapi::get_friend))
                .route("/post", get(mysqlapi::get_post))
                .route("/randomfriend", get(mysqlapi::get_randomfriend))
                .route("/randompost", get(mysqlapi::get_randompost))
                .with_state(dbpool)
                .layer(service);
        }
        // "mongodb" => {}
        _ => return,
    }
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
