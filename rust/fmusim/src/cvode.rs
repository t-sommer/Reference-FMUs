use std::{ffi::c_void, slice::from_raw_parts_mut};
use cvode::{cvode::{CV_NORMAL, CV_ROOT_RETURN, CVode, CVodeCreate, CVodeFree, CVodeInit, CVodeReInit, CVodeRootInit, CVodeSVtolerances, CVodeSetUserData}, cvode_ls::CVodeSetLinearSolver, nvector_serial::{N_VNew_Serial, NV_DATA_S, NV_LENGTH_S}, sundials_context::{SUNContext_Create, SUNContext_Free}, sundials_linearsolver::{SUNLinSolFree, SUNLinearSolver}, sundials_matrix::{SUNMatDestroy, SUNMatrix}, sundials_nvector::{N_VDestroy, N_Vector}, sundials_types::{SUN_COMM_NULL, SUNContext, sunindextype, sunrealtype}, sunlinsol_dense::SUNLinSol_Dense, sunmatrix_dense::SUNDenseMatrix};
use crate::sim::{GetContinuousStateDerivativesFn, GetContinuousStatesFn, GetEventIndicatorsFn, SetContinuousInputsFn, SetContinuousStatesFn, SetTimeFn, Solver, SolverFactory};

type Error = Box<dyn std::error::Error>;

struct Functions<'a> {
    set_time: SetTimeFn<'a>,
    set_continuous_inputs: SetContinuousInputsFn<'a>,
    get_event_indicators: GetEventIndicatorsFn<'a>,
    get_continuous_states: GetContinuousStatesFn<'a>,
    get_continuous_state_derivatives: GetContinuousStateDerivativesFn<'a>,
    set_continuous_states: SetContinuousStatesFn<'a>,
}

pub struct CVodeSolver<'a> {
    nx: usize,
    nz: usize,
    sunctx: SUNContext,
    x: N_Vector,
    reltol: sunrealtype,
    abstol: N_Vector,
    A: SUNMatrix,
    LS: SUNLinearSolver,
    cvode_mem: *mut c_void,
    functions: Box<Functions<'a>>,
}

pub struct CVodeSolverFactory {
}

impl SolverFactory for CVodeSolverFactory {
    fn create<'a>(
        &self,
        start_time: f64,
        nx: usize,
        nz: usize,
        set_time: SetTimeFn<'a>,
        set_continuous_inputs: SetContinuousInputsFn<'a>,
        get_event_indicators: GetEventIndicatorsFn<'a>,
        get_continuous_states: GetContinuousStatesFn<'a>,
        get_continuous_state_derivatives: GetContinuousStateDerivativesFn<'a>,
        set_continuous_states: SetContinuousStatesFn<'a>,
    ) -> Result<Box<dyn Solver + 'a>, Error> {

        Ok(Box::new(CVodeSolver::new(
            start_time,
            nx,
            nz,
            set_time,
            set_continuous_inputs,
            get_event_indicators,
            get_continuous_states,
            get_continuous_state_derivatives,
            set_continuous_states,
        )))
    }
}

impl<'a> CVodeSolver<'a> {

    pub fn new(
        start_time: f64,
        nx: usize,
        nz: usize,
        set_time: SetTimeFn<'a>,
        set_continuous_inputs: SetContinuousInputsFn<'a>,
        get_event_indicators: GetEventIndicatorsFn<'a>,
        get_continuous_states: GetContinuousStatesFn<'a>,
        get_continuous_state_derivatives: GetContinuousStateDerivativesFn<'a>,
        set_continuous_states: SetContinuousStatesFn<'a>,
    ) -> Self {



        unsafe {
        
            // let my_num: Box<i32> = Box::new(10);
            // let my_num_ptr: *const i32 = &*my_num;
            // let mut my_speed: Box<i32> = Box::new(88);
            // let my_speed_ptr: *mut i32 = &mut *my_speed;
        
            let functions = Box::new(Functions {
                set_time,
                set_continuous_inputs,
                get_event_indicators,
                get_continuous_states,
                get_continuous_state_derivatives,
                set_continuous_states,
            });

            let mut sunctx = std::ptr::null_mut();
            let err_code =  SUNContext_Create(SUN_COMM_NULL, &mut sunctx);
            assert!(err_code == 0, "Failed to create SUNDIALS context: error code {}", err_code);

            let cvode_mem = CVodeCreate(cvode::cvode::CV_BDF, sunctx);
            assert!(!cvode_mem.is_null(), "Failed to create CVODE memory");

            // let user_data = Box::into_raw(functions) as *mut c_void;
            let user_data: *const Functions = &*functions;

            println!("user_data: {:p}", user_data);

            let flag = CVodeSetUserData(cvode_mem, user_data as *mut c_void);
            assert!(flag == 0, "Failed to set user data: error code {}", flag);

            let x = N_VNew_Serial(nx as sunindextype, sunctx);
            assert!(!x.is_null(), "Failed to create N_Vector");

            let x_slice = from_raw_parts_mut(NV_DATA_S(x), NV_LENGTH_S(x) as usize);

            (functions.get_continuous_states)(x_slice).expect("Failed to get continuous states");

            let reltol = 1e-5;
            
            let abstol = N_VNew_Serial(nx as sunindextype, sunctx);
            assert!(!abstol.is_null(), "Failed to create N_Vector");

            let abstol_slice = from_raw_parts_mut(NV_DATA_S(abstol), nx as usize);
            abstol_slice.fill(reltol);

            let flag = CVodeInit(cvode_mem, f, start_time, x);
            assert!(flag == 0, "Failed to initialize CVODE: error code {}", flag);

            let flag = CVodeSVtolerances(cvode_mem, reltol, abstol);
            assert!(flag == 0, "Failed to set tolerances: error code {}", flag);

            let A = SUNDenseMatrix(nx as sunindextype, nx as sunindextype, sunctx);
            assert!(!A.is_null(), "Failed to create dense matrix");
            
            let LS = SUNLinSol_Dense(x, A, sunctx);
            assert!(!LS.is_null(), "Failed to create linear solver");

            let flag = CVodeSetLinearSolver(cvode_mem, LS, A);
            assert!(flag == 0, "Failed to set linear solver");

            let flag = CVodeRootInit(cvode_mem, nz as i32, g);
            assert!(flag == 0, "Failed to initialize rootfinding: error code {}", flag);

            Self {
                nx,
                nz,
                sunctx,
                x,
                reltol,
                abstol,
                A,
                LS,
                cvode_mem,
                functions,
            }
        }
    }

}

