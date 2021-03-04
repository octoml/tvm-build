use tracing::{self, info};

mod core;
mod targets;

use targets::local_target;

pub use self::core::BuildConfig;

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

    Ok(core::BuildResult  {
        revision: rev,
    })
}
