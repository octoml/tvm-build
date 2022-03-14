mod target;

pub use target::Target;

pub fn local_target() -> Target {
    match env!("TARGET_OS") {
        "macos" => {
            let cmake_defines = match env!("TARGET_ARCH") {
                "aarch64" => {
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
        "linux" => {
            Target {
                host: "Linux".into(),
                target_str: env!("TARGET").into(),
                cmake_defines: vec![],
            }
        }
        _ => {
            panic!(
                "Platform `{}` unsupported, please check the issue tracker.",
                env!("TARGET_OS")
            );
        }
    }
}
