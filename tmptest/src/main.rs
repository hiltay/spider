use tracing::info;
use tracing_subscriber::{fmt, prelude::*};
fn main() {
    let formmater_string = "%Y-%m-%d %H:%M:%S (%Z)".to_string();
    let timer = tracing_subscriber::fmt::time::ChronoLocal::new(formmater_string);
    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_timer(timer)
        .with_file(true)
        .with_line_number(true)
        .compact();
    tracing_subscriber::registry()
        .with(fmt_layer)
        .init();
    info!("Hello, world!");
}
