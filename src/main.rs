use std::fs;
use chrono::Utc;

pub fn start_tracing_logger() -> std::io::Result<()> {
    use tracing_subscriber::{fmt, Registry};
    use tracing_subscriber::fmt::time::ChronoUtc;
    use tracing_subscriber::layer::SubscriberExt;

    let start_time_formatted = Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let file_name = format!("logs/{}.log", start_time_formatted);
    fs::create_dir_all("logs")?;

    let log_file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_name)?;

    let console_fmt = fmt::format()
        .compact()
        .with_timer(ChronoUtc::new(String::from("[%H:%M:%S]")))
        .with_source_location(true)
        .with_target(false);

    let file_fmt = fmt::format()
        .compact()
        .with_timer(ChronoUtc::new(String::from("[%H:%M:%S%.3f]")))
        .with_source_location(true)
        .with_target(false)
        .with_ansi(false);

    let console_layer = fmt::Layer::default()
        .event_format(console_fmt)
        .with_writer(std::io::stdout);

    let file_layer = fmt::Layer::default()
        .event_format(file_fmt)
        .with_writer(move || log_file.try_clone().unwrap() );

    tracing::subscriber::set_global_default(
        Registry::default()
            .with(console_layer)
            .with(file_layer)
    ).unwrap();

    Ok(())
}

fn main() {
    start_tracing_logger()
        .expect("Expected logger to start");

    println!("Hello, world!");
}
