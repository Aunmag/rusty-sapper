use crate::models::ui::element::Element;
use crate::models::ui::element::ElementEvent;
use std::any::Any;
use termwiz::input::InputEvent;

const LABEL: &'static str = "";

pub struct Spacer {
    events: Vec<ElementEvent>,
}

impl Spacer {
    pub fn new() -> Self {
        return Spacer { events: Vec::new() };
    }
}

impl Element for Spacer {
    fn update(&mut self, _input: &InputEvent) {}

    fn render(&self) -> String {
        return LABEL.to_string();
    }

    fn label(&self) -> &str {
        return LABEL;
    }

    fn is_active(&self) -> bool {
        return false;
    }

    fn events_mut(&mut self) -> &mut Vec<ElementEvent> {
        return &mut self.events;
    }

    fn as_any(&mut self) -> &mut dyn Any {
        return self;
    }
}
