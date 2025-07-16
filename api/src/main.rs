use api_dependence::{mysql::mysqlapi, sqlite::sqliteapi};
use axum::{Router, routing::get};
use db::{mysql, sqlite};
use tools::init_tracing;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::error;

// 创建 SQLite 应用
pub async fn create_sqlite_app(db_path: &str) -> Router {
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers(Any);
    let service = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    let dbpool = sqlite::connect_sqlite_dbpool(db_path).await.unwrap();
    Router::new()
        .route("/all", get(sqliteapi::get_all))
        .route("/friend", get(sqliteapi::get_friend))
        .route("/post", get(sqliteapi::get_post))
        .route("/randomfriend", get(sqliteapi::get_randomfriend))
        .route("/randompost", get(sqliteapi::get_randompost))
        .with_state(dbpool)
        .layer(service)
}

// 创建 MySQL 应用
pub async fn create_mysql_app(conn_str: &str) -> Router {
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers(Any);
    let service = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    let dbpool = mysql::connect_mysql_dbpool(conn_str).await.unwrap();
    Router::new()
        .route("/all", get(mysqlapi::get_all))
        .route("/friend", get(mysqlapi::get_friend))
        .route("/post", get(mysqlapi::get_post))
        .route("/randomfriend", get(mysqlapi::get_randomfriend))
        .route("/randompost", get(mysqlapi::get_randompost))
        .with_state(dbpool)
        .layer(service)
}

#[tokio::main]
async fn main() {
    let fc_settings = tools::get_yaml_settings("./fc_settings.yaml").unwrap();
    let _guard = init_tracing("api", None);

    let app = match fc_settings.database.as_str() {
        "sqlite" => create_sqlite_app("data.db").await,
        "mysql" => {
            // get mysql conn pool
            let mysqlconnstr = match tools::get_env_var("MYSQL_URI") {
                Ok(mysqlconnstr) => mysqlconnstr,
                Err(e) => {
                    error!("{}", e);
                    return;
                }
            };
            create_mysql_app(&mysqlconnstr).await
        }
        // "mongodb" => {}
        _ => return,
    };

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
