// SPDX-FileCopyrightText: 2026 LingCage
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

mod cli;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .init();
    cli::Cli::parse().run()
}
