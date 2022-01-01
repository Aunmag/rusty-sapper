pub mod button;
pub mod input_number;
pub mod input_text;
pub mod menu;
pub mod page;
pub mod spacer;
pub mod text;

use std::any::Any;
use termwiz::input::InputEvent;

pub const EMPTY_LABEL: &str = "";

pub enum Event {
    ButtonPressed(&'static str),
    PageChanged,
    MenuChanged,
}

pub trait Element {
    fn update(&mut self, _input: &InputEvent) {}

    fn render(&self) -> String;

    fn pull_events_into(&mut self, _buffer: &mut Vec<Event>) {}

    fn get_label(&self) -> &str;

    fn is_active(&self) -> bool;

    fn is_selectable(&self) -> bool;

    fn get_tooltip(&self) -> Option<&'static str> {
        return None;
    }

    fn get_tooltip_extra(&self) -> Option<&'static str> {
        return None;
    }

    fn get_tooltip_full(&self) -> Option<String> {
        let mut tooltip_full = None;

        if let Some(description) = self.get_tooltip_extra() {
            tooltip_full = Some(description.to_owned());
        }

        if let Some(tooltip) = self.get_tooltip() {
            if let Some(tooltip_full) = tooltip_full.as_mut() {
                tooltip_full.push(' ');
                tooltip_full.push_str(tooltip);
            } else {
                tooltip_full = Some(tooltip.to_owned());
            }
        }

        return tooltip_full;
    }

    fn as_any(&mut self) -> &mut dyn Any;
}
