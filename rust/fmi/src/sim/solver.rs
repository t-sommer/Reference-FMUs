type Error = Box<dyn std::error::Error>;

type SetTimeFn<'a> = Box<dyn Fn(f64) -> Result<(), Error> + 'a>;
type GetEventIndicatorsFn<'a> = Box<dyn Fn(&mut [f64]) -> Result<(), Error> + 'a>;
type GetContinuousStatesFn<'a> = Box<dyn Fn(&mut [f64]) -> Result<(), Error> + 'a>;
type GetContinuousStateDerivativesFn<'a> = Box<dyn Fn(&mut [f64]) -> Result<(), Error> + 'a>;
type SetContinuousStatesFn<'a> = Box<dyn Fn(&[f64]) -> Result<(), Error> + 'a>;

pub trait Solver {
    fn reset(&mut self, time: f64) -> Result<(), Error>;
    fn step(&mut self, next_time: f64) -> Result<(f64, bool), Error>;
}

pub struct ForwardEuler<'a> {
    start_time: f64,
    fixed_step_size: f64,
    n_steps: usize,
    x: Vec<f64>,
    der_x: Vec<f64>,
    z: Vec<f64>,
    pre_z: Vec<f64>,
    set_time: SetTimeFn<'a>,
    get_event_indicators: GetEventIndicatorsFn<'a>,
    get_continuous_states: GetContinuousStatesFn<'a>,
    get_continuous_state_derivatives: GetContinuousStateDerivativesFn<'a>,
    set_continuous_states: SetContinuousStatesFn<'a>,
}

pub trait Model {
    fn set_time(&mut self, time: f64) -> Result<(), Error>;
    fn get_event_indicators(&mut self, event_indicators: &mut [f64]) -> Result<(), Error>;
    fn get_continuous_states(&mut self, continuous_states: &mut [f64]) -> Result<(), Error>;
    fn get_continuous_state_derivatives(&mut self, state_derivatives: &mut [f64]) -> Result<(), Error>;
    fn set_continuous_states(&mut self, continuous_states: &[f64]) -> Result<(), Error>;
} 

pub trait SolverFactory {
    fn create<'a>(
        &self,
        start_time: f64,
        nx: usize,
        nz: usize,
        set_time: SetTimeFn<'a>,
        get_event_indicators: GetEventIndicatorsFn<'a>,
        get_continuous_states: GetContinuousStatesFn<'a>,
        get_continuous_state_derivatives: GetContinuousStateDerivativesFn<'a>,
        set_continuous_states: SetContinuousStatesFn<'a>,
    ) -> Result<Box<dyn Solver + 'a>, Error>;
}

pub struct ForwardEulerFactory {
    pub step_size: f64,
}

impl SolverFactory for ForwardEulerFactory {
    fn create<'a>(
        &self,
        start_time: f64,
        nx: usize,
        nz: usize,
        set_time: SetTimeFn<'a>,
        get_event_indicators: GetEventIndicatorsFn<'a>,
        get_continuous_states: GetContinuousStatesFn<'a>,
        get_continuous_state_derivatives: GetContinuousStateDerivativesFn<'a>,
        set_continuous_states: SetContinuousStatesFn<'a>,
    ) -> Result<Box<dyn Solver + 'a>, Error> {
        Ok(Box::new({
            ForwardEuler {
                start_time,
                fixed_step_size: self.step_size,
                n_steps: 0,
                x: vec![0.0; nx],
                der_x: vec![0.0; nx],
                z: vec![0.0; nz],
                pre_z: vec![0.0; nz],
                set_time,
                get_event_indicators,
                get_continuous_states,
                get_continuous_state_derivatives,
                set_continuous_states,
            }
        }))
    }
}

impl<'a> ForwardEuler<'a> {

    fn do_fixed_step(&mut self) -> Result<(f64, bool), Error> {

        if self.x.len() > 0 {
            (self.get_continuous_state_derivatives)(self.der_x.as_mut_slice())?;

            for i in 0..self.x.len() {
                self.x[i] += self.der_x[i] * self.fixed_step_size;
            }
            
            (self.set_continuous_states)(self.x.as_slice())?;
        }

        self.n_steps += 1;

        let time = self.start_time + self.n_steps as f64 * self.fixed_step_size;

        (self.set_time)(time);

        let mut state_event = false;

        if self.z.len() > 0 {
            (self.get_event_indicators)(self.z.as_mut_slice())?;

            for i in 0..self.z.len() {
                if self.pre_z[i] <= 0.0 && self.z[i] > 0.0 {
                    state_event = true; // -\+
                } else if self.pre_z[i] > 0.0 && self.z[i] <= 0.0 {
                    state_event = true; // +/-
                }

                self.pre_z[i] = self.z[i];
            }
        }

        Ok((time, state_event))
    }

}

impl<'a> Solver for ForwardEuler<'a> {
    
    fn reset(&mut self, time: f64) -> Result<(), Error> {
        self.start_time = time;
        self.n_steps = 0;
        self.x.fill(0.0);
        self.der_x.fill(0.0);
        self.z.fill(0.0);
        (self.get_event_indicators)(self.pre_z.as_mut_slice())?;
        Ok(())
    }

    fn step(&mut self, next_time: f64) -> Result<(f64, bool), Error> {

        let mut time = self.start_time + self.n_steps as f64 * self.fixed_step_size;

        while time < next_time {
            let (time_reached, state_event) = self.do_fixed_step()?;

            if state_event {
                return Ok((time_reached, true));
            }

            time = time_reached;
        }

        Ok((time, false))
    }
}
