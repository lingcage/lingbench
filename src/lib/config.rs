// SPDX-FileCopyrightText: Copyright (c) 2026 LingCage. All rights reserved.
//
// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_workdir")]
    pub workdir: PathBuf,
    pub kernel: KernelConfig,
    pub rootfs: RootfsConfig,
}

#[derive(Debug, Deserialize)]
pub struct KernelConfig {
    pub version: String,
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default)]
    pub sha256: Option<String>,
    #[serde(default)]
    pub config_fragment: Option<PathBuf>,
    #[serde(default = "default_arch")]
    pub arch: String,
    #[serde(default)]
    pub extra_make_args: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RootfsConfig {
    #[serde(default = "default_containerfile")]
    pub containerfile: PathBuf,
    #[serde(default = "default_size_mib")]
    pub size_mib: u64,
    #[serde(default = "default_builder")]
    pub builder: String,
    #[serde(default)]
    pub formats: Vec<RootfsFormat>,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RootfsFormat {
    Tar,
    Ext4,
    Cpio,
}

fn default_workdir() -> PathBuf {
    PathBuf::from("build")
}
fn default_arch() -> String {
    "x86_64".into()
}
fn default_containerfile() -> PathBuf {
    PathBuf::from("guest/rootfs/Containerfile")
}
fn default_size_mib() -> u64 {
    512
}
fn default_builder() -> String {
    "podman".into()
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let s = std::fs::read_to_string(path)
            .with_context(|| format!("reading config {}", path.display()))?;
        toml::from_str(&s).with_context(|| format!("parsing config {}", path.display()))
    }
    pub fn kernel_dir(&self) -> PathBuf {
        self.workdir.join("kernel")
    }
    pub fn rootfs_dir(&self) -> PathBuf {
        self.workdir.join("rootfs")
    }
    pub fn download_dir(&self) -> PathBuf {
        self.workdir.join("downloads")
    }
}
