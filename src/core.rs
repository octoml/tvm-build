use std::path::{Path, PathBuf};

use git2::build::RepoBuilder;
use thiserror::Error;
use tracing::{self, info};
use cmake;
use dirs;

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
    pub verbose: bool,
    pub clean: bool,
}

impl std::default::Default for BuildConfig  {
    fn default() -> BuildConfig {
        BuildConfig {
            repository: None,
            repository_path: None,
            output_path: None,
            branch: None,
            verbose: false,
            clean: false,
        }
    }
}

pub struct BuildResult {
    pub python_libraries: PathBuf,
    pub tvm_runtime_shared_library: PathBuf,
    pub tvm_compiler_shared_library: PathBuf,
}

pub fn make_target_directory(output_path: &Path) -> std::io::Result<()> {
    Ok(())
}

// TODO: split per revision
pub fn init_tvm_build_dir(build_config: &BuildConfig) -> Result<PathBuf, Error> {
    info!("tvm_build::build");
    let repository_url =
        build_config.repository.clone().unwrap_or(TVM_REPO.into());

    let branch = build_config.branch.clone().unwrap_or(DEFAULT_BRANCH.into());

    let repo_path: PathBuf = match &build_config.repository_path {
        Some(path) => std::path::Path::new(&path).into(),
        // todo: check that the provided path exists
        None => {
            let mut home_dir = dirs::home_dir().expect("requires a home directory");
            home_dir = home_dir.join(".tvm_build");
            home_dir = home_dir.join(&branch);
            home_dir = home_dir.join("source");
            home_dir
        }
    };

    // If a user specifies the repository directory we assume we
    // don't own it and won't clean it.
    if build_config.clean && build_config.repository_path.is_none() {
        std::fs::remove_dir_all(&repo_path).unwrap();
    }

    if !repo_path.exists() {
        let mut repo_builder = RepoBuilder::new();
        repo_builder.branch(&branch);

        let repo = match repo_builder.clone(&repository_url, &repo_path) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to clone: {}", e),
        };

        for mut submodule in repo.submodules().unwrap() {
            submodule.update(true, None).unwrap();
        }
    }

    Ok(repo_path)
}
