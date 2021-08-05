use crate::core::Revision;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tracing::{self, info};

mod core;
mod targets;

use targets::local_target;

pub use self::core::{BuildConfig, UserSettings};

#[derive(Serialize, Deserialize)]
pub struct VersionConfig {
    pub tvm_python_path: PathBuf,
}

/// Build TVM given a build configuration.
#[tracing::instrument]
pub fn build(build_config: core::BuildConfig) -> Result<core::BuildResult, core::Error> {
    info!("tvm_build::build");
    let rev = build_config.get_revision()?;
    let target = local_target();

    // TODO(@jroesch): map this to target triple based target directory
    // should probably be target + host + profile.
    match build_config.output_path {
        None => (),
        _ => panic!("this option is currently disabled"),
    };

    rev.build_for(&build_config, target)?;

    // info!(target = target.target_str);
    // info!(dst = dst.display().to_string());

    Ok(core::BuildResult { revision: rev })
}

pub fn uninstall(revision: String) -> Result<(), core::Error> {
    let directory = core::tvm_build_directory().join(revision);
    std::fs::remove_dir(directory)?;
    Ok(())
}

pub fn version_config(revision: String) -> Result<VersionConfig, core::Error> {
    let rev = Revision::new(revision);
    let version = VersionConfig {
        tvm_python_path: rev.source_path().join("python").join("tvm"),
    };
    Ok(version)
}
