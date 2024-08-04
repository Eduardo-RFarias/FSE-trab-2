use crate::common::{Direction, Elevator, Floor};
use embedded_graphics::{
    mono_font::{ascii, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Line, PrimitiveStyle, Triangle},
    text::Text,
};
use rppal::i2c::I2c;
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};

struct ElevatorState {
    direction: Direction,
    floor: Floor,
    temperature: f32,
}

pub struct SSD1306 {
    display: Ssd1306<I2CInterface<I2c>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>,
    elevator_1: ElevatorState,
    elevator_2: ElevatorState,
}

impl SSD1306 {
    pub fn new() -> Self {
        let i2c = I2c::new().unwrap();

        let mut ssd1306 = Self {
            display: Ssd1306::new(
                I2CDisplayInterface::new(i2c),
                DisplaySize128x64,
                DisplayRotation::Rotate0,
            )
            .into_buffered_graphics_mode(),
            elevator_1: ElevatorState {
                direction: Direction::Up,
                floor: Floor::First,
                temperature: 25.0,
            },
            elevator_2: ElevatorState {
                direction: Direction::Down,
                floor: Floor::Ground,
                temperature: 30.0,
            },
        };

        ssd1306.display.init().unwrap();
        ssd1306.refresh_screen();

        ssd1306
    }

    fn refresh_screen(&mut self) {
        self.display.clear(BinaryColor::Off).unwrap();

        self.render_background();
        self.render_temperature();
        self.render_floor();
        self.render_direction();

        self.display.flush().unwrap();
    }

    fn render_background(&mut self) {
        // Create a line separating the screen vertically in half
        Line::new(Point::new(64, 0), Point::new(64, 64))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut self.display)
            .unwrap();

        let text_style = MonoTextStyleBuilder::new()
            .font(&ascii::FONT_4X6)
            .text_color(BinaryColor::On)
            .build();

        // Write "Elevador 1" on the top left of the first half with a 5px padding
        Text::new("Elevador 1", Point::new(5, 5), text_style)
            .draw(&mut self.display)
            .unwrap();

        // Write "Elevador 2" on the top left of the second half with a 5px padding
        Text::new("Elevador 2", Point::new(69, 5), text_style)
            .draw(&mut self.display)
            .unwrap();
    }

    fn render_temperature(&mut self) {
        let text_1 = format!("{:.0}'C", self.elevator_1.temperature);
        let text_2 = format!("{:.0}'C", self.elevator_2.temperature);

        // The temperature is written below the title with a padding of 5px
        let point_1 = Point::new(5, 15);
        let point_2 = Point::new(69, 15);

        let text_style = MonoTextStyleBuilder::new()
            .font(&ascii::FONT_4X6)
            .text_color(BinaryColor::On)
            .build();

        Text::new(&text_1, point_1, text_style)
            .draw(&mut self.display)
            .unwrap();

        Text::new(&text_2, point_2, text_style)
            .draw(&mut self.display)
            .unwrap();
    }

    fn render_floor(&mut self) {
        let text_1 = match self.elevator_1.floor {
            Floor::Ground => "T",
            Floor::First => "1",
            Floor::Second => "2",
            Floor::Third => "3",
        };

        let text_2 = match self.elevator_2.floor {
            Floor::Ground => "T",
            Floor::First => "1",
            Floor::Second => "2",
            Floor::Third => "3",
        };

        // The floor is written on the center right of the respective elevator with a 10px padding
        let point_1 = Point::new(44, 42);
        let point_2 = Point::new(108, 42);

        let text_style = MonoTextStyleBuilder::new()
            .font(&ascii::FONT_10X20)
            .text_color(BinaryColor::On)
            .build();

        Text::new(&text_1, point_1, text_style)
            .draw(&mut self.display)
            .unwrap();

        Text::new(&text_2, point_2, text_style)
            .draw(&mut self.display)
            .unwrap();
    }

    fn render_direction(&mut self) {
        // The direction is represented by two triangles on the bottom left of the respective elevator
        // The triangle filled with the color represents the current direction
        // The triangle outlined with the color represents the opposite direction
        let upper_triangle_1 =
            Triangle::new(Point::new(15, 30), Point::new(25, 35), Point::new(5, 35));
        let upper_triangle_2 =
            Triangle::new(Point::new(79, 30), Point::new(89, 35), Point::new(69, 35));

        let lower_triangle_1 =
            Triangle::new(Point::new(15, 44), Point::new(25, 39), Point::new(5, 39));
        let lower_triangle_2 =
            Triangle::new(Point::new(79, 44), Point::new(89, 39), Point::new(69, 39));

        let stroke = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
        let fill = PrimitiveStyle::with_fill(BinaryColor::On);

        match self.elevator_1.direction {
            Direction::Up => {
                upper_triangle_1
                    .into_styled(fill)
                    .draw(&mut self.display)
                    .unwrap();
                lower_triangle_1
                    .into_styled(stroke)
                    .draw(&mut self.display)
                    .unwrap();
            }
            Direction::Down => {
                upper_triangle_1
                    .into_styled(stroke)
                    .draw(&mut self.display)
                    .unwrap();
                lower_triangle_1
                    .into_styled(fill)
                    .draw(&mut self.display)
                    .unwrap();
            }
            Direction::Stop => {
                upper_triangle_1
                    .into_styled(stroke)
                    .draw(&mut self.display)
                    .unwrap();
                lower_triangle_1
                    .into_styled(stroke)
                    .draw(&mut self.display)
                    .unwrap();
            }
        }

        match self.elevator_2.direction {
            Direction::Up => {
                upper_triangle_2
                    .into_styled(fill)
                    .draw(&mut self.display)
                    .unwrap();
                lower_triangle_2
                    .into_styled(stroke)
                    .draw(&mut self.display)
                    .unwrap();
            }
            Direction::Down => {
                upper_triangle_2
                    .into_styled(stroke)
                    .draw(&mut self.display)
                    .unwrap();
                lower_triangle_2
                    .into_styled(fill)
                    .draw(&mut self.display)
                    .unwrap();
            }
            Direction::Stop => {
                upper_triangle_2
                    .into_styled(stroke)
                    .draw(&mut self.display)
                    .unwrap();
                lower_triangle_2
                    .into_styled(stroke)
                    .draw(&mut self.display)
                    .unwrap();
            }
        }
    }

    pub fn update_temperature(&mut self, elevator: Elevator, temperature: f32) {
        let elevator = match elevator {
            Elevator::One => &mut self.elevator_1,
            Elevator::Two => &mut self.elevator_2,
        };

        if elevator.temperature == temperature {
            return;
        }

        elevator.temperature = temperature;

        self.refresh_screen();
    }

    pub fn update_floor(&mut self, elevator: Elevator, floor: Floor) {
        let elevator = match elevator {
            Elevator::One => &mut self.elevator_1,
            Elevator::Two => &mut self.elevator_2,
        };

        if elevator.floor == floor {
            return;
        }

        elevator.floor = floor;

        self.refresh_screen();
    }

    pub fn update_direction(&mut self, elevator: Elevator, direction: Direction) {
        let elevator = match elevator {
            Elevator::One => &mut self.elevator_1,
            Elevator::Two => &mut self.elevator_2,
        };

        if elevator.direction == direction {
            return;
        }

        elevator.direction = direction;

        self.refresh_screen();
    }
}
