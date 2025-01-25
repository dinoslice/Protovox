use std::{io, fs, fs::OpenOptions};
use std::str::FromStr;
use chrono::Utc;
use tracing_subscriber::{fmt, fmt::time::ChronoUtc, EnvFilter, Registry};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use engine::application;

fn init_tracing() -> io::Result<()> {
    let file_name = format!("logs/{}.log", Utc::now().format("%Y-%m-%d_%H-%M-%S"));
    fs::create_dir_all("logs")?;

    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_name)?;

    // TODO: use custom formatter
    let console_fmt = fmt::format()
        .compact()
        .with_timer(ChronoUtc::new(String::from("[%H:%M:%S]")));

    let file_fmt = fmt::format()
        .compact()
        .with_timer(ChronoUtc::new(String::from("[%H:%M:%S%.3f]")))
        // .with_source_location(true)
        .with_ansi(false);

    let console_layer = fmt::Layer::default()
        .event_format(console_fmt)
        .with_writer(io::stdout);

    let file_layer = fmt::Layer::default()
        .event_format(file_fmt)
        .with_writer(move || log_file.try_clone().expect("failed to clone log file handle"));

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or(
            EnvFilter::from_str("warn,engine=debug")
                .expect("failed to set default env filter")
        );

    let subscriber = Registry::default()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer);

    subscriber.try_init().expect("logger has already been set");

    Ok(())
}

fn main() {
    init_tracing().expect("tracing initialized");

    application::run_game();
}
