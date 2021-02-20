use std::path::{Path, PathBuf};

use tracing::{self, info};
use cmake;

mod core;
mod targets;

use targets::local_target;

pub use self::core::BuildConfig;

/// Build TVM given a build configuration.
#[tracing::instrument]
pub fn build(build_config: core::BuildConfig) -> Result<core::BuildResult, core::Error> {
    info!("tvm_build::build");
    let rev = core::init_tvm_build_dir(&build_config)?;
    let source_path = rev.source_path();
    let build_path = rev.build_path();

    let target = local_target();

    // TODO(@jroesch): map this to target triple based target directory
    // should probably be target + host + profile.
    match build_config.output_path {
        None => (),
        _ => panic!("this option is currently disabled"),
    };

    if !build_path.exists() {
        std::fs::create_dir_all(build_path.clone()).unwrap();
    }

    core::build_revision(&rev, target)?;

    // info!(target = target.target_str);
    // info!(dst = dst.display().to_string());

    Ok(core::BuildResult  {
        revision: rev,
    })
}
