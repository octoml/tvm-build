use std::{ascii::AsciiExt, path::PathBuf, str::FromStr};
use structopt::StructOpt;

use cmake;
use dirs;
use git2::build::RepoBuilder;
use thiserror::Error;
use tracing::{self, info};

use super::targets::Target;

const TVM_REPO: &'static str = "https://github.com/apache/tvm";
const DEFAULT_BRANCH: &'static str = "main";

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Git2(#[from] git2::Error),
    #[error("{0}")]
    IoError(#[from] std::io::Error),
    #[error("the directory does not exist: {0}")]
    DirectoryNotFound(String),
    #[error("the requested revision ({revision}) and repository ({repository}) combination does not exist.")]
    RevisionNotFound {
        revision: String,
        repository: String,
    },
}

/// Many TVM CMake settings are either OFF (disabled), ON (with auto detection) or
/// a path implying on with a fixed configuration.
///
/// This enumeration represents all cases in a more Rust friendly way.
#[derive(Debug)]
pub enum CMakeSetting {
    On,
    Off,
    Path(PathBuf),
}

impl FromStr for CMakeSetting {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "on" => Ok(CMakeSetting::On),
            "off" => Ok(CMakeSetting::Off),
            _ => Ok(CMakeSetting::Path(PathBuf::from_str(s)?)),
        }
    }
}

/// Convert something into a value that can be used in `cmake::Config::define`.
trait CMakeSettingValue {
    fn as_string_value(&self) -> String;
}

impl CMakeSettingValue for &bool {
    fn as_string_value(&self) -> String {
        if **self { "ON" } else { "OFF" }.to_string()
    }
}

impl CMakeSettingValue for &PathBuf {
    fn as_string_value(&self) -> String {
        format!("{}", self.display())
    }
}

impl CMakeSettingValue for &String {
    fn as_string_value(&self) -> String {
        self.to_string()
    }
}

impl CMakeSettingValue for &CMakeSetting {
    fn as_string_value(&self) -> String {
        match self {
            CMakeSetting::On => "ON".to_string(),
            CMakeSetting::Off => "OFF".to_string(),
            CMakeSetting::Path(path) => path.as_string_value(),
        }
    }
}

#[derive(Debug, Default, StructOpt)]
pub struct UserSettings {
    // TVM Build Configuration Options
    /// Build with the CUDA support enabled.
    #[structopt(long)]
    pub use_cuda: Option<CMakeSetting>,
    /// Build with the CUDA runtime enabled.
    #[structopt(long)]
    pub use_opencl: Option<CMakeSetting>,
    // Build with Vulkan runtime enabled.
    #[structopt(long)]
    pub use_vulkan: Option<CMakeSetting>,
    #[structopt(long)]
    pub use_metal: Option<CMakeSetting>,
    #[structopt(long)]
    pub use_rocm: Option<CMakeSetting>,
    /// The path to ROCM.
    #[structopt(long)]
    pub rocm_path: Option<PathBuf>,
    /// Build with Hexagon device support in TVM runtime.
    #[structopt(long)]
    pub use_hexagon_device: Option<bool>,
    /// Path to the Hexagon SDK root (required for Hexagon support in TVM runtime or for building TVM runtime for Hexagon.
    #[structopt(long)]
    pub use_heaxgon_dsk: Option<PathBuf>,
    /// Whether to enable TVM RPC.
    #[structopt(long)]
    pub use_rpc: Option<bool>,
    /// Build with threading support.
    #[structopt(long)]
    pub use_threads: Option<bool>,
    /// Build with LLVM, can also be set to specific llvm-config path.
    #[structopt(long)]
    pub use_llvm: Option<CMakeSetting>,
    /// Enable TVM's stackvm in the runtime.
    #[structopt(long)]
    pub use_stackvm_runtime: Option<bool>,
    /// Build with graph runtime, defaults to ON.
    #[structopt(long)]
    pub use_graph_runtime: Option<bool>,
    /// Build with graph runtime debug mode, defaults to OFF.
    #[structopt(long)]
    pub use_graph_runtime_debug: Option<bool>,
    /// Build with OpenMP thread pool implementation, defaults to OFF.
    #[structopt(long)]
    pub use_openmp: Option<bool>,
    /// Build Relay in debug mode, defaults to OFF.
    #[structopt(long)]
    pub use_relay_debug: Option<bool>,
    /// Build with RTTI, defaults to ON.
    #[structopt(long)]
    pub use_rtti: Option<bool>,
    /// Build with multi-threaded MSCV runtime.
    #[structopt(long)]
    pub use_mscv_mt: Option<bool>,
    /// Build with Micro TVM support.
    #[structopt(long)]
    pub use_micro: Option<bool>,
    /// Install compiler infrastructure, defaults to OFF.
    #[structopt(long)]
    pub use_install_dev: Option<bool>,
    /// Compile with -fvisibility=hidden.
    #[structopt(long)]
    pub hide_private_symbols: Option<bool>,
    /// Use TVM's POD compatible Map, defaults to OFF.
    #[structopt(long)]
    pub use_fallback_stl_map: Option<bool>,
    /// tvm_option(USE_ETHOSN "Build with Arm Ethos-N" OFF)
    #[structopt(long)]
    pub use_ethosn: Option<bool>,
    /// Defaults the index datatype to int64.
    #[structopt(long)]
    pub use_index_default_i64: Option<bool>,
    /// Build with TensorFlow TVMDSOOp.
    #[structopt(long)]
    pub use_tf_tvmdsoop: Option<bool>,

