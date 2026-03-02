// SPDX-FileCopyrightText: 2026 LingCage
//
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};

use crate::config::Config;
use crate::util;

pub fn build(cfg: &Config) -> Result<()> {
    let kcfg = &cfg.kernel;
    let version = &kcfg.version;
    let major = version.split('.').next().unwrap_or("6");
    let tarball_name = format!("linux-{version}.tar.xz");
    let url = kcfg.source_url.clone().unwrap_or_else(|| {
        format!("https://cdn.kernel.org/pub/linux/kernel/v{major}.x/{tarball_name}")
    });

    let download_dir = cfg.download_dir();
    let tarball = download_dir.join(&tarball_name);
    if !tarball.exists() {
        util::download(&url, &tarball)?;
    }
    if let Some(expected) = &kcfg.sha256 {
        util::verify_sha256(&tarball, expected)?;
    }

    let kernel_dir = cfg.kernel_dir();
    std::fs::create_dir_all(&kernel_dir)?;
    let src_dir = kernel_dir.join(format!("linux-{version}"));
    if !src_dir.exists() {
        tracing::info!("extracting {}", tarball.display());
        util::run(
            Command::new("tar")
                .arg("-xf")
                .arg(&tarball)
                .arg("-C")
                .arg(&kernel_dir),
        )?;
    }

    let arch = &kcfg.arch;
    util::run(
        Command::new("make")
            .current_dir(&src_dir)
            .arg(format!("ARCH={arch}"))
            .arg("defconfig"),
    )?;

    if let Some(frag) = &kcfg.config_fragment {
        let frag_abs = std::fs::canonicalize(frag)
            .with_context(|| format!("canonicalizing {}", frag.display()))?;
        util::run(
            Command::new("./scripts/kconfig/merge_config.sh")
                .current_dir(&src_dir)
                .arg("-m")
                .arg(".config")
                .arg(&frag_abs),
        )?;
        util::run(
            Command::new("make")
                .current_dir(&src_dir)
                .arg(format!("ARCH={arch}"))
                .arg("olddefconfig"),
        )?;
    }

    let jobs = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let mut make = Command::new("make");
    make.current_dir(&src_dir)
        .arg(format!("ARCH={arch}"))
        .arg(format!("-j{jobs}"))
        .arg(build_target(arch));
    for extra in &kcfg.extra_make_args {
        make.arg(extra);
    }
    util::run(&mut make)?;

    let built = src_dir.join(artifact_path(arch));
    let out = kernel_dir.join(output_name(arch));
    std::fs::copy(&built, &out)
        .with_context(|| format!("copying {} -> {}", built.display(), out.display()))?;
    tracing::info!("kernel ready: {}", out.display());
    Ok(())
}

fn build_target(arch: &str) -> &'static str {
    match arch {
        "x86_64" | "i386" => "bzImage",
        "arm64" | "aarch64" => "Image",
        _ => "vmlinux",
    }
}

fn artifact_path(arch: &str) -> PathBuf {
    match arch {
        "x86_64" | "i386" => PathBuf::from("arch/x86/boot/bzImage"),
        "arm64" | "aarch64" => PathBuf::from("arch/arm64/boot/Image"),
        _ => PathBuf::from("vmlinux"),
    }
}

fn output_name(arch: &str) -> &'static str {
    match arch {
        "x86_64" | "i386" => "bzImage",
        "arm64" | "aarch64" => "Image",
        _ => "vmlinux",
    }
}
