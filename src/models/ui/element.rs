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
    fn events_mut(&mut self) -> &mut Vec<ElementEvent>;
    fn as_any(&mut self) -> &mut dyn Any;
}
