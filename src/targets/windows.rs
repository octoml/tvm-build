use crate::core::{BuildConfig, BuildResult, Error};

/// Build TVM given a build configuration and a target directory.
#[tracing::instrument]
pub fn build(build_config: BuildConfig) -> Result<BuildResult, Error> {
    todo!()
}