    // Contrib library options.
    /// Build with BYODT software emulated posit custom datatype.
    #[structopt(long)]
    pub use_byodt_posit: Option<bool>,
    /// The blas library to be linked.
    #[structopt(long)]
    pub use_blas: Option<String>,
    // tvm_option(USE_MKL "MKL root path when use MKL blas" OFF)
    #[structopt(long)]
    pub use_mkl: Option<CMakeSetting>,
    /// "Build with MKLDNN"
    #[structopt(long)]
    pub use_mkldnn: Option<CMakeSetting>,
    /// Enable MKLDNN (DNNL) codegen.
    #[structopt(long)]
    pub use_dnnl_codegen: Option<bool>,
    // tvm_opt"Build with cuDNN" OFF)
    #[structopt(long)]
    pub use_cudnn: Option<bool>,
    // tvm_option(USE_CUBLAS "Build with cuBLAS" OFF)
    #[structopt(long)]
    pub use_cublas: Option<bool>,
    // tvm_option(USE_THRUST "Build with Thrust" OFF)
    #[structopt(long)]
    pub use_thrust: Option<bool>,
    // tvm_option(USE_MIOPEN "Build with ROCM:MIOpen" OFF)
    #[structopt(long)]
    pub use_miopen: Option<bool>,
    // tvm_option(USE_ROCBLAS "Build with ROCM:RoCBLAS" OFF)
    #[structopt(long)]
    pub use_rocblas: Option<bool>,
    // tvm_option(USE_SORT "Build with sort support" ON)
    #[structopt(long)]
    pub use_sort: Option<bool>,
    // tvm_option(USE_NNPACK "Build with nnpack support" OFF)
    #[structopt(long)]
    pub use_nnpack: Option<bool>,
    // tvm_option(USE_RANDOM "Build with random support" ON)
    #[structopt(long)]
    pub use_random: Option<bool>,
    // tvm_option(USE_MICRO_STANDALONE_RUNTIME "Build with micro.standalone_runtime support" OFF)
    #[structopt(long)]
    pub use_micro_standalone_runtime: Option<bool>,
    // tvm_option(USE_CPP_RPC "Build CPP RPC" OFF)
    #[structopt(long)]
    pub use_cpp_rpc: Option<bool>,
    // tvm_option(USE_TFLITE "Build with tflite support" OFF)
    #[structopt(long)]
    pub use_tflite: Option<bool>,
    // tvm_option(USE_TENSORFLOW_PATH "TensorFlow root path when use TFLite" none)
    #[structopt(long)]
    pub use_tensorflow_path: Option<PathBuf>,
    // tvm_option(USE_COREML "Build with coreml support" OFF)
    #[structopt(long)]
    pub use_coreml: Option<bool>,
    // tvm_option(USE_TARGET_ONNX "Build with ONNX Codegen support" OFF)
    #[structopt(long)]
    pub use_target_onnx: Option<bool>,
    // tvm_option(USE_ARM_COMPUTE_LIB "Build with Arm Compute Library" OFF)
    #[structopt(long)]
    pub use_arm_compute_lib: Option<bool>,
    // tvm_option(USE_ARM_COMPUTE_LIB_GRAPH_RUNTIME "Build with Arm Compute Library graph runtime" OFF)
    #[structopt(long)]
    pub use_arm_compute_lib_graph_runtime: Option<CMakeSetting>,
    /// Build with TensorRT Codegen support, defaults to OFF>
    #[structopt(long)]
    pub use_tensorrt_codegen: Option<bool>,
    /// Build with TensorRT runtime, defaults to OFF.
    #[structopt(long)]
    pub use_tensorrt_runtime: Option<CMakeSetting>,
    /// Build with Rust based compiler extensions, defaults to OFF.
    #[structopt(long)]
    pub use_rust_ext: Option<String>,
    /// Build with VITIS-AI Codegen support, defaults to OFF.
    #[structopt(long)]
    pub use_vitis_ai: Option<bool>,
    // Note(@jroesch): these options are supported by TVM but not exposed by this interface
    // we instead use defaults.
    //
    // Configuration for 3rdparty libraries.
    // tvm_option(DLPACK_PATH "Path to DLPACK" "3rdparty/dlpack/include")
    // tvm_option(DMLC_PATH "Path to DMLC" "3rdparty/dmlc-core/include")
    // tvm_option(RANG_PATH "Path to RANG" "3rdparty/rang/include")
    // tvm_option(COMPILER_RT_PATH "Path to COMPILER-RT" "3rdparty/compiler-rt")
    // tvm_option(PICOJSON_PATH "Path to PicoJSON" "3rdparty/picojson")

