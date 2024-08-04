use crate::common::Direction;

const KP: f64 = 0.005;
const KI: f64 = 0.0;
const KD: f64 = 0.01;
const T: f64 = 1.0;
const MAX_POTENCY: f64 = 100.0;
const MIN_POTENCY: f64 = -100.0;

pub struct PidController {
    total_error: f64,
    last_error: f64,
}

impl PidController {
    pub fn new() -> Self {
        PidController {
            total_error: 0.0,
            last_error: 0.0,
        }
    }

    pub fn get_control_signal(&mut self, origin: i32, target: i32) -> (f64, Direction) {
        let error = target as f64 - origin as f64;

        self.total_error += error;

        if self.total_error >= MAX_POTENCY {
            self.total_error = MAX_POTENCY;
        } else if self.total_error <= MIN_POTENCY {
            self.total_error = MIN_POTENCY;
        }

        let delta_error = error - self.last_error;

        let mut control_signal = KP * error + (KI * T) * self.total_error + (KD / T) * delta_error;

        if control_signal >= MAX_POTENCY {
            control_signal = MAX_POTENCY;
        } else if control_signal <= MIN_POTENCY {
            control_signal = MIN_POTENCY;
        }

        self.last_error = error;

        let direction = if control_signal >= 0.0 {
            Direction::Up
        } else {
            Direction::Down
        };

        let potency = control_signal.abs() / 100.0;

        (potency, direction)
    }
}
