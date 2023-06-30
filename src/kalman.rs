pub struct Kalman {
    gain: f64,
    process_variance: f64,
    estimation_error: f64,
    current_estimation: f64,
    last_estimation: f64,
}

impl Kalman {
    const ROOM_TEMPERATURE: f64 = 25.0;

    /// Creates new instance of the Kalman filter
    ///
    /// measurement_error: How much do we expect to our measurement vary
    /// process_variance: How fast your measurement moves. Usually 0.001 - 1, recommended 0.01.
    pub fn new(measurement_error: f64, process_variance: f64) -> Self {
        // Can be initilized with the same value as measurement_error,
        // since the kalman filter will adjust its value.
        let estimation_error = measurement_error;
        let gain = estimation_error / (estimation_error + measurement_error);

        Self {
            gain,
            process_variance,
            estimation_error,
            current_estimation: Self::ROOM_TEMPERATURE,
            last_estimation: Self::ROOM_TEMPERATURE,
        }
    }

    pub fn update(&mut self, temp: f64) {
        self.current_estimation = self.last_estimation + self.gain * (temp - self.last_estimation);
        self.estimation_error = (1.0 - self.gain) * self.estimation_error
            + f64::abs(self.last_estimation - self.current_estimation) * self.process_variance;
        self.last_estimation = self.current_estimation;
    }

    pub fn value(&self) -> f64 {
        self.current_estimation
    }
}
