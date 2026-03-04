// SPDX-FileCopyrightText: 2026 LingCage
//
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use lingbench::{Config, kernel, rootfs};

#[derive(Parser)]
#[command(name = "lingbench", about = "VMM test framework", version)]
pub struct Cli {
    #[arg(long, short, default_value = "lingbench.toml", global = true)]
    config: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Build guest artifacts
    Build {
        #[command(subcommand)]
        target: BuildTarget,
    },
    /// Remove the working directory
    Clean,
}

#[derive(Subcommand)]
enum BuildTarget {
    /// Fetch and build the guest kernel
    Kernel,
    /// Build the guest rootfs from a Containerfile
    Rootfs,
    /// Build both kernel and rootfs
    All,
}

impl Cli {
    pub fn run(self) -> Result<()> {
        let cfg = Config::load(&self.config)?;
        match self.command {
            Command::Build { target } => match target {
                BuildTarget::Kernel => kernel::build(&cfg)?,
                BuildTarget::Rootfs => rootfs::build(&cfg)?,
                BuildTarget::All => {
                    kernel::build(&cfg)?;
                    rootfs::build(&cfg)?;
                }
            },
            Command::Clean => {
                if cfg.workdir.exists() {
                    std::fs::remove_dir_all(&cfg.workdir)?;
                    tracing::info!("removed {}", cfg.workdir.display());
                }
            }
        }
        Ok(())
    }
}
