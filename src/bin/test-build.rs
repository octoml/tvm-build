use tvm_build::{build, BuildConfig};
use tracing_subscriber;

fn main() {
    tracing_subscriber::fmt::init();
    let mut config = BuildConfig::default();
    config.verbose = true;
    build(config).unwrap();
}
