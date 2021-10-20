#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unreachable_code)]
#![allow(clippy::type_complexity)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::upper_case_acronyms)]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]

#[macro_use]
extern crate log;
#[macro_use]
extern crate anyhow;

pub type Result<T> = anyhow::Result<T>;

mod bucket;
mod client;
mod config;
mod conn;
mod error;
mod request;
mod types;
mod util;
mod version;

pub use bucket::Bucket;
pub use client::Client;
pub use version::VERSION;
