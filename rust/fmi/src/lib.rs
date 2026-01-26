pub mod types;
pub mod fmi2;
pub mod fmi3;
pub mod model_description;
pub mod util;

#[cfg(all(target_os = "linux"))]
pub const SHARED_LIBRARY_EXTENSION: &str = ".so";

#[cfg(all(target_os = "macos"))]
pub const SHARED_LIBRARY_EXTENSION: &str = ".dylib";

#[cfg(all(target_os = "windows"))]
pub const SHARED_LIBRARY_EXTENSION: &str = ".dll";
