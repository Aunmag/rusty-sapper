use crate::models::ui::element::Element;
use crate::models::ui::element::ElementEvent;
use std::any::Any;
use termwiz::input::InputEvent;
use termwiz::input::KeyCode;
use termwiz::input::KeyEvent;

pub struct Button {
    pub label: &'static str,
    pub is_active: bool,
    pub events: Vec<ElementEvent>,
}

impl Button {
    pub fn new(label: &'static str, is_active: bool) -> Self {
        return Button { label, is_active, events: Vec::new() };
    }
}

impl Element for Button {
    fn update(&mut self, input: &InputEvent) {
        if self.is_active {
            if let InputEvent::Key(KeyEvent {key: KeyCode::Enter, ..}) = input {
                self.events.push(ElementEvent::ButtonPressed(self.label));
            }
        }
    }

    fn render(&self) -> String {
        return format!(" - {} ", self.label);
    }

    fn label(&self) -> &str {
        return self.label;
    }

    fn is_active(&self) -> bool {
        return self.is_active;
    }

    fn events_mut(&mut self) -> &mut Vec<ElementEvent> {
        return &mut self.events;
    }

    fn as_any(&mut self) -> &mut dyn Any {
        return self;
    }
}
