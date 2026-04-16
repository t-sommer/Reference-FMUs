pub mod sundials_types;
pub mod sundials_context;

use crate::sundials_types::*;
use crate::sundials_context::*;

fn main() {

    let mut ctx = std::ptr::null_mut();

    let err_code = unsafe { SUNContext_Create(SUN_COMM_NULL,&mut ctx) };
    
    if err_code != 0 {
        panic!("Failed to create SUNDIALS context: error code {}", err_code);
    }

    println!("Hello, world!");

    let err_code = unsafe { SUNContext_Free(&mut ctx) };

    if err_code != 0 {
        panic!("Failed to free SUNDIALS context: error code {}", err_code);
    }
}
