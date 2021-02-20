mod target;

pub use target::Target;

pub fn local_target() -> Target {
    let platform = futures::executor::block_on(heim::host::platform()).unwrap();
    match platform.system() {
       "Darwin" => {
            let cmake_defines = match platform.architecture() {
                heim::host::Arch::Unknown | heim::host::Arch::AARCH64 => {
                    vec![("CMAKE_OSX_ARCHITECTURES".into(), "arm64".into())]
                },
                _ => vec![]
            };

            Target {
                host: "Darwin".into(),
                target_str: "arm64-apple-darwin".into(),
                cmake_defines,
            }
       },
       _ => {
           panic!("Platform `{}` unsupported, please check the issue tracker.", platform.system());
       }
    }
}
