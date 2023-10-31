use std::{future::Future, path::Path};

use sqlx::{sqlite::SqliteConnectOptions, sqlite::SqlitePool, sqlite::SqlitePoolOptions, Error};
// use sqlx::mysql::MySqlPoolOptions;
// etc.

pub async fn connect_sqlite_dbpool(
    filename: impl AsRef<Path>,
) -> impl Future<Output = Result<SqlitePool, Error>> {
    let options = SqliteConnectOptions::new()
        .filename(filename)
        .create_if_missing(true);
    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
}
