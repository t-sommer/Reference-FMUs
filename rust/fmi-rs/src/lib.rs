pub mod fmi2;
pub mod fmi3;
pub mod model_description;
pub mod sim;
pub mod types;

#[cfg(feature = "zip")]
pub mod util;

#[macro_use]
extern crate approx;

#[cfg(all(target_os = "linux"))]
pub const SHARED_LIBRARY_EXTENSION: &str = ".so";

#[cfg(all(target_os = "macos"))]
pub const SHARED_LIBRARY_EXTENSION: &str = ".dylib";

#[cfg(all(target_os = "windows"))]
pub const SHARED_LIBRARY_EXTENSION: &str = ".dll";
