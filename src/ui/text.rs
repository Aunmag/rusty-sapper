use crate::ui::Element;
use crate::ui::EMPTY_LABEL;
use std::any::Any;

pub struct Text {
    text: String,
}

impl Text {
    pub const fn new(text: String) -> Self {
        return Self { text };
    }
}

impl Element for Text {
    fn render(&self) -> String {
        return self.text.clone(); // TODO: Optimize
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
