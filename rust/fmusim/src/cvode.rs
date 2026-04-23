use std::{ffi::c_void, slice::from_raw_parts_mut};
use cvode::{cvode::{CV_NORMAL, CV_ROOT_RETURN, CVode, CVodeCreate, CVodeFree, CVodeInit, CVodeReInit, CVodeRootInit, CVodeSVtolerances, CVodeSetMaxErrTestFails, CVodeSetMaxNonlinIters, CVodeSetMaxNumSteps, CVodeSetUserData}, cvode_ls::{CVodeSetJacFn, CVodeSetLinearSolver}, nvector_serial::{N_VNew_Serial, NV_DATA_S, NV_LENGTH_S}, sundials_context::{SUNContext_Create, SUNContext_Free}, sundials_linearsolver::{SUNLinSolFree, SUNLinearSolver}, sundials_matrix::{SUNMatDestroy, SUNMatrix}, sundials_nvector::{N_VDestroy, N_Vector}, sundials_types::{SUN_COMM_NULL, SUNContext, sunindextype, sunrealtype}, sunlinsol_dense::SUNLinSol_Dense, sunmatrix_dense::{SM_COLUMN_D, SUNDenseMatrix}};
use fmi::sim::{GetDirectionalDerivativeFn, GetNominalsOfContinuousStatesFn};
use crate::sim::{GetContinuousStateDerivativesFn, GetContinuousStatesFn, GetEventIndicatorsFn, SetContinuousInputsFn, SetContinuousStatesFn, SetTimeFn, Solver, SolverFactory};

type Error = Box<dyn std::error::Error>;

struct Functions<'a> {
    nx: usize,
    nz: usize,
    unknowns: Vec<u32>,
    knowns: Vec<u32>,
    rtol: f64,
    set_time: SetTimeFn<'a>,
    set_continuous_inputs: SetContinuousInputsFn<'a>,
    get_event_indicators: GetEventIndicatorsFn<'a>,
    get_continuous_states: GetContinuousStatesFn<'a>,
    get_nominals_of_continuous_states: GetNominalsOfContinuousStatesFn<'a>,
    get_continuous_state_derivatives: GetContinuousStateDerivativesFn<'a>,
    get_directional_derivative: GetDirectionalDerivativeFn<'a>,
    set_continuous_states: SetContinuousStatesFn<'a>,
}

pub struct CVodeSolver<'a> {
    sunctx: SUNContext,
    x: N_Vector,
    abstol: N_Vector,
    A: SUNMatrix,
    LS: SUNLinearSolver,
    cvode_mem: *mut c_void,
    functions: Box<Functions<'a>>,
}

pub struct CVodeSolverFactory;

impl SolverFactory for CVodeSolverFactory {
    fn create<'a>(
        &self,
        start_time: f64,
        nx: usize,
        nz: usize,
        rtol: f64,
        unknowns: Vec<u32>,
        knowns: Vec<u32>,
        set_time: SetTimeFn<'a>,
        set_continuous_inputs: SetContinuousInputsFn<'a>,
        get_event_indicators: GetEventIndicatorsFn<'a>,
        get_continuous_states: GetContinuousStatesFn<'a>,
        get_nominals_of_continuous_states: GetNominalsOfContinuousStatesFn<'a>,
        get_continuous_state_derivatives: GetContinuousStateDerivativesFn<'a>,
        get_directional_derivative: GetDirectionalDerivativeFn<'a>,
        set_continuous_states: SetContinuousStatesFn<'a>,
    ) -> Result<Box<dyn Solver + 'a>, Error> {

        Ok(Box::new(CVodeSolver::new(
            start_time,
            nx,
            nz,
            rtol,
            unknowns,
            knowns,
            set_time,
            set_continuous_inputs,
            get_event_indicators,
            get_continuous_states,
            get_nominals_of_continuous_states,
            get_continuous_state_derivatives,
            get_directional_derivative,
            set_continuous_states,
        )))
    }
}

impl<'a> CVodeSolver<'a> {

