//! Substrate Node Demo Cli
#![warn(missing_docs)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;
mod command;
mod http_proxy;

pub use sc_cli::{error, VersionInfo};

fn main() -> Result<(), error::Error> {
	let version = VersionInfo {
		name: "Substrate LFS Demo Node",
		commit: env!("VERGEN_SHA_SHORT"),
		version: env!("CARGO_PKG_VERSION"),
		executable_name: "lfs-demo",
		author: "Anonymous",
		description: "Template Node",
		support_url: "support.anonymous.an",
		copyright_start_year: 2019,
	};

	command::run(version)
}
