use tvm_build::{build, BuildConfig};
use tracing_subscriber;

fn main() {
    tracing_subscriber::fmt::init();

    let config = BuildConfig {
        repository: None,
        repository_path: None,
        branch: None,
        clean: false,
        output_path: None,
    };

    build(config).unwrap();
}