    pub fn new(
        start_time: f64,
        nx: usize,
        nz: usize,
        rtol: f64,
        unknowns: Vec<u32>,
        knowns: Vec<u32>,
        set_time: SetTimeFn<'a>,
        set_continuous_inputs: SetContinuousInputsFn<'a>,
        get_event_indicators: GetEventIndicatorsFn<'a>,
        get_continuous_states: GetContinuousStatesFn<'a>,
        get_nominals_of_continuous_states: GetNominalsOfContinuousStatesFn<'a>,
        get_continuous_state_derivatives: GetContinuousStateDerivativesFn<'a>,
        get_directional_derivative: GetDirectionalDerivativeFn<'a>,
        set_continuous_states: SetContinuousStatesFn<'a>,
    ) -> Self {
        unsafe {
            let functions = Box::new(Functions {
                nx,
                nz,
                rtol,
                unknowns,
                knowns,
                set_time,
                set_continuous_inputs,
                get_event_indicators,
                get_continuous_states,
                get_nominals_of_continuous_states,
                get_continuous_state_derivatives,
                get_directional_derivative,
                set_continuous_states,
            });

            let mut sunctx = std::ptr::null_mut();
            let err_code =  SUNContext_Create(SUN_COMM_NULL, &mut sunctx);
            assert!(err_code == 0, "Failed to create SUNDIALS context: error code {}", err_code);

            let cvode_mem = CVodeCreate(cvode::cvode::CV_BDF, sunctx);
            assert!(!cvode_mem.is_null(), "Failed to create CVODE memory");

            let user_data: *const Functions = &*functions;

            let flag = CVodeSetUserData(cvode_mem, user_data as *mut c_void);
            assert!(flag == 0, "Failed to set user data: error code {}", flag);

            let x = N_VNew_Serial(nx as sunindextype, sunctx);
            assert!(!x.is_null(), "Failed to create N_Vector");
            (functions.get_continuous_states)((*x).as_mut()).expect("Failed to get continuous states");

            let abstol = N_VNew_Serial(nx as sunindextype, sunctx);
            assert!(!abstol.is_null(), "Failed to create N_Vector");
            let abstol_slice = (*abstol).as_mut();
            (functions.get_nominals_of_continuous_states)(abstol_slice).expect("Failed to get continuous states");

            for i in 0..nx {
                abstol_slice[i] = abstol_slice[i] * rtol;
            }

            let flag = CVodeInit(cvode_mem, f, start_time, x);
            assert!(flag == 0, "Failed to initialize CVODE: error code {}", flag);

            let flag = CVodeSVtolerances(cvode_mem, rtol, abstol);
            assert!(flag == 0, "Failed to set tolerances: error code {}", flag);

            let A = SUNDenseMatrix(nx as sunindextype, nx as sunindextype, sunctx);
            assert!(!A.is_null(), "Failed to create dense matrix");
            
            let LS = SUNLinSol_Dense(x, A, sunctx);
            assert!(!LS.is_null(), "Failed to create linear solver");

            let flag = CVodeSetLinearSolver(cvode_mem, LS, A);
            assert!(flag == 0, "Failed to set linear solver");
            
            // let flag = CVodeSetJacFn(cvode_mem, jac);
            // assert!(flag == 0, "Failed to set Jacobian function");

            let flag = CVodeRootInit(cvode_mem, nz as i32, g);
            assert!(flag == 0, "Failed to initialize rootfinding: error code {}", flag);

            // let flag = CVodeSetMaxNumSteps(cvode_mem, 5000);
            // assert!(flag == 0, "Failed to set max num steps: error code {}", flag);

            // let flag = CVodeSetMaxNonlinIters(cvode_mem, 3);
            // assert!(flag == 0, "Failed to set max nonlin iters: error code {}", flag);

            // let flag = CVodeSetMaxErrTestFails(cvode_mem, 15);
            // assert!(flag == 0, "Failed to set max err test fails: error code {}", flag);

            Self {
                sunctx,
                x,
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
            (self.functions.get_continuous_states)((*self.x).as_mut()).expect("get_continuous_states failed");
            
            let abstol_slice = (*self.abstol).as_mut();

            (self.functions.get_nominals_of_continuous_states)(abstol_slice).expect("get_nominals_of_continuous_states failed");

            for i in 0..abstol_slice.len() {
                abstol_slice[i] = abstol_slice[i] * self.functions.rtol;
            }
            
            let flag = CVodeReInit(self.cvode_mem, time, self.x);
            assert!(flag == 0, "CVodeReInit failed");
        }

        Ok(())
    }

    fn step(&mut self, next_time: f64) -> Result<(f64, bool), Error> {
            let mut tret = 0.0;

            let flag = unsafe { CVode(self.cvode_mem, next_time, self.x, &mut tret, CV_NORMAL) };
            
            if flag < 0 {
                return Err(format!("Solver error: {flag}").into());
            }

            Ok((tret, flag == CV_ROOT_RETURN))
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

    let functions: &Functions = unsafe { &*(user_data as *const Functions) };

    unsafe {
        (functions.set_time)(t).expect("Failed to set time");
        (functions.set_continuous_inputs)(t).expect("Failed to set inputs");
        (functions.set_continuous_states)((*y).as_mut()).expect("Failed to set continuous states");
        (functions.get_continuous_state_derivatives)((*ydot).as_mut()).expect("Failed to get continuous state derivatives");
    }

    0
}

// Root function
extern "C" fn g(t: sunrealtype, y: N_Vector, gout: *mut sunrealtype, user_data: *mut c_void) -> i32 {

    unsafe {
        let functions: &Functions =  &*(user_data as *const Functions);

        (functions.set_time)(t).expect("Failed to set time");
        (functions.set_continuous_inputs)(t).expect("Failed to set inputs");
        (functions.set_continuous_states)((*y).as_mut()).expect("Failed to set continuous states");
        
        let z = from_raw_parts_mut(gout, functions.nz);
        (functions.get_event_indicators)(z).expect("Failed to get event indicators");
    }
    
    0
}

// Jacobian function
extern "C" fn jac(t: sunrealtype, y: N_Vector, fy: N_Vector, Jac: SUNMatrix,
    user_data: *mut std::ffi::c_void, tmp1: N_Vector, tmp2: N_Vector, tmp3: N_Vector) -> i32 {

    unsafe {
        let functions: &Functions =  &*(user_data as *const Functions);
    
        // 1. Synchronize FMU state with CVODE's current 'y' and 't'
        (functions.set_time)(t).expect("Failed to set time");
        (functions.set_continuous_inputs)(t).expect("Failed to set inputs");
        (functions.set_continuous_states)((*y).as_mut()).expect("Failed to set continuous states");
        
        // 2. Prepare for Directional Derivatives
        // We want d(dx/dt) / d(x)
        // We compute this column by column
        let mut seed_v = vec![0.0; functions.nx]; // The 'direction' vector
        let mut result_v = vec![0.0; functions.nx]; // The resulting column

        for j in 0..functions.nx {

            // Set seed for the j-th column
            seed_v[j] = 1.0;

            if j > 0 {
                seed_v[j-1] = 0.0; // Reset previous column's seed
            }

            // 3. Call FMU to get the j-th column of the Jacobian
            (functions.get_directional_derivative)(&functions.unknowns, &functions.knowns, &seed_v, &mut result_v).expect("Failed to get directional derivative");

            // 4. Copy the result into the SUNMatrix
            // SM_COLUMN_D provides a pointer to the start of the j-th column
            let column_j = SM_COLUMN_D(Jac, j);

            let colmn_j_slice = from_raw_parts_mut(column_j, functions.nx);

            for i in 0..functions.nx {
                colmn_j_slice[i] = result_v[i];
            }
        }

//     realtype seed_v[data->nx]; // The 'direction' vector
//     realtype result_v[data->nx]; // The resulting column
    
//     // Initialize seed vector to 0
//     for (int k = 0; k < data->nx; k++) seed_v[k] = 0.0;

//     for (int j = 0; j < data->nx; j++) {
//         // Set seed for the j-th column
//         seed_v[j] = 1.0;
//         if (j > 0) seed_v[j-1] = 0.0; // Reset previous column's seed

//         // 3. Call FMU to get the j-th column of the Jacobian
//         status = fmi2GetDirectionalDerivative(
//             data->fmu_instance,
//             data->vr_derivatives, data->nx, // Knowns (outputs of the diff)
//             data->vr_states,      data->nx, // Unknowns (inputs we are varying)
//             seed_v,                         // Seed vector
//             result_v                        // Output: j-th column of J
//         );

//         if (status != fmi2OK) return -1; // CVODE recovery/error

//         // 4. Copy the result into the SUNMatrix
//         // SM_COLUMN_D provides a pointer to the start of the j-th column
//         realtype *column_j = SM_COLUMN_D(J, j);
//         for (int i = 0; i < data->nx; i++) {
//             column_j[i] = result_v[i];
//         }
//     }

//     return 0; // Success


    }
    
    0
}