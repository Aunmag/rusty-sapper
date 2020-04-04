use crate::models::ui::Element;
use crate::models::ui::EMPTY_LABEL;
use std::any::Any;

pub struct Spacer {}

impl Spacer {
    pub fn new() -> Self {
        return Spacer {};
    }
}

impl Element for Spacer {
    fn render(&self) -> String {
        return EMPTY_LABEL.to_string();
    }

    fn get_label(&self) -> &str {
        return EMPTY_LABEL;
    }

    fn is_active(&self) -> bool {
        return true;
    }

    fn is_selectable(&self) -> bool {
        return false;
    }

    fn as_any(&mut self) -> &mut dyn Any {
        return self;
    }
}
