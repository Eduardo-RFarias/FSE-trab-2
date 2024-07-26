use crate::{
    common::{Elevator, Floor},
    gpio::elevator_control::Direction,
};
use embedded_graphics::{
    mono_font::{ascii, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Line, PrimitiveStyle, Triangle},
    text::Text,
};
use rppal::i2c::I2c;
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};

pub struct SSD1306 {
    display: Ssd1306<I2CInterface<I2c>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>,
}

impl SSD1306 {
    pub fn new() -> Self {
        let i2c = I2c::new().unwrap();
        let interface = I2CDisplayInterface::new(i2c);

        let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();

        display.init().unwrap();

        // First clear the display
        display.clear(BinaryColor::Off).unwrap();

        // Create a line separating the screen vertically in half
        Line::new(Point::new(64, 0), Point::new(64, 64))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut display)
            .unwrap();

        let text_style = MonoTextStyleBuilder::new()
            .font(&ascii::FONT_6X10)
            .text_color(BinaryColor::On)
            .build();

        // Write "Elevador 1" on the top left of the first half with a 10px padding
        Text::new("Elevador 1", Point::new(10, 10), text_style)
            .draw(&mut display)
            .unwrap();

        // Write "Elevador 2" on the top left of the second half with a 10px padding
        Text::new("Elevador 2", Point::new(74, 10), text_style)
            .draw(&mut display)
            .unwrap();

        display.flush().unwrap();

        println!("SSD1306 initialized");
        Self { display }
    }

    pub fn update_temperature(&mut self, elevator: Elevator, temperature: f32) {
        let text = format!("{:.0}°C", temperature);

        // The temperature is written on the top right of the respective elevator with a 10px padding
        let point = match elevator {
            Elevator::One => Point::new(54, 10),
            Elevator::Two => Point::new(118, 10),
        };

        Text::new(
            &text,
            point,
            MonoTextStyleBuilder::new()
                .font(&ascii::FONT_7X13_BOLD)
                .text_color(BinaryColor::On)
                .build(),
        )
        .draw(&mut self.display)
        .unwrap();

        self.display.flush().unwrap();
    }

    pub fn update_floor(&mut self, elevator: Elevator, floor: Floor) {
        let text = match floor {
            Floor::Ground => "T",
            Floor::First => "1",
            Floor::Second => "2",
            Floor::Third => "3",
        };

        // The floor is written on the center right of the respective elevator with a 10px padding
        let point = match elevator {
            Elevator::One => Point::new(54, 32),
            Elevator::Two => Point::new(118, 32),
        };

        Text::new(
            &text,
            point,
            MonoTextStyleBuilder::new()
                .font(&ascii::FONT_10X20)
                .text_color(BinaryColor::On)
                .build(),
        )
        .draw(&mut self.display)
        .unwrap();

        self.display.flush().unwrap();
    }

    pub fn update_direction(&mut self, elevator: Elevator, direction: Direction) {
        // The direction is represented by two triangles on the bottom left of the respective elevator
        // The triangle filled with the color represents the current direction
        // The triangle outlined with the color represents the opposite direction
        let upper_triangle_point = match elevator {
            Elevator::One => Point::new(10, 54),
            Elevator::Two => Point::new(74, 54),
        };

        let lower_triangle_point = match elevator {
            Elevator::One => Point::new(10, 64),
            Elevator::Two => Point::new(74, 64),
        };

        let upper_triangle = Triangle::new(
            upper_triangle_point,
            Point::new(upper_triangle_point.x + 10, upper_triangle_point.y),
            Point::new(upper_triangle_point.x + 5, upper_triangle_point.y + 5),
        );

        let lower_triangle = Triangle::new(
            lower_triangle_point,
            Point::new(lower_triangle_point.x + 10, lower_triangle_point.y),
            Point::new(lower_triangle_point.x + 5, lower_triangle_point.y - 5),
        );

        match direction {
            Direction::Up => {
                upper_triangle
                    .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                    .draw(&mut self.display)
                    .unwrap();
                lower_triangle
                    .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
                    .draw(&mut self.display)
                    .unwrap();
            }
            Direction::Down => {
                upper_triangle
                    .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
                    .draw(&mut self.display)
                    .unwrap();
                lower_triangle
                    .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                    .draw(&mut self.display)
                    .unwrap();
            }
            Direction::Idle | Direction::Stop => {
                upper_triangle
                    .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
                    .draw(&mut self.display)
                    .unwrap();
                lower_triangle
                    .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
                    .draw(&mut self.display)
                    .unwrap();
            }
        }

        self.display.flush().unwrap();
    }
}
