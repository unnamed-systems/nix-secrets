use tracing::level_filters::LevelFilter;
use tracing_error::ErrorLayer;
use tracing_subscriber::{
    Layer as _, Registry, fmt, layer::SubscriberExt as _, util::SubscriberInitExt as _,
};

pub fn init_logger() {
    let _ = color_eyre::config::HookBuilder::default()
        .display_env_section(false)
        .install();
    let is_debug = cfg!(debug_assertions);
    let default_level = if is_debug {
        LevelFilter::TRACE
    } else {
        LevelFilter::INFO
    };
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(default_level.into())
        .from_env_lossy();

    let fmt_layer = fmt::layer()
        .compact()
        .with_target(is_debug)
        .with_writer(std::io::stderr)
        .with_ansi(std::io::IsTerminal::is_terminal(&std::io::stderr()))
        .with_timer(tracing_subscriber::fmt::time::Uptime::default());

    let fmt_layer_final = if is_debug {
        fmt_layer.boxed()
    } else {
        fmt_layer.without_time().boxed()
    };

    Registry::default()
        .with(filter)
        .with(ErrorLayer::default())
        .with(fmt_layer_final)
        .init();

    trace!("Initialized tracing");
}
