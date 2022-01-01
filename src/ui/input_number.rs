use crate::ui::Element;
use crate::ui::Event;
use std::any::Any;
use termwiz::input::InputEvent;
use termwiz::input::KeyCode;
use termwiz::input::KeyEvent;

const TOOLTIP: &str = "Use left and right arrow keys to change the value.";

pub struct InputNumber {
    pub label: &'static str,
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub tooltip_extra: Option<&'static str>,
    events: Vec<Event>,
}

impl InputNumber {
    pub const fn new(
        label: &'static str,
        value: f64,
        min: f64,
        max: f64,
        step: f64,
        tooltip_extra: Option<&'static str>,
    ) -> Self {
        return Self {
            label,
            value,
            min,
            max,
            step,
            tooltip_extra,
            events: Vec::new(),
        };
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
            InputEvent::Key(KeyEvent {
                key: KeyCode::LeftArrow,
                ..
            }) => {
                self.value -= self.step;
                self.normalize();
                self.events.push(Event::PageChanged);
            }
            InputEvent::Key(KeyEvent {
                key: KeyCode::RightArrow,
                ..
            }) => {
                self.value += self.step;
                self.normalize();
                self.events.push(Event::PageChanged);
            }
            _ => {}
        }
    }

    fn render(&self) -> String {
        if self.value.fract() == 0.0 {
            return format!(" $ {} : {} ", self.label, self.value);
        } else {
            return format!(" $ {} : {:.2} ", self.label, self.value);
        }
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
