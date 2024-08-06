use crate::common::Elevator;
use crate::uart::esp32::{Button, Encoder, Esp32};

#[test]
fn get_encoder_value() {
    // Arrange
    let mut uart = Esp32::new();

    // Act
    uart.get_encoder_value(Encoder::One);
    uart.get_encoder_value(Encoder::Two);

    // Assert
    // No assertion needed, just testing that the function does not panic
}

#[test]
fn send_temp() {
    // Arrange
    let mut uart = Esp32::new();

    // Act
    uart.send_temp(Elevator::One, 37.0);
    uart.send_temp(Elevator::Two, 35.0);

    // Assert
    // No assertion needed, just testing that the function does not panic
}

#[test]
fn send_control_signal() {
    // Arrange
    let mut uart = Esp32::new();

    // Act
    uart.send_control_signal(Encoder::One, 50);
    uart.send_control_signal(Encoder::Two, 40);

    // Assert
    // No assertion needed, just testing that the function does not panic
}

#[test]
fn read_all_buttons() {
    // Arrange
    let mut uart = Esp32::new();

    uart.write_all_buttons(Elevator::One, &[false; 11]);
    uart.write_all_buttons(Elevator::Two, &[false; 11]);

    // Act
    let buttons = uart.read_all_buttons(Elevator::One);
    let buttons2 = uart.read_all_buttons(Elevator::Two);

    // Assert
    assert_eq!(buttons.len(), 11);
    assert_eq!(buttons2.len(), 11);

    for (_, state) in buttons {
        assert_eq!(state, false);
    }

    for (_, state) in buttons2 {
        assert_eq!(state, false);
    }
}

#[test]
fn read_buttons_in_range() {
    // Arrange
    let mut uart = Esp32::new();

    uart.write_all_buttons(Elevator::One, &[false; 11]);
    uart.write_all_buttons(Elevator::Two, &[false; 11]);

    // Act
    let buttons =
        uart.read_buttons_in_range(Elevator::One, Button::GroundFloorUp1, Button::Emergency1);

    let buttons2 =
        uart.read_buttons_in_range(Elevator::Two, Button::GroundFloorUp2, Button::Emergency2);

    // Assert
    assert_eq!(buttons.len(), 7);
    assert_eq!(buttons2.len(), 7);

    for (_, state) in buttons {
        assert_eq!(state, false);
    }

    for (_, state) in buttons2 {
        assert_eq!(state, false);
    }
}

#[test]
fn write_button() {
    // Arrange
    let mut uart = Esp32::new();

    uart.write_all_buttons(Elevator::One, &[false; 11]);
    uart.write_all_buttons(Elevator::Two, &[false; 11]);

    // Act
    uart.write_button(Elevator::One, Button::GroundFloorCall1, true);
    uart.write_button(Elevator::Two, Button::GroundFloorCall2, true);

    // Assert
    let button_state = uart.read_buttons_in_range(
        Elevator::One,
        Button::GroundFloorCall1,
        Button::GroundFloorCall1,
    )[&Button::GroundFloorCall1];

    let button_state2 = uart.read_buttons_in_range(
        Elevator::Two,
        Button::GroundFloorCall2,
        Button::GroundFloorCall2,
    )[&Button::GroundFloorCall2];

    assert_eq!(button_state, true);
    assert_eq!(button_state2, true);
}

#[test]
fn write_button_in_range() {
    // Arrange
    let mut uart = Esp32::new();

    uart.write_all_buttons(Elevator::One, &[false; 11]);
    uart.write_all_buttons(Elevator::Two, &[false; 11]);

    // Act
    uart.write_button_in_range(
        Elevator::One,
        Button::GroundFloorUp1,
        Button::Emergency1,
        &[true; 7],
    );

    uart.write_button_in_range(
        Elevator::Two,
        Button::GroundFloorUp2,
        Button::Emergency2,
        &[true; 7],
    );

    // Assert
    let buttons = uart.read_buttons_in_range(
        Elevator::One,
        Button::GroundFloorUp1,
        Button::GroundFloorCall1,
    );

    for (button, state) in buttons {
        if button == Button::GroundFloorCall1 {
            assert_eq!(state, false);
        } else {
            assert_eq!(state, true);
        }
    }

    let buttons2 = uart.read_buttons_in_range(
        Elevator::Two,
        Button::GroundFloorUp2,
        Button::GroundFloorCall2,
    );

    for (button, state) in buttons2 {
        if button == Button::GroundFloorCall2 {
            assert_eq!(state, false);
        } else {
            assert_eq!(state, true);
        }
    }
}

#[test]
fn write_all_buttons() {
    // Arrange
    let mut uart = Esp32::new();

    // Act
    uart.write_all_buttons(Elevator::One, &[true; 11]);
    uart.write_all_buttons(Elevator::Two, &[true; 11]);

    // Assert
    let buttons = uart.read_all_buttons(Elevator::One);

    for (_, state) in buttons {
        assert_eq!(state, true);
    }

    let buttons2 = uart.read_all_buttons(Elevator::Two);

    for (_, state) in buttons2 {
        assert_eq!(state, true);
    }
}
