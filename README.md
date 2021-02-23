# tvm-build
A library for building TVM programmatically.

`tvm-build` contains a library for programmatically installing multiple versions of TVM using
different configurations and build settings, as well as a command line tool for installing TVM.

The library is currently under active development and the goal is to serve a tool for installing
TVM for the Rust bindings as well as provide an easy tool for end-users to install both mainline
and custom forks of TVM.

There exists some duplicate functionality across the many pieces of the stack, but the goal is
to mimic the ease of use enjoyed by tools such as `pyenv` and `rbenv`.

Currently the easiest way to get started is to install the tool directly from Cargo using:
```
cargo install tvm-build
```

Once installed you can see available commands by running `tvm-build --help`.

For programmatic use you can perform a minimal build using the below code:
```
let mut build_config = BuildConfig::default();
build_config.repository = Some("https://github.com/jroesch/tvm".to_string());
build_config.branch = Some("rust-tvm-build".to_string());
```
