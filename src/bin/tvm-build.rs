use structopt::StructOpt;
use tracing_subscriber;
use tvm_build::{self, build, BuildConfig, UserSettings};

#[derive(StructOpt, Debug)]
#[structopt()]
struct InstallCommand {
    revision: String,
    repository: Option<String>,
    #[structopt(short, long)]
    /// The directory to build TVM in.
    output_path: Option<String>,
    #[structopt(short, long)]
    debug: bool,
    #[structopt(short, long)]
    clean: bool,
    #[structopt(short, long)]
    verbose: bool,
    #[structopt(flatten)]
    settings: UserSettings,
}

#[derive(StructOpt, Debug)]
#[structopt()]
struct UninstallCommand {
    revision: String,
    #[structopt(short, long)]
    /// The directory that TVM was built in.
    output_path: Option<String>
}

#[derive(StructOpt, Debug)]
#[structopt()]
struct VersionCommand {
    revision: String,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "A CLI for maintaining TVM installations.")]
enum TVMBuildArgs {
    /// Install a revision of TVM on your machine.
    Install(InstallCommand),
    /// Remove a revision of TVM on your machine.
    Uninstall(UninstallCommand),
    /// Get the configuration of the version.
    VersionConfig(VersionCommand),
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let args = TVMBuildArgs::from_args();
    match args {
        TVMBuildArgs::Install(install_cmd) => {
            let mut config = BuildConfig::default();
            config.verbose = true;
            config.branch = Some(install_cmd.revision);
            config.clean = install_cmd.clean;
            config.repository = install_cmd.repository;
            config.verbose = install_cmd.verbose;
            config.output_path = install_cmd.output_path;
            config.settings = install_cmd.settings;
            build(config)?;
            Ok(())
        }
        TVMBuildArgs::Uninstall(uninstall_cmd) => {
            tvm_build::uninstall(uninstall_cmd.revision, uninstall_cmd.output_path)?;
            Ok(())
        }
        TVMBuildArgs::VersionConfig(version_cmd) => {
            let config = tvm_build::version_config(version_cmd.revision)?;
            println!("{}", serde_json::to_string(&config).unwrap());
            Ok(())
        }
    }
}
