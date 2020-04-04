use crate::models::ui::Element;
use crate::models::ui::Event;
use std::any::Any;
use termwiz::input::InputEvent;
use termwiz::input::KeyCode;
use termwiz::input::KeyEvent;

pub struct Button {
    pub label: &'static str,
    pub is_active: bool,
    events: Vec<Event>,
}

impl Button {
    pub fn new(label: &'static str, is_active: bool) -> Self {
        return Button {
            label,
            is_active,
            events: Vec::new(),
        };
    }
}

impl Element for Button {
    fn update(&mut self, input: &InputEvent) {
        if self.is_active {
            if let InputEvent::Key(KeyEvent {
                key: KeyCode::Enter,
                ..
            }) = input {
                self.events.push(Event::ButtonPressed(self.label));
            }
        }
    }

    fn pull_events_into(&mut self, buffer: &mut Vec<Event>) {
        buffer.append(&mut self.events);
    }

    fn render(&self) -> String {
        return format!(" > {} ", self.label);
    }

    fn get_label(&self) -> &str {
        return self.label;
    }

    fn is_active(&self) -> bool {
        return self.is_active;
    }

    fn is_selectable(&self) -> bool {
        return true;
    }

    fn as_any(&mut self) -> &mut dyn Any {
        return self;
    }
}