    /// Whether to build static libtvm_runtime.a, the default is to build the dynamic
    /// version: libtvm_runtime.so.
    #[structopt(long)]
    build_static_runtime: Option<bool>,
}

#[derive(Debug)]
pub struct BuildConfig {
    pub repository: Option<String>,
    pub repository_path: Option<String>,
    pub output_path: Option<String>,
    pub branch: Option<String>,
    pub verbose: bool,
    pub clean: bool,
    pub settings: UserSettings,
}

impl std::default::Default for BuildConfig {
    fn default() -> BuildConfig {
        BuildConfig {
            repository: None,
            repository_path: None,
            output_path: None,
            branch: None,
            verbose: false,
            clean: false,
            settings: UserSettings::default(),
        }
    }
}

// convert to lazy<T>?
pub(crate) fn tvm_build_directory() -> PathBuf {
    let home_dir = dirs::home_dir().expect("requires a home directory");
    home_dir.join(".tvm_build")
}

impl BuildConfig {
    // TODO: split per revision
    pub fn get_revision(&self) -> Result<Revision, Error> {
        info!("tvm_build::build");
        let repository_url = self.repository.clone().unwrap_or(TVM_REPO.into());

        let branch = self.branch.clone().unwrap_or(DEFAULT_BRANCH.into());
        let revision = Revision::new(branch);

        let revision_path = match &self.repository_path {
            Some(path) => std::path::Path::new(&path).into(),
            // todo: check that the provided path exists
            None => revision.path(),
        };

        // If a user specifies the repository directory we assume we
        // don't own it and won't clean it.
        if revision_path.exists() && self.clean && self.repository_path.is_none() {
            // This fails if doesn't exist
            std::fs::remove_dir_all(&revision_path)?;
        }

        if !revision.source_path().exists() {
            let mut repo_builder = RepoBuilder::new();
            repo_builder.branch(&revision.revision);
            println!("{}", repository_url);
            let repo_path = revision_path.join("source");
            let repo = match repo_builder.clone(&repository_url, &repo_path) {
                Ok(repo) => Ok(repo),
                Err(e) => Err(match e.code() {
                    git2::ErrorCode::NotFound => Error::RevisionNotFound {
                        repository: repository_url,
                        revision: revision.revision.clone(),
                    },
                    _ => e.into(),
                }),
            }?;
            // todo(@jroesch): key build repos by sha? right now branch alone potentially conflicts.
            let submodules = repo.submodules()?;
            for mut submodule in submodules {
                submodule.update(true, None)?;
            }
        }

        Ok(revision)
    }

    fn setting_key_value<T: CMakeSettingValue>(k: &str, value: T) -> (String, String) {
        (k.to_string(), value.as_string_value())
    }

