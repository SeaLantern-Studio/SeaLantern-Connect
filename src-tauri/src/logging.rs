use std::fmt;

use tracing::{Event, Level, Subscriber};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::FmtContext;
use tracing_subscriber::fmt::format::{FormatEvent, FormatFields, Writer};
use tracing_subscriber::registry::LookupSpan;

const LOG_LEVEL_ENV: &str = "RUST_LOG";

struct EventFormatter;

impl<S, N> FormatEvent<S, N> for EventFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        context: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let (label, color) = match *event.metadata().level() {
            Level::ERROR => ("ERROR", 31),
            Level::WARN => ("WARN", 33),
            Level::INFO => ("INFO", 32),
            _ => ("DEBUG", 34),
        };
        write!(writer, "\x1b[{color}m[{label}]\x1b[0m ")?;
        context
            .field_format()
            .format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}

pub(crate) fn init() {
    let level = std::env::var(LOG_LEVEL_ENV)
        .map(|value| parse_level(&value))
        .unwrap_or("info");
    let filter = EnvFilter::new(format!("sealantern_connect={level}"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .event_format(EventFormatter)
        .try_init();
}

fn parse_level(value: &str) -> &'static str {
    match value.trim().to_ascii_lowercase().as_str() {
        "debug" => "debug",
        "warn" => "warn",
        "error" => "error",
        _ => "info",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_supported_log_levels() {
        assert_eq!(parse_level("debug"), "debug");
        assert_eq!(parse_level(" INFO "), "info");
        assert_eq!(parse_level("Warn"), "warn");
        assert_eq!(parse_level("ERROR"), "error");
        assert_eq!(parse_level("trace"), "info");
    }
}
