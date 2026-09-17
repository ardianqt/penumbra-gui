use env_logger::Env;

pub fn init() {
    env_logger::Builder::from_env(
        Env::default().filter_or("SONORA_LOG", "warn,toolkit=debug,ui=debug"),
    )
    .format_timestamp_micros()
    .init();
}