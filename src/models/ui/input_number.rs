use crate::models::ui::element::Element;
use crate::models::ui::element::ElementEvent;
use std::any::Any;
use termwiz::input::InputEvent;
use termwiz::input::KeyCode;
use termwiz::input::KeyEvent;

pub struct InputNumber {
    pub label: &'static str,
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub events: Vec<ElementEvent>,
}

impl InputNumber {
    pub fn new(
        label: &'static str,
        value: f64,
        min: f64,
        max: f64,
        step: f64,
    ) -> Self {
        return InputNumber { label, value, min, max, step, events: Vec::new() };
    }

    fn normalize(&mut self) {
        if self.value < self.min {
            self.value = self.min;
        } else if self.value > self.max {
            self.value = self.max;
        }
    }
}

impl Element for InputNumber {
    fn update(&mut self, input: &InputEvent) {
        match input {
            InputEvent::Key(KeyEvent {key: KeyCode::LeftArrow, ..}) => {
                self.value -= self.step;
                self.normalize();
                self.events.push(ElementEvent::PageChanged);
            }
            InputEvent::Key(KeyEvent {key: KeyCode::RightArrow, ..}) => {
                self.value += self.step;
                self.normalize();
                self.events.push(ElementEvent::PageChanged);
            }
            _ => {}
        }
    }

    fn render(&self) -> String {
        if self.value.fract() == 0.0 {
            return format!(" * {} : {} ", self.label, self.value);
        } else {
            return format!(" * {} : {:.2} ", self.label, self.value);
        }
    }

    fn label(&self) -> &str {
        return self.label;
    }

    fn is_active(&self) -> bool {
        return true;
    }

    fn events_mut(&mut self) -> &mut Vec<ElementEvent> {
        return &mut self.events;
    }

    fn as_any(&mut self) -> &mut dyn Any {
        return self;
    }
}
