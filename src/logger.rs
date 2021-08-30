use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;
use std::thread;

pub fn configure_logger() {
    Builder::new()
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
        .filter(None, LevelFilter::Debug)
        .init();
}
