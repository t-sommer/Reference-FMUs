pub trait SolvertTrait {
    fn reset(&mut self, time: f64) -> Result<(), Box<dyn std::error::Error>>;
    fn step(&mut self, next_time: f64) -> Result<(f64, bool), Box<dyn std::error::Error>>;
}

pub struct Solver<'a> {
    time: f64,
    x: Vec<f64>,
    der_x: Vec<f64>,
    z: Vec<f64>,
    pre_z: Vec<f64>,
    get_event_indicators: Box<dyn Fn(&mut [f64]) -> Result<(), Box<dyn std::error::Error>> + 'a>,
    get_continuous_states: Box<dyn Fn(&mut [f64]) -> Result<(), Box<dyn std::error::Error>> + 'a>,
    get_continuous_state_derivatives: Box<dyn Fn(&mut [f64]) -> Result<(), Box<dyn std::error::Error>> + 'a>,
    set_continuous_states: Box<dyn Fn(&[f64]) -> Result<(), Box<dyn std::error::Error>> + 'a>,
}

pub type GetEventIndicatorsFn<'a> = dyn Fn(&mut [f64]) -> Result<(), Box<dyn std::error::Error>> + 'a;
pub type GetContinuousStatesFn<'a> = dyn Fn(&mut [f64]) -> Result<(), Box<dyn std::error::Error>> + 'a;
pub type GetContinuousStateDerivativesFn<'a> = dyn Fn(&mut [f64]) -> Result<(), Box<dyn std::error::Error>> + 'a;
pub type SetContinuousStatesFn<'a> = dyn Fn(&[f64]) -> Result<(), Box<dyn std::error::Error>> + 'a;

pub trait SolverFactory {
    fn create<'a>(
        &self,
        time: f64,
        nx: usize,
        nz: usize,
        get_event_indicators: Box<dyn Fn(&mut [f64]) -> Result<(), Box<dyn std::error::Error>> + 'a>,
        get_continuous_states: Box<dyn Fn(&mut [f64]) -> Result<(), Box<dyn std::error::Error>> + 'a>,
        get_continuous_state_derivatives: Box<dyn Fn(&mut [f64]) -> Result<(), Box<dyn std::error::Error>> + 'a>,
        set_continuous_states: Box<dyn Fn(&[f64]) -> Result<(), Box<dyn std::error::Error>> + 'a>,
    ) -> Result<Box<dyn SolvertTrait + 'a>, Box<dyn std::error::Error>>;
}

pub struct DefaultSolverFactory;

impl SolverFactory for DefaultSolverFactory {
    fn create<'a>(
        &self,
        time: f64,
        nx: usize,
        nz: usize,
        get_event_indicators: Box<dyn Fn(&mut [f64]) -> Result<(), Box<dyn std::error::Error>> + 'a>,
        get_continuous_states: Box<dyn Fn(&mut [f64]) -> Result<(), Box<dyn std::error::Error>> + 'a>,
        get_continuous_state_derivatives: Box<dyn Fn(&mut [f64]) -> Result<(), Box<dyn std::error::Error>> + 'a>,
        set_continuous_states: Box<dyn Fn(&[f64]) -> Result<(), Box<dyn std::error::Error>> + 'a>,
    ) -> Result<Box<dyn SolvertTrait + 'a>, Box<dyn std::error::Error>> {
        Ok(Box::new(new_solver(
            time,
            nx,
            nz,
            get_event_indicators,
            get_continuous_states,
            get_continuous_state_derivatives,
            set_continuous_states,
        )))
    }
}

pub fn new_solver<'a>(
    time: f64,
    nx: usize,
    nz: usize,
    get_event_indicators: Box<GetEventIndicatorsFn<'a>>,
    get_continuous_states: Box<GetContinuousStatesFn<'a>>,
    get_continuous_state_derivatives: Box<GetContinuousStateDerivativesFn<'a>>,
    set_continuous_states: Box<SetContinuousStatesFn<'a>>,
) -> Solver<'a> {
    Solver {
        time,
        x: vec![0.0; nx],
        der_x: vec![0.0; nx],
        z: vec![0.0; nz],
        pre_z: vec![0.0; nz],
        get_event_indicators,
        get_continuous_states,
        get_continuous_state_derivatives,
        set_continuous_states,
    }
}

impl<'a> SolvertTrait for Solver<'a> {
    fn reset(&mut self, time: f64) -> Result<(), Box<dyn std::error::Error>> {
        self.time = time;
        self.x.fill(0.0);
        self.der_x.fill(0.0);
        self.z.fill(0.0);
        (self.get_event_indicators)(self.pre_z.as_mut_slice())?;
        Ok(())
    }

    fn step(&mut self, next_time: f64) -> Result<(f64, bool), Box<dyn std::error::Error>> {
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
