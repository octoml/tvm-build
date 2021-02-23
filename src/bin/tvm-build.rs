use tvm_build::{build, BuildConfig};
use tracing_subscriber;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt()]
struct InstallCommand {
    revision: String,
    repository: Option<String>,
    #[structopt(short, long)]
    debug: bool,
    #[structopt(short, long)]
    clean: bool,
}

#[derive(StructOpt, Debug)]
#[structopt()]
struct UninstallCommand {
    revision: String,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "A CLI for maintaining TVM installations.")]
enum TVMBuildArgs {
    /// Install a revision of TVM locally.
    Install(InstallCommand),
    /// Remove a revision of TVM.
    Uninstall(UninstallCommand),
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
            build(config)?;
            Ok(())
        },
        _ => {
            panic!("Command not yet supported")
        }
    }
}
