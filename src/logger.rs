use chrono::Local;
use env_logger::Builder;
// use log::LevelFilter;
use std::io::Write;
use std::{env, thread};

pub fn configure_logger() {
    let mut builder = Builder::from_default_env();

    // workaround for this issue: https://github.com/seanmonstar/pretty-env-logger/issues/41
    if let Some(log_level) = env::var_os("RUST_LOG") {
        builder.parse_filters(&log_level.to_string_lossy());
    } else {
        let mut builder = Builder::from_default_env();
        builder.parse_filters("error");
    }

    builder
        .format(|buf, record| {
            writeln!(
                buf,
                "{} {:?} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S.%6f"),
                thread::current().id(),
                record.level(),
                record.args()
            )
        })
        // .filter(None, level)
        .init();
}
