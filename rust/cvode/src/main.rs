mod nvector_serial;
mod sundials_context;
mod sundials_nvector;
mod sundials_types;

use crate::nvector_serial::N_VNew_Serial;
use crate::nvector_serial::NV_DATA_S;
use crate::sundials_types::*;
use crate::sundials_context::*;

fn main() {
    unsafe {
        let RTOL = 1e-5;
        let T0 = 0.0;
        let nx = 2;  // number of states (height, velocity)
        let nz = 1;  // number of event indicators

        let mut sunctx = std::ptr::null_mut();

        let err_code =  SUNContext_Create(SUN_COMM_NULL,&mut sunctx) ;

        assert!(err_code == 0, "Failed to create SUNDIALS context: error code {}", err_code);

        let abstol = N_VNew_Serial(nx, sunctx);

        assert!(!abstol.is_null(), "Failed to create N_Vector");

        let abstol_slice = std::slice::from_raw_parts_mut(NV_DATA_S(abstol), nx as usize);
        abstol_slice.fill(RTOL);

        let y = N_VNew_Serial(nx, sunctx);

        let x_ = std::slice::from_raw_parts_mut(NV_DATA_S(y), nx as usize);
        x_[0] = 1.0;
        x_[1] = 5.0;

        let err_code = SUNContext_Free(&mut sunctx);

        if err_code != 0 {
            panic!("Failed to free SUNDIALS context: error code {}", err_code);
        }

        println!("Success!");
    }
}