    // Returns any user settings to be "set" as cmake definitions.
    pub fn as_cmake_define_key_values(&self) -> impl Iterator<Item = (String, String)> {
        let UserSettings {
            use_cuda,
            use_opencl,
            use_vulkan,
            use_metal,
            use_rocm,
            rocm_path,
            use_hexagon_device,
            use_heaxgon_dsk,
            use_rpc,
            use_threads,
            use_llvm,
            use_stackvm_runtime,
            use_graph_runtime,
            use_graph_runtime_debug,
            use_openmp,
            use_relay_debug,
            use_rtti,
            use_mscv_mt,
            use_micro,
            use_install_dev,
            hide_private_symbols,
            use_fallback_stl_map,
            use_ethosn,
            use_index_default_i64,
            use_tf_tvmdsoop,
            use_byodt_posit,
            use_blas,
            use_mkl,
            use_mkldnn,
            use_dnnl_codegen,
            use_cudnn,
            use_cublas,
            use_thrust,
            use_miopen,
            use_rocblas,
            use_sort,
            use_nnpack,
            use_random,
            use_micro_standalone_runtime,
            use_cpp_rpc,
            use_tflite,
            use_tensorflow_path,
            use_coreml,
            use_target_onnx,
            use_arm_compute_lib,
            use_arm_compute_lib_graph_runtime,
            use_tensorrt_codegen,
            use_tensorrt_runtime,
            use_rust_ext,
            use_vitis_ai,
            build_static_runtime
        } = &self.settings;

        vec![
            use_cuda
                .as_ref()
                .map(|s| Self::setting_key_value("USE_CUDA", s)),
            use_opencl
                .as_ref()
                .map(|s| Self::setting_key_value("USE_OPENCL", s)),
            use_vulkan
                .as_ref()
                .map(|s| Self::setting_key_value("USE_VULKAN", s)),
            use_metal
                .as_ref()
                .map(|s| Self::setting_key_value("USE_METAL", s)),
            use_rocm
                .as_ref()
                .map(|s| Self::setting_key_value("USE_ROCM", s)),
            rocm_path
                .as_ref()
                .map(|s| Self::setting_key_value("ROCM_PATH", s)),
            use_hexagon_device
                .as_ref()
                .map(|s| Self::setting_key_value("USE_HEXAGON_DEVICE", s)),
            use_heaxgon_dsk
                .as_ref()
                .map(|s| Self::setting_key_value("USE_HEAXGON_DSK", s)),
            use_rpc
                .as_ref()
                .map(|s| Self::setting_key_value("USE_RPC", s)),
            use_threads
                .as_ref()
                .map(|s| Self::setting_key_value("USE_THREADS", s)),
            use_llvm
                .as_ref()
                .map(|s| Self::setting_key_value("USE_LLVM", s)),
            use_stackvm_runtime
                .as_ref()
                .map(|s| Self::setting_key_value("USE_STACKVM_RUNTIME", s)),
            use_graph_runtime
                .as_ref()
                .map(|s| Self::setting_key_value("USE_GRAPH_RUNTIME", s)),
            use_graph_runtime_debug
                .as_ref()
                .map(|s| Self::setting_key_value("USE_GRAPH_RUNTIME_DEBUG", s)),
            use_openmp
                .as_ref()
                .map(|s| Self::setting_key_value("USE_OPENMP", s)),
            use_relay_debug
                .as_ref()
                .map(|s| Self::setting_key_value("USE_RELAY_DEBUG", s)),
            use_rtti
                .as_ref()
                .map(|s| Self::setting_key_value("USE_RTTI", s)),
            use_mscv_mt
                .as_ref()
                .map(|s| Self::setting_key_value("USE_MSCV_MT", s)),
            use_micro
                .as_ref()
                .map(|s| Self::setting_key_value("USE_MICRO", s)),
            use_install_dev
                .as_ref()
                .map(|s| Self::setting_key_value("USE_INSTALL_DEV", s)),
            hide_private_symbols
                .as_ref()
                .map(|s| Self::setting_key_value("HIDE_PRIVATE_SYMBOLS", s)),
            use_fallback_stl_map
                .as_ref()
                .map(|s| Self::setting_key_value("USE_FALLBACK_STL_MAP", s)),
            use_ethosn
                .as_ref()
                .map(|s| Self::setting_key_value("USE_ETHOSN", s)),
            use_index_default_i64
                .as_ref()
                .map(|s| Self::setting_key_value("USE_INDEX_DEFAULT_I64", s)),
            use_tf_tvmdsoop
                .as_ref()
                .map(|s| Self::setting_key_value("USE_TF_TVMDSOOP", s)),
            use_byodt_posit
                .as_ref()
                .map(|s| Self::setting_key_value("USE_BYODT_POSIT", s)),
            use_blas
                .as_ref()
                .map(|s| Self::setting_key_value("USE_BLAS", s)),
            use_mkl
                .as_ref()
                .map(|s| Self::setting_key_value("USE_MKL", s)),
            use_mkldnn
                .as_ref()
                .map(|s| Self::setting_key_value("USE_MKLDNN", s)),
            use_dnnl_codegen
                .as_ref()
                .map(|s| Self::setting_key_value("USE_DNNL_CODEGEN", s)),
            use_cudnn
                .as_ref()
                .map(|s| Self::setting_key_value("USE_CUDNN", s)),
            use_cublas
                .as_ref()
                .map(|s| Self::setting_key_value("USE_CUBLAS", s)),
            use_thrust
                .as_ref()
                .map(|s| Self::setting_key_value("USE_THRUST", s)),
            use_miopen
                .as_ref()
                .map(|s| Self::setting_key_value("USE_MIOPEN", s)),
            use_rocblas
                .as_ref()
                .map(|s| Self::setting_key_value("USE_ROCBLAS", s)),
            use_sort
                .as_ref()
                .map(|s| Self::setting_key_value("USE_SORT", s)),
            use_nnpack
                .as_ref()
                .map(|s| Self::setting_key_value("USE_NNPACK", s)),
            use_random
                .as_ref()
                .map(|s| Self::setting_key_value("USE_RANDOM", s)),
            use_micro_standalone_runtime
                .as_ref()
                .map(|s| Self::setting_key_value("USE_MICRO_STANDALONE_RUNTIME", s)),
            use_cpp_rpc
                .as_ref()
                .map(|s| Self::setting_key_value("USE_CPP_RPC", s)),
            use_tflite
                .as_ref()
                .map(|s| Self::setting_key_value("USE_TFLITE", s)),
            use_tensorflow_path
                .as_ref()
                .map(|s| Self::setting_key_value("USE_TENSORFLOW_PATH", s)),
            use_coreml
                .as_ref()
                .map(|s| Self::setting_key_value("USE_COREML", s)),
            use_target_onnx
                .as_ref()
                .map(|s| Self::setting_key_value("USE_TARGET_ONNX", s)),
            use_arm_compute_lib
                .as_ref()
                .map(|s| Self::setting_key_value("USE_ARM_COMPUTE_LIB", s)),
            use_arm_compute_lib_graph_runtime
                .as_ref()
                .map(|s| Self::setting_key_value("USE_ARM_COMPUTE_LIB_GRAPH_RUNTIME", s)),
            use_tensorrt_codegen
                .as_ref()
                .map(|s| Self::setting_key_value("USE_TENSORRT_CODEGEN", s)),
            use_tensorrt_runtime
                .as_ref()
                .map(|s| Self::setting_key_value("USE_TENSORRT_RUNTIME", s)),
            use_rust_ext
                .as_ref()
                .map(|s| Self::setting_key_value("USE_RUST_EXT", s)),
            use_vitis_ai
                .as_ref()
                .map(|s| Self::setting_key_value("USE_VITIS_AI", s)),
            build_static_runtime
                .as_ref()
                .map(|s| Self::setting_key_value("BUILD_STATIC_RUNTIME", s)),
        ]
        .into_iter()
        .flatten()
    }
}

pub struct Revision {
    revision: String,
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

    pub fn build_for(&self, build_config: &BuildConfig, target: Target) -> Result<(), Error> {
        let source_path = self.source_path();
        let build_path = self.build_path();

        if !build_path.exists() {
            std::fs::create_dir_all(build_path.clone())?;
            // .map_err
            // Err(err) =>
            // .context(format!("the build directory does not exist: {:?}", build_path))?;
        }

        let mut cmake_config = cmake::Config::new(source_path.clone());

        cmake_config
            .generator("Unix Makefiles")
            .out_dir(build_path.clone())
            .target(&target.target_str)
            .host(&target.host)
            .profile("Debug");

        for (key, value) in build_config.as_cmake_define_key_values() {
            println!("setting {}={}", key, value);
            let _ = cmake_config.define(key, value);
        }

        if build_config.verbose {
            cmake_config.very_verbose(true);
        }

        cmake_config.build();

        Ok(())
    }
}

pub struct BuildResult {
    pub revision: Revision,
}
