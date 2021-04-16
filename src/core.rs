use std::path::PathBuf;

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
pub enum TVMCMakeSetting {
    On,
    Off,
    Path(PathBuf),
}

#[derive(Debug)]
pub struct BuildConfig {
    pub repository: Option<String>,
    pub repository_path: Option<String>,
    pub output_path: Option<String>,
    pub branch: Option<String>,
    pub verbose: bool,
    pub clean: bool,

    // TVM Build Configuration Options
    /// Build with the CUDA support enabled.
    pub use_cuda: Option<TVMCMakeSetting>,
    /// Build with the CUDA runtime enabled.
    pub use_opencl: Option<TVMCMakeSetting>,
    // Build with Vulkan runtime enabled.
    pub use_vulkan: Option<TVMCMakeSetting>,
    pub use_metal: Option<TVMCMakeSetting>,
    pub use_rocm: Option<TVMCMakeSetting>,
    /// The path to ROCM.
    pub rocm_path: Option<PathBuf>,
    /// Build with Hexagon device support in TVM runtime.
    pub use_hexagon_device: Option<bool>,
    /// Path to the Hexagon SDK root (required for Hexagon support in TVM runtime or for building TVM runtime for Hexagon.
    pub use_heaxgon_dsk: Option<PathBuf>,
    /// Whether to enable TVM RPC.
    pub use_rpc: Option<bool>,
    /// Build with threading support.
    pub use_threads: Option<bool>,
    /// Build with LLVM, can also be set to specific llvm-config path.
    pub use_llvm: Option<TVMCMakeSetting>,
    /// Enable TVM's stackvm in the runtime.
    pub use_stackvm_runtime: Option<bool>,
    /// Build with graph runtime, defaults to ON.
    pub use_graph_runtime: Option<bool>,
    /// Build with graph runtime debug mode, defaults to OFF.
    pub use_graph_runtime_debug: Option<bool>,
    /// Build with OpenMP thread pool implementation, defaults to OFF.
    pub use_openmp: Option<bool>,
    /// Build Relay in debug mode, defaults to OFF.
    pub use_relay_debug: Option<bool>,
    /// Build with RTTI, defaults to ON.
    pub use_rtti: Option<bool>,
    /// Build with multi-threaded MSCV runtime.
    pub use_mscv_mt: Option<bool>,
    /// Build with Micro TVM support.
    pub use_micro: Option<bool>,
    /// Install compiler infrastructure, defaults to OFF.
    pub use_install_dev: Option<bool>,
    /// Compile with -fvisibility=hidden.
    pub hide_private_symbols: Option<bool>,
    /// Use TVM's POD compatible Map, defaults to OFF.
    pub use_fallback_stl_map: Option<bool>,
    /// tvm_option(USE_ETHOSN "Build with Arm Ethos-N" OFF)
    pub use_ethosn: Option<bool>,
    /// Defaults the index datatype to int64.
    pub use_index_default_i64: Option<bool>,
    /// Build with TensorFlow TVMDSOOp.
    pub use_tf_tvmdsoop: Option<bool>,

