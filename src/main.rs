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
    clippy::redundant_else,
    clippy::shadow_reuse,
    clippy::shadow_same,
    clippy::shadow_unrelated,
    clippy::unreachable,
    clippy::unwrap_used, // TODO: Resolve later
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

fn main() {
    Application::new().run();
}
