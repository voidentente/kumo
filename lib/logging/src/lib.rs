#![feature(io_error_more)]

use bevy::prelude::*;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use std::io::{Error, ErrorKind, Result};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_log::LogTracer;

/// The WorkerGuard this setup returns ensures that the log file buffer is flushed
/// before the program exits. It should be owned in such a way that the owner goes out
/// of scope at the very end of the program, as to ensure that all events are flushed.
pub fn setup() -> Result<WorkerGuard> {
    // Initialize the log tracer. This can only be done once per process.
    // On further calls, it maps to and returns an IO error, as in the LogTracer was already set.
    LogTracer::init().map_err(|e| Error::new(ErrorKind::AlreadyExists, e))?;

    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |infos| {
        let thread = std::thread::current();
        let thread_name = thread.name().unwrap_or("[unknown]");

        // Format a simple crash log event.
        error!("Thread {thread_name} {infos}");

        // The counterpart to ErrorLayer.
        error!("Error trace:");
        for line in tracing_error::SpanTrace::capture().to_string().lines() {
            error!("{line}");
        }

        hook(infos);
    }));

    // Create a tracing registry; This is in line with how bevy_log does it.
    let subscriber = tracing_subscriber::registry::Registry::default();

    // Apply a filter to the base registry.
    // RUST_LOG gets priority over bevy_log's default.
    let filter_layer = EnvFilter::try_from_default_env().unwrap_or({
        let default = bevy::log::LogPlugin::default();
        EnvFilter::new(format!("{},{}", default.level, default.filter))
    });
    let subscriber = subscriber.with(filter_layer);

    // Apply a layer that traces error spans for panics.
    let error_trace_layer = tracing_error::ErrorLayer::default();
    let subscriber = subscriber.with(error_trace_layer);

    // Apply a layer that prints to stdout.
    // Note that the stdout event log does not include process identification.
    let stdout_layer = tracing_subscriber::fmt::Layer::new()
        // `enable_ansi_support` always returns Ok(()) on non-Windows platforms.
        // On Windows, this enables ansi if supported.
        .with_ansi(enable_ansi_support::enable_ansi_support().is_ok());
    let subscriber = subscriber.with(stdout_layer);

    // Remove any old output.log file
    let exe_path = std::env::current_exe().unwrap();
    let exe_dir = exe_path.parent().unwrap();
    #[allow(unused_must_use)] {
        std::fs::remove_file(exe_dir.join("kumo.log"));
    }

    // Apply a layer that writes to file
    let writer = tracing_appender::rolling::never(exe_dir, "kumo.log");
    let (non_blocking_appender, guard) =
        tracing_appender::non_blocking(writer);
    let file_layer = tracing_subscriber::fmt::Layer::default()
        .with_ansi(false)
        .with_writer(non_blocking_appender);
    let subscriber = subscriber.with(file_layer);

    bevy::utils::tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| Error::new(ErrorKind::AlreadyExists, e))?;

    // Move ownership of the log file guard to the function caller
    Ok(guard)
}