    // Contrib library options.
    /// Build with BYODT software emulated posit custom datatype.
    pub use_byodt_posit: Option<bool>,
    /// The blas library to be linked.
    pub use_blas: Option<String>,
    // tvm_option(USE_MKL "MKL root path when use MKL blas" OFF)
    pub use_mkl: Option<TVMCMakeSetting>,
    /// "Build with MKLDNN"
    pub use_mkldnn: Option<TVMCMakeSetting>,
    /// Enable MKLDNN (DNNL) codegen.
    pub use_dnnl_codegen: Option<bool>,
    // tvm_opt"Build with cuDNN" OFF)
    pub use_cudnn: Option<bool>,
    // tvm_option(USE_CUBLAS "Build with cuBLAS" OFF)
    pub use_cublas: Option<bool>,
    // tvm_option(USE_THRUST "Build with Thrust" OFF)
    pub use_thrust: Option<bool>,
    // tvm_option(USE_MIOPEN "Build with ROCM:MIOpen" OFF)
    pub use_miopen: Option<bool>,
    // tvm_option(USE_ROCBLAS "Build with ROCM:RoCBLAS" OFF)
    pub use_rocblas: Option<bool>,
    // tvm_option(USE_SORT "Build with sort support" ON)
    pub use_sort: Option<bool>,
    // tvm_option(USE_NNPACK "Build with nnpack support" OFF)
    pub use_nnpack: Option<bool>,
    // tvm_option(USE_RANDOM "Build with random support" ON)
    pub use_random: Option<bool>,
    // tvm_option(USE_MICRO_STANDALONE_RUNTIME "Build with micro.standalone_runtime support" OFF)
    pub use_micro_standalone_runtime: Option<bool>,
    // tvm_option(USE_CPP_RPC "Build CPP RPC" OFF)
    pub use_cpp_rpc: Option<bool>,
    // tvm_option(USE_TFLITE "Build with tflite support" OFF)
    pub use_tflite: Option<bool>,
    // tvm_option(USE_TENSORFLOW_PATH "TensorFlow root path when use TFLite" none)
    pub use_tensorflow_path: Option<PathBuf>,
    // tvm_option(USE_COREML "Build with coreml support" OFF)
    pub use_coreml: Option<bool>,
    // tvm_option(USE_TARGET_ONNX "Build with ONNX Codegen support" OFF)
    pub use_target_onnx: Option<bool>,
    // tvm_option(USE_ARM_COMPUTE_LIB "Build with Arm Compute Library" OFF)
    pub use_arm_compute_lib: Option<bool>,
    // tvm_option(USE_ARM_COMPUTE_LIB_GRAPH_RUNTIME "Build with Arm Compute Library graph runtime" OFF)
    pub use_arm_compute_lib_graph_runtime: Option<TVMCMakeSetting>,
    /// Build with TensorRT Codegen support, defaults to OFF>
    pub use_tensorrt_codegen: Option<bool>,
    /// Build with TensorRT runtime, defaults to OFF.
    pub use_tensorrt_runtime: Option<TVMCMakeSetting>,
    /// Build with Rust based compiler extensions, defaults to OFF.
    pub use_rust_ext: Option<String>,
    /// Build with VITIS-AI Codegen support, defaults to OFF.
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
            use_cuda: None,
            use_opencl: None,
            use_vulkan: None,
            use_metal: None,
            use_rocm: None,
            rocm_path: None,
            use_hexagon_device: None,
            use_heaxgon_dsk: None,
            use_rpc: None,
            use_threads: None,
            use_llvm: None,
            use_stackvm_runtime: None,
            use_graph_runtime: None,
            use_graph_runtime_debug: None,
            use_openmp: None,
            use_relay_debug: None,
            use_rtti: None,
            use_mscv_mt: None,
            use_micro: None,
            use_install_dev: None,
            hide_private_symbols: None,
            use_fallback_stl_map: None,
            use_ethosn: None,
            use_index_default_i64: None,
            use_tf_tvmdsoop: None,
            use_byodt_posit: None,
            use_blas: None,
            use_mkl: None,
            use_mkldnn: None,
            use_dnnl_codegen: None,
            use_cudnn: None,
            use_cublas: None,
            use_thrust: None,
            use_miopen: None,
            use_rocblas: None,
            use_sort: None,
            use_nnpack: None,
            use_random: None,
            use_micro_standalone_runtime: None,
            use_cpp_rpc: None,
            use_tflite: None,
            use_tensorflow_path: None,
            use_coreml: None,
            use_target_onnx: None,
            use_arm_compute_lib: None,
            use_arm_compute_lib_graph_runtime: None,
            use_tensorrt_codegen: None,
            use_tensorrt_runtime: None,
            use_rust_ext: None,
            use_vitis_ai: None,
        }
    }
}

pub struct Revision {
    revision: String,
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
