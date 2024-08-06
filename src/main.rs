use elevator::elevator_control::ElevatorControl;
use signal_hook::{
    consts::{SIGINT, SIGTERM},
    iterator::Signals,
};

mod common;
mod elevator;
mod gpio;
mod i2c;
mod uart;

fn main() {
    let mut elevator = ElevatorControl::new();

    elevator.init();

    println!("Elevator is ready.");
    println!("Press Ctrl+C to stop (or send SIGINT/SIGTERM but not SIGKILL).");

    let mut signals = Signals::new([SIGINT, SIGTERM]).unwrap();

    for signal in signals.forever() {
        let signal = match signal {
            SIGINT => "SIGINT",
            SIGTERM => "SIGTERM",
            _ => unreachable!(),
        };

        println!("Received {}, shutting down...", signal);
        elevator.stop();

        break;
    }
}
