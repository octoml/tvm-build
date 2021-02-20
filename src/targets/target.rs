/// A target for installing TVM, contains all target specific
/// information needed for locating tool chains and running
/// CMake.
pub struct Target {
    pub host: String,
    pub target_str: String,
    pub cmake_defines: Vec<(String, String)>,
}
