use iced::{button, Align, Button, Column, Element, Sandbox, Settings, Text, Row, Length, Container};
mod data;

pub fn main() {
    let kaylin = data::read_entity_from_path("samples/kaylin.json").unwrap();
    //println!("Reading Kaylin data:\n{:#?}", kaylin);
    Counter::run(Settings::default())
}

#[derive(Default)]
struct Counter {
    value: i32,
    increment_button: button::State,
    decrement_button: button::State,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    IncrementPressed,
    DecrementPressed,
}

type MyRow<'a> = Row<'a, Message>;

impl Sandbox for Counter {
    type Message = Message;

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("Counter - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::IncrementPressed => {
                self.value += 1;
            }
            Message::DecrementPressed => {
                self.value -= 1;
            }
        }
    }

    fn view(&mut self) -> Element<Message> {

        let menu = Column::new()
            .align_items(Align::Start)
            .width(Length::from(50))
            .push(Text::new("Party"));

        let characters = Column::new()
            .width(Length::from(150))
            .push(Text::new("Kaylin"))
            .push(Text::new("Solace"))
            .push(Text::new("Amiri"))
            .push(Text::new("Ember"));

        // Statistics

        let main_stats = Row::new()
            .width(Length::Fill)
            .height(Length::from(50))
            .push(Text::new("Money: 38747G"))
            .push(Text::new("Experience: 38747"))
            .push(Text::new("Alignment: Neutral Good"));

        let abilities_stats = Column::new()
            .height(Length::Fill)
            .push(Text::new("STR 10"))
            .push(Text::new("DEX 15"));

        let combat_stats = Column::new()
            .push(Text::new("CMB 5"))
            .push(Text::new("CMD 7"));

        let skills_stats = Column::new()
            .push(Text::new("Athletics 7"))
            .push(Text::new("Mobility 13"));

        let statistics = Row::new()
            .push(abilities_stats)
            .push(combat_stats)
            .push(skills_stats);

        let character = Column::new()
            .width(Length::Fill)
            .push(main_stats)
            .push(statistics);

        Row::new()
            .push(menu)
            .push(characters)
            .push(character)
            .into()
    }
}
