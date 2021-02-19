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
    let repo_path = core::init_tvm_build_dir(&build_config)?;

    let mut cmake_config = cmake::Config::new(repo_path.clone());

    let target = local_target();

    // TODO(@jroesch): map this to target triple based target directory
    // should probably be target + host + profile.
    let build_path = match build_config.output_path {
        None => repo_path.join("..").join("build"),
        _ => panic!(),
    };

    if !build_path.exists() {
        std::fs::create_dir_all(build_path.clone()).unwrap();
    }

    let config = cmake_config
        .generator("Unix Makefiles")
        .out_dir(build_path.clone())
        .very_verbose(true)
        .target(&target.target_str)
        .host(&target.host)
        .profile("Debug");

    // M1 only config
    config.define("CMAKE_OSX_ARCHITECTURES", "arm64");

    config
        .build();

    // info!(target = target.target_str);
    // info!(dst = dst.display().to_string());

    Ok(core::BuildResult  {
        revision_path: build_path,
    })
}
