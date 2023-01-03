use crate::{data::Alignment, theme::Theme, widgets::Element};
use iced::widget::canvas::{
    self, path::Builder, Cache, Canvas, Cursor, Frame, Geometry, Path, Program,
};
use iced::{Length, Point, Rectangle};
use log::trace;

pub struct AlignmentWidget {
    // data
    alignment: Alignment,
    // view state
    pin_cache: Cache,
    background: Cache,
    debug_mode: bool,
}

#[derive(Debug, Clone)]
pub enum Message {}

impl AlignmentWidget {
    pub fn new(alignment: Alignment, debug_mode: bool) -> AlignmentWidget {
        AlignmentWidget {
            alignment,
            pin_cache: Default::default(),
            background: Default::default(),
            debug_mode,
        }
    }

    pub fn view(&self) -> Element<Message> {
        let canvas = Canvas::new(self)
            .width(Length::Units(200))
            .height(Length::Units(200));

        iced::widget::Container::new(canvas).into()
    }
}

impl Program<Message, Theme> for AlignmentWidget {
    type State = ();
    
    fn draw(&self, _state: &(), _theme: &Theme, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        // Represent a character alignment on the Alignment Wheel
        //
        // Note that this part of the iced API will change considerably when upgrading
        // from 0.1 to 0.2. As such, I tried to keep most of the draw content Layer or Drawable
        // agnostic and only relying on Path and Frame primitive.
        let alignment = self.pin_cache.draw(bounds.size(), |frame| {
            // Translate the frame such as (0, 0) in in the middle of it
            let (_, radius) = prep_frame(frame);

            // TODO Find out what is the min/max for alignments value and normalize it
            let pin_point = Point::new(
                self.alignment.x / 100f32 * radius,
                -self.alignment.y / 100f32 * radius,
            );

            // TODO Find how to do this without over drawing
            frame.fill(&Path::circle(pin_point, 9.0), colors::pin_outer_border());
            frame.fill(&Path::circle(pin_point, 8.0), colors::pin_outer_filling());
            frame.fill(&Path::circle(pin_point, 7.0), colors::pin_outer_border());
            frame.fill(&Path::circle(pin_point, 6.0), colors::pin_inner());
        });

        // Draw the alignment wheel itself (background)
        let background = self.background.draw(bounds.size(), |frame| {
            // Prepare the frame to be used in the correct coordinates
            let (inner_radius, outer_radius) = prep_frame(frame);

            // True neutral inner circle
            let neutral = Path::circle(Point::new(0.0, 0.0), inner_radius);
            frame.fill(&neutral, colors::neutral());

            // Individual sections angles
            let angles = build_wheel_angles();

            // Brush for the separations
            let thin_stroke = canvas::Stroke {
                width: 2.0,
                line_cap: canvas::LineCap::Round,
                ..canvas::Stroke::default()
            }.with_color(colors::border());

            // Paint each alignment section
            for (index, start_angle) in angles.iter().enumerate() {
                let end_angle = start_angle + 45.0;

                let color = match index {
                    0..=2 => colors::evil(),
                    4..=6 => colors::good(),
                    _ => colors::neutral(),
                };

                trace!("angle: start = {}°, end = {}°", start_angle, end_angle);

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

                trace!("points = [{:?},{:?},{:?},{:?}]", p1, p2, p3, p4);

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
                frame.stroke(&path, thin_stroke.clone());
                frame.fill(&path, color);

                // Debug mode, label cell with its index
                if self.debug_mode {
                    frame.with_save(|frame| {
                        let x = (p1.x + p3.x) / 2.0;
                        let y = (p4.y + p2.y) / 2.0;
                        frame.translate(iced::Vector::new(x, y));
                        frame.fill_text(format!("{}", index));
                    });
                }
            }
        });

        vec![background, alignment]
    }
}

/// Center the frame (use -1:1 based coordinates) and return useful radiuses
fn prep_frame(frame: &mut Frame) -> (f32, f32) {
    let radius = frame.width().min(frame.height()) / 2.0;

    let inner_radius = radius / 3.0;
    let outer_radius = (radius * 2.0) / 3.0;

    // Translate the frame to have the middle of it be coordinates (0, 0)
    frame.translate(iced::Vector::new(radius, radius));

    (inner_radius, outer_radius)
}

fn build_wheel_angles() -> [f32; 8] {
    let mut angles: [f32; 8] = [0.0; 8];

    angles[0] = 22.0;
    for i in 1..8 {
        angles[i] = angles[i - 1] + 45.0;
    }

    angles
}

/// Provide the colors used by the alignment widget
///
/// The wheel itself:
/// - Good: `#7a8d9e`
/// - Neutral: `#8c8c8c`
/// - Evil: `#a37974`
/// - Border: `#5e595a`
///
/// And the pin:
/// - pin inner: #373335 (20px wide)
/// - pin outer filling: #a0976f (2px wide)
/// - pin outer border: #736a52 (1px wide)
mod colors {
    use iced::Color;

    pub(super) fn good() -> Color {
        Color::from_rgb8(0x77, 0x8b, 0x9d)
    }

    pub(super) fn neutral() -> Color {
        Color::from_rgb8(0x8c, 0x8c, 0x8c)
    }

    pub(super) fn evil() -> Color {
        Color::from_rgb8(0xa3, 0x79, 0x74)
    }

    pub(super) fn border() -> Color {
        Color::from_rgb8(0x5e, 0x59, 0x6a)
    }

    pub(super) fn pin_inner() -> Color {
        Color::from_rgb8(0x37, 0x33, 0x35)
    }

    pub(super) fn pin_outer_filling() -> Color {
        Color::from_rgb8(0xa0, 0x97, 0x6f)
    }

    pub(super) fn pin_outer_border() -> Color {
        Color::from_rgb8(0x73, 0x6a, 0x52)
    }
}
