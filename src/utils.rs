use color_eyre::Result;
use tracing_error::ErrorLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn init_tracing() -> Result<()> {
    let show_colors = std::env::var("NO_COLOR").is_err();

    #[cfg(debug_assertions)]
    let fmt_layer = tracing_subscriber::fmt::layer()
        .compact()
        .without_time()
        .with_writer(std::io::stdout)
        .with_ansi(show_colors);

    #[cfg(not(debug_assertions))]
    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .without_time()
        .with_writer(std::io::stdout)
        .with_ansi(show_colors);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,crabbot=debug,tower_http=debug".into()),
        )
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .try_init()?;

    Ok(())
}