impl<'a> Solver for CVodeSolver<'a> {
    
    fn reset(&mut self, time: f64) -> Result<(), Error> {

        unsafe {
            let x_slice = from_raw_parts_mut(NV_DATA_S(self.x), self.nx as usize);
            (self.functions.get_continuous_states)(x_slice).expect("get_continuous_states failed");

            // TODO: set tolerances
            
            let flag = CVodeReInit(self.cvode_mem, time, self.x);
            assert!(flag == 0, "CVodeReInit failed");
        }

        Ok(())
    }

    fn step(&mut self, next_time: f64) -> Result<(f64, bool), Error> {

        unsafe {

            let mut tret = 0.0;
            
            let x_slice = from_raw_parts_mut(NV_DATA_S(self.x), self.nx as usize);
            
            (self.functions.get_continuous_states)(x_slice).expect("get_continuous_states failed");

            println!("Before CVodeSolver::step()");
            
            let flag = CVode(self.cvode_mem, next_time, self.x, &mut tret, CV_NORMAL);
            // assert!(flag == 0, "Failed to step CVODE: error code {}", flag);
            
            println!("After CVodeSolver::step()");

            (self.functions.set_continuous_states)(x_slice).expect("set_continuous_states failed");
            
            if flag < 0 {
                return Err(format!("Solver error: {flag}").into());
            }

            Ok((tret, flag == CV_ROOT_RETURN))
        }
    }
}

impl<'a> Drop for CVodeSolver<'a> {
    fn drop(&mut self) {
        unsafe {
            N_VDestroy(self.x);
            N_VDestroy(self.abstol);
            CVodeFree(&mut self.cvode_mem);
            SUNLinSolFree(self.LS);
            SUNMatDestroy(self.A);
            SUNContext_Free(&mut self.sunctx);
        }
    }
}

// Right-hand-side function
extern "C" fn f(t: sunrealtype, y: N_Vector, ydot: N_Vector, user_data: *mut c_void) -> i32 {
    
    println!("f(..., user_data: {:p})", user_data);

    //let functions = unsafe { Box::from_raw(user_data as *mut Functions) };

    let functions: &Functions = unsafe { &*(user_data as *const Functions) };

    unsafe {
        let x = from_raw_parts_mut(NV_DATA_S(y), NV_LENGTH_S(y) as usize);
        let dx = from_raw_parts_mut(NV_DATA_S(ydot), NV_LENGTH_S(ydot) as usize);
        dx[0] = x[1];  // velocity
        dx[1] = -9.81;  // gravity

        (functions.set_time)(t).expect("Failed to set time");
        (functions.set_continuous_states)(x).expect("Failed to set continuous states");
        (functions.get_continuous_state_derivatives)(dx).expect("Failed to get continuous state derivatives");
    }
    0
}

// Root function
extern "C" fn g(t: sunrealtype, y: N_Vector, gout: *mut sunrealtype, user_data: *mut c_void) -> i32 {

    println!("g(..., user_data: {:p})", user_data);

    unsafe {
        let functions: &Functions =  &*(user_data as *const Functions);

        (functions.set_time)(t).expect("Failed to set time");

        // TODO: apply input

        let x = from_raw_parts_mut(NV_DATA_S(y), NV_LENGTH_S(y) as usize);

        (functions.set_continuous_states)(x).expect("Failed to set continuous states");
        
        let z = from_raw_parts_mut(gout, 1);

        (functions.get_event_indicators)(z).expect("Failed to get event indicators");
    }
    
    0
}