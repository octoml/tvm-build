use git2::build::RepoBuilder;
use thiserror::Error;
use tracing::{self, info};
use cmake;

const TVM_REPO: &'static str = "https://github.com/apache/incubator-tvm";
const DEFAULT_BRANCH: &'static str = "main";

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Git2(#[from] git2::Error),
}

#[derive(Debug)]
pub struct BuildConfig {
    pub repository: Option<String>,
    pub repository_path: Option<String>,
    pub output_path: Option<String>,
    pub branch: Option<String>,
    pub clean: bool,
}

/// Build TVM given a build configuration and a target directory.
#[tracing::instrument]
pub fn build(build_config: BuildConfig) -> Result<(), Error> {
    info!("tvm_build::build");
    let repository_url =
        build_config.repository.unwrap_or(TVM_REPO.into());

    let branch = build_config.branch.unwrap_or(DEFAULT_BRANCH.into());

    // let path = match build_config.repository_path {
    //     Some(path) => std::path::Path::new(&path),
    //     None => {
    //         let tmp_dir = TempDir::new("tvm_build")?;
    //     }
    // };
    // let path = build_config.repository_path.unwrap_or(".".into());
    // let path = std::path::Path::new(&path);
    let path = "tvm_source";
    let path = std::path::Path::new(path);

    if build_config.clean {
        std::fs::remove_dir_all(path).unwrap();
    }

    if !path.exists() {
        let mut repo_builder = RepoBuilder::new();
        repo_builder.branch(&branch);

        let repo = match repo_builder.clone(&repository_url, path) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to clone: {}", e),
        };

        for mut submodule in repo.submodules().unwrap() {
            submodule.update(true, None).unwrap();
        }
    }

    let mut cmake_config = cmake::Config::new(path);

    let target = "x86_64-apple-darwin19.5.0";

    let dst = cmake_config
        .generator("Ninja")
        .out_dir("tvm_source/build")
        .very_verbose(true)
        .target(target)
        .host("Darwin")
        .profile("Debug")
        .build();

    info!(target = target);
    // info!(dst = dst.display().to_string());

    Ok(())
}
