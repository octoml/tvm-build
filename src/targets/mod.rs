mod target;

pub use target::Target;

pub fn local_target() -> Target {
    let platform = futures::executor::block_on(heim::host::platform()).unwrap();
    match platform.system() {
        "Darwin" => {
            let cmake_defines = match platform.architecture() {
                heim::host::Arch::Unknown | heim::host::Arch::AARCH64 => {
                    vec![("CMAKE_OSX_ARCHITECTURES".into(), "arm64".into())]
                }
                _ => vec![],
            };

            Target {
                host: "Darwin".into(),
                target_str: "arm64-apple-darwin".into(),
                cmake_defines,
            }
        }
        "Linux" => {
            let target = match platform.architecture() {
                heim::host::Arch::AARCH64 => "aarch64-unknown-linux-gnu",
                heim::host::Arch::X86_64 => "x86_64-unknown-linux-gnu",
                _ => panic!("not supported"),
            };

            Target {
                host: "Linux".into(),
                target_str: target.into(),
                cmake_defines: vec![],
            }
        }
        _ => {
            panic!(
                "Platform `{}` unsupported, please check the issue tracker.",
                platform.system()
            );
        }
    }
}
