type Error = Box<dyn std::error::Error>;

pub trait Solver {
    fn reset(&mut self, time: f64) -> Result<(), Error>;
    fn step(&mut self, next_time: f64) -> Result<(f64, bool), Error>;
}

pub struct ForwardEuler<'a> {
    time: f64,
    x: Vec<f64>,
    der_x: Vec<f64>,
    z: Vec<f64>,
    pre_z: Vec<f64>,
    get_event_indicators: Box<dyn Fn(&mut [f64]) -> Result<(), Error> + 'a>,
    get_continuous_states: Box<dyn Fn(&mut [f64]) -> Result<(), Error> + 'a>,
    get_continuous_state_derivatives: Box<dyn Fn(&mut [f64]) -> Result<(), Error> + 'a>,
    set_continuous_states: Box<dyn Fn(&[f64]) -> Result<(), Error> + 'a>,
}

pub type GetEventIndicatorsFn<'a> = dyn Fn(&mut [f64]) -> Result<(), Error> + 'a;
pub type GetContinuousStatesFn<'a> = dyn Fn(&mut [f64]) -> Result<(), Error> + 'a;
pub type GetContinuousStateDerivativesFn<'a> = dyn Fn(&mut [f64]) -> Result<(), Error> + 'a;
pub type SetContinuousStatesFn<'a> = dyn Fn(&[f64]) -> Result<(), Error> + 'a;

pub trait SolverFactory {
    fn create<'a>(
        &self,
        time: f64,
        nx: usize,
        nz: usize,
        get_event_indicators: Box<dyn Fn(&mut [f64]) -> Result<(), Error> + 'a>,
        get_continuous_states: Box<dyn Fn(&mut [f64]) -> Result<(), Error> + 'a>,
        get_continuous_state_derivatives: Box<dyn Fn(&mut [f64]) -> Result<(), Error> + 'a>,
        set_continuous_states: Box<dyn Fn(&[f64]) -> Result<(), Error> + 'a>,
    ) -> Result<Box<dyn Solver + 'a>, Error>;
}

pub struct ForwardEulerFactory;

impl SolverFactory for ForwardEulerFactory {
    fn create<'a>(
        &self,
        time: f64,
        nx: usize,
        nz: usize,
        get_event_indicators: Box<dyn Fn(&mut [f64]) -> Result<(), Error> + 'a>,
        get_continuous_states: Box<dyn Fn(&mut [f64]) -> Result<(), Error> + 'a>,
        get_continuous_state_derivatives: Box<dyn Fn(&mut [f64]) -> Result<(), Error> + 'a>,
        set_continuous_states: Box<dyn Fn(&[f64]) -> Result<(), Error> + 'a>,
    ) -> Result<Box<dyn Solver + 'a>, Error> {
        Ok(Box::new({
            let nx = nx;
            let nz = nz;
            ForwardEuler {
                time: time,
                x: vec![0.0; nx],
                der_x: vec![0.0; nx],
                z: vec![0.0; nz],
                pre_z: vec![0.0; nz],
                get_event_indicators: get_event_indicators,
                get_continuous_states: get_continuous_states,
                get_continuous_state_derivatives: get_continuous_state_derivatives,
                set_continuous_states: set_continuous_states,
            }
        }))
    }
}

impl<'a> Solver for ForwardEuler<'a> {
    fn reset(&mut self, time: f64) -> Result<(), Error> {
        self.time = time;
        self.x.fill(0.0);
        self.der_x.fill(0.0);
        self.z.fill(0.0);
        (self.get_event_indicators)(self.pre_z.as_mut_slice())?;
        Ok(())
    }

    fn step(&mut self, next_time: f64) -> Result<(f64, bool), Error> {
        if self.x.len() > 0 {
            (self.get_continuous_states)(self.x.as_mut_slice())?;
            (self.get_continuous_state_derivatives)(self.der_x.as_mut_slice())?;

            let h = next_time - self.time;

            for i in 0..self.x.len() {
                self.x[i] += self.der_x[i] * h;
            }

            (self.set_continuous_states)(self.x.as_slice())?;
        }

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

        self.time = next_time;

        Ok((self.time, state_event))
    }
}
