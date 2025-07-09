use api_dependence::{mysql::mysqlapi, sqlite::sqliteapi};
use axum::{
    Router, http,
    response::Response,
    routing::{get, post},
};
use db::{mysql, sqlite};
use logroller::{Compression, LogRollerBuilder, Rotation, RotationAge, TimeZone};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, fmt, prelude::*, registry::Registry};
fn init_tracing()->WorkerGuard {
    // TODO stdout和file同时输出，并设置不同的fmt
    // TODO 输出划分为http和core两个文件，似乎可以通过filter来实现 https://docs.rs/tracing-subscriber/latest/tracing_subscriber/layer/index.html
    let formmater_string = "%Y-%m-%d %H:%M:%S (%Z)".to_string();
    let timer = tracing_subscriber::fmt::time::ChronoLocal::new(formmater_string);
    let appender = LogRollerBuilder::new("./logs", "fcircle.log")
        .rotation(Rotation::AgeBased(RotationAge::Daily)) // Rotate daily
        .max_keep_files(7) // Keep a week's worth of logs
        .time_zone(TimeZone::Local) // Use local timezone
        .compression(Compression::Gzip) // Compress old logs
        .build()
        .unwrap();
    let (non_blocking, _guard) = tracing_appender::non_blocking(appender);
    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_timer(timer)
        .with_file(true)
        .with_line_number(true)
        .with_writer(non_blocking)
        .compact();

    let filter = EnvFilter::new("trace,tower_http=trace,sqlx::query=info");
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();
    info!("Setup tracing success");
    _guard
}

#[tokio::main]
async fn main() {
    let fc_settings = tools::get_yaml_settings("./fc_settings.yaml").unwrap();
    let _guard = init_tracing();
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers(Any);
    let service = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    let app;
    match fc_settings.database.as_str() {
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
