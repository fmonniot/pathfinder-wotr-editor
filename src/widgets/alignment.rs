use crate::data::Alignment;
use iced::{canvas, Color};
use log::debug;

/// Represent a character alignment on the Alignment Wheel
///
/// Note that this part of the iced API will change considerably when upgrading
/// from 0.1 to 0.2. As such, I tried to keep most of the draw content Layer or Drawable
/// agnostic and only relying on Path and Frame primitive.
impl canvas::Drawable for Alignment {
    fn draw(&self, frame: &mut canvas::Frame) {
        use canvas::path::Builder;
        use canvas::Path;
        use iced::Point;

        let center = frame.center();
        let radius = frame.width().min(frame.height()) / 2.0;

        let inner_radius = radius / 3.0;
        let outer_radius = (radius * 2.0) / 3.0;

        // True neutral inner circle
        let neutral = Path::circle(center, inner_radius);
        frame.fill(&neutral, color_neutral());

        // Individual sections angles
        let angles = build_wheel_angles();

        // Brush for the separations
        let thin_stroke = canvas::Stroke {
            width: radius / 100.0,
            color: color_border(),
            line_cap: canvas::LineCap::Round,
            ..canvas::Stroke::default()
        };

        // Translate the frame to have the middle of it be coordinates (0, 0)
        frame.translate(iced::Vector::new(radius, radius));

        // Paint each alignment section
        let mut index = 0;
        for start_angle in &angles {
            let end_angle = start_angle + 45.0;

            let color = match index {
                0..=2 => color_evil(),
                4..=6 => color_good(),
                _ => color_neutral(),
            };
            index += 1;

            debug!("angle: start = {}°, end = {}°", start_angle, end_angle);

            let x_s = start_angle.to_radians().cos();
            let y_s = start_angle.to_radians().sin();
            let x_e = end_angle.to_radians().cos();
            let y_e = end_angle.to_radians().sin();

            // TODO Needs an ASCII drawing to position the 4 points
            // Especially useful for the path Builder below
            let p1 = Point::new(x_s * inner_radius, y_s * inner_radius);
            let p2 = Point::new(x_s * outer_radius, y_s * outer_radius);
            let p3 = Point::new(x_e * outer_radius, y_e * outer_radius);
            let p4 = Point::new(x_e * inner_radius, y_e * inner_radius);

            debug!("points = [{:?},{:?},{:?},{:?}]", p1, p2, p3, p4);

            // arc_to draw in clockwise direction only. We shuffle the starting
            // points to limit the number of instruction and make sure the fill
            // will cover all surface without overdraw.
            let mut builder = Builder::new();
            builder.move_to(p3);
            builder.arc_to(p3, p2, outer_radius);
            builder.move_to(p3);
            builder.line_to(p4);
            builder.arc_to(p4, p1, inner_radius);
            builder.line_to(p2);

            let path = builder.build();

            // Make the lines and fill with the alignment color
            frame.stroke(&path, thin_stroke);
            frame.fill(&path, color);

            // Debug mode, label cell with its index
            // TODO add a debug flag
            frame.with_save(|frame| {
                let x = (p1.x + p3.x) / 2.0;
                let y = (p4.y + p2.y) / 2.0;
                frame.translate(iced::Vector::new(x, y));
                frame.fill_text(format!("{}", index));
            });
        }
    }
}

fn build_wheel_angles() -> [f32; 8] {
    let mut angles: [f32; 8] = [0.0; 8];

    angles[0] = 22.0;
    for i in 1..8 {
        angles[i] = angles[i - 1] + 45.0;
    }

    angles
}

// #7a8d9e
fn color_good() -> Color { Color::from_rgb8(0x77, 0x8b, 0x9d) }

// #8c8c8c
fn color_neutral() -> Color { Color::from_rgb8(0x8c, 0x8c, 0x8c) }

// #a37974
fn color_evil() -> Color { Color::from_rgb8(0xa3, 0x79, 0x74) }

// #5e595a
fn color_border() -> Color { Color::from_rgb8(0x5e, 0x59, 0x6a) }