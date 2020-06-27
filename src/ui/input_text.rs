use crate::ui::Element;
use crate::ui::Event;
use std::any::Any;
use termwiz::input::InputEvent;
use termwiz::input::KeyCode;
use termwiz::input::KeyEvent;

const TOOLTIP: &'static str = "Type in some text. Press `Backspace` to remove \
    the last letter. You can also pase text from clipboard.";

pub struct InputText {
    pub label: &'static str,
    pub value: String,
    pub tooltip_extra: Option<&'static str>,
    events: Vec<Event>,
}

impl InputText {
    pub fn new(label: &'static str, tooltip_extra: Option<&'static str>) -> Self {
        return InputText {
            label,
            value: String::new(),
            tooltip_extra,
            events: Vec::new(),
        };
    }
}

impl Element for InputText {
    fn update(&mut self, input: &InputEvent) {
        match input {
            InputEvent::Key(KeyEvent {
                key: KeyCode::Char(c),
                ..
            }) => {
                self.value.push(*c);
                self.events.push(Event::PageChanged);
            }
            InputEvent::Key(KeyEvent {
                key: KeyCode::Backspace,
                ..
            }) => {
                self.value.pop();
                self.events.push(Event::PageChanged);
            }
            InputEvent::Paste(text) => {
                self.value += text;
            }
            _ => {}
        }
    }

    fn render(&self) -> String {
        return format!(" $ {} : {} ", self.label, self.value);
    }

    fn pull_events_into(&mut self, buffer: &mut Vec<Event>) {
        buffer.append(&mut self.events);
    }

    fn get_label(&self) -> &str {
        return self.label;
    }

    fn is_active(&self) -> bool {
        return true;
    }

    fn is_selectable(&self) -> bool {
        return true;
    }

    fn get_tooltip(&self) -> Option<&'static str> {
        return Some(TOOLTIP);
    }

    fn get_tooltip_extra(&self) -> Option<&'static str> {
        return self.tooltip_extra;
    }

    fn as_any(&mut self) -> &mut dyn Any {
        return self;
    }
}
