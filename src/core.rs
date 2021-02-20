use std::{iter::Rev, path::{Path, PathBuf}};

use git2::build::RepoBuilder;
use thiserror::Error;
use tracing::{self, info};
use cmake;
use dirs;

use super::targets::Target;

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

pub struct Revision {
    revision: String
}


// convert to lazy<T>?
fn tvm_build_directory() -> PathBuf {
    let home_dir = dirs::home_dir().expect("requires a home directory");
    home_dir.join(".tvm_build")
}

impl Revision {
    pub fn new(revision: String) -> Revision {
        Revision { revision }
    }

    pub fn path(&self) -> PathBuf {
        tvm_build_directory().join(&self.revision)
    }

    pub fn source_path(&self) -> PathBuf {
        self.path().join("source")
    }

    pub fn build_path(&self) -> PathBuf {
        self.path().join("build")
    }
}

pub struct BuildResult {
    pub revision: Revision,
}

pub fn make_target_directory(output_path: &Path) -> std::io::Result<()> {
    Ok(())
}

// TODO: split per revision
pub fn init_tvm_build_dir(build_config: &BuildConfig) -> Result<Revision, Error> {
    info!("tvm_build::build");
    let repository_url =
        build_config.repository.clone().unwrap_or(TVM_REPO.into());

    let branch = build_config.branch.clone().unwrap_or(DEFAULT_BRANCH.into());
    let revision = Revision::new(branch);

    let revision_path = match &build_config.repository_path {
        Some(path) => std::path::Path::new(&path).into(),
        // todo: check that the provided path exists
        None => {
            revision.path()
        }
    };

    // If a user specifies the repository directory we assume we
    // don't own it and won't clean it.
    if build_config.clean && build_config.repository_path.is_none() {
        std::fs::remove_dir_all(&revision_path).unwrap();
    }

    if !revision_path.exists() {
        let mut repo_builder = RepoBuilder::new();
        repo_builder.branch(&revision.revision);

        let repo_path = revision_path.join("source");
        let repo = match repo_builder.clone(&repository_url, &repo_path) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to clone: {}", e),
        };

        let submodules = repo.submodules().unwrap();
        for mut submodule in submodules {
            submodule.update(true, None).unwrap();
        }
    }

    Ok(revision)
}

pub fn build_revision(revision: &Revision, target: Target) -> Result<(), Error> {
    let source_path = revision.source_path();
    let build_path = revision.build_path();

    if !build_path.exists() {
        std::fs::create_dir_all(build_path.clone()).unwrap();
    }

    let mut cmake_config = cmake::Config::new(source_path.clone());

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

    Ok(())
}
