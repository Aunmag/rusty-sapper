use crate::ui::Element;
use crate::ui::EMPTY_LABEL;
use std::any::Any;

pub struct Spacer {}

impl Spacer {
    pub const fn new() -> Self {
        return Self {};
    }
}

impl Element for Spacer {
    fn render(&self) -> String {
        return EMPTY_LABEL.to_owned();
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
