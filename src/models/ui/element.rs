use std::any::Any;
use termwiz::input::InputEvent;

pub enum ElementEvent {
    ButtonPressed(&'static str),
    PageChanged,
    MenuChanged,
}

pub trait Element {
    fn update(&mut self, input: &InputEvent);

    fn render(&self) -> String;

    fn label(&self) -> &str;

    fn is_active(&self) -> bool;

    fn get_tooltip(&self) -> Option<&'static str> {
        return None;
    }

    fn get_tooltip_extra(&self) -> Option<&'static str> {
        return None;
    }

    fn get_tooltip_full(&self) -> Option<String> {
        let mut tooltip_full = None;

        if let Some(description) = self.get_tooltip_extra() {
            tooltip_full = Some(description.to_string());
        }

        if let Some(tooltip) = self.get_tooltip() {
            if let Some(tooltip_full) = tooltip_full.as_mut() {
                tooltip_full.push_str(" ");
                tooltip_full.push_str(tooltip);
            } else {
                tooltip_full = Some(tooltip.to_string());
            }
        }

        return tooltip_full;
    }

    fn events_mut(&mut self) -> &mut Vec<ElementEvent>;

    fn as_any(&mut self) -> &mut dyn Any;
}
