#![warn(
    clippy::all,
    clippy::cargo,
    clippy::nursery,
    clippy::pedantic,
    clippy::restriction
)]
#![allow(
    clippy::blanket_clippy_restriction_lints,
    clippy::cargo_common_metadata,
    clippy::default_numeric_fallback,
    clippy::else_if_without_else,
    clippy::exhaustive_enums,
    clippy::exhaustive_structs,
    clippy::expect_used, // TODO: Resolve later
    clippy::float_arithmetic,
    clippy::implicit_return, // would be cool to enable but excepting closures (https://github.com/rust-lang/rust-clippy/issues/6480)
    clippy::integer_arithmetic,
    clippy::match_same_arms, // would be cool to enable only for complex bodies
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::modulo_arithmetic,
    clippy::multiple_crate_versions,
    clippy::must_use_candidate,
    clippy::needless_return,
    clippy::pattern_type_mismatch,
    clippy::print_stderr,
    clippy::redundant_else,
    clippy::shadow_reuse,
    clippy::shadow_same,
    clippy::shadow_unrelated,
    clippy::unreachable,
    clippy::unwrap_used, // TODO: Resolve later
    clippy::use_debug,
    clippy::wildcard_enum_match_arm,
)]

mod application;
mod cell;
mod event;
mod field;
mod game;
mod net;
mod sapper;
mod ui;
mod utils;

use crate::application::Application;
use anyhow::Context;
use anyhow::Result;
use log::LevelFilter;
use simplelog::CombinedLogger;
use simplelog::ConfigBuilder;
use simplelog::LevelPadding;
use simplelog::WriteLogger;
use std::fs::File;

fn main() {
    if let Err(error) = init_logger() {
        eprintln!("{:?}", error);
    }

    log::info!(
        "Starting {} v{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    );
    log::info!("Logging in UTC");

    Application::new().run();

    log::info!("Terminating");
}

fn init_logger() -> Result<()> {
    fn inner() -> Result<()> {
        let file_name = format!("{}.log", env!("CARGO_PKG_NAME"));

        let file = File::create(&file_name)
            .with_context(|| format!("Failed to crate the output file ({})", file_name))?;

        let config = ConfigBuilder::default()
            .set_time_format_str("%F %T")
            .set_level_padding(LevelPadding::Right)
            .set_target_level(LevelFilter::Error)
            .set_thread_level(LevelFilter::Trace)
            .build();

        let logger = WriteLogger::new(LevelFilter::Info, config, file);

        CombinedLogger::init(vec![logger])?;
        return Ok(());
    }

    return inner().context("Failed to initialize logger");
}
