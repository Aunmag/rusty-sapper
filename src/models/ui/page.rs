use crate::models::ui::button::Button;
use crate::models::ui::element::Element;
use crate::models::ui::element::ElementEvent;
use termwiz::cell::AttributeChange;
use termwiz::color::AnsiColor;
use termwiz::color::ColorAttribute;
use termwiz::input::InputEvent;
use termwiz::input::KeyCode;
use termwiz::input::KeyEvent;
use termwiz::surface::Change;
use termwiz::surface::Surface;

pub const WIDTH: usize = 64;

pub struct Page {
    pub label: &'static str,
    pub elements: Vec<Box<dyn Element>>,
    pub cursor: usize,
    pub events: Vec::<ElementEvent>,
}

impl Page {
    pub fn new(label: &'static str) -> Self {
        return Page {
            label,
            elements: Vec::new(),
            cursor: 0,
            events: Vec::new(),
        };
    }

    pub fn update(&mut self, input: &InputEvent) {
        match input {
            InputEvent::Key(KeyEvent {key: KeyCode::UpArrow, ..}) => {
                self.move_cursor(false);
            }
            InputEvent::Key(KeyEvent {key: KeyCode::DownArrow, ..}) => {
                self.move_cursor(true);
            }
            _ => {}
        }

        if let Some(element) = self.elements.get_mut(self.cursor) {
            element.update(input);
        }
    }

    pub fn render(&self) -> Surface {
        let mut surface = Surface::new(WIDTH, self.elements.len() + 2);

        surface.add_change(format!("### {}\r\n", self.label));

        for (i, element) in self.elements.iter().enumerate() {
            let color = if element.is_active() {
                ColorAttribute::Default
            } else {
                AnsiColor::Grey.into()
            };

            surface.add_change(Change::Attribute(AttributeChange::Foreground(color)));
            surface.add_change(Change::Attribute(AttributeChange::Reverse(self.cursor == i)));
            surface.add_change("\r\n");
            surface.add_change(element.render());
        }

        surface.add_change(Change::Attribute(AttributeChange::Foreground(ColorAttribute::Default)));
        surface.add_change(Change::Attribute(AttributeChange::Reverse(false)));

        return surface;
    }

    pub fn reset_cursor(&mut self) {
        self.cursor = 0;

        if !self.elements.get(self.cursor).map(|c| c.is_active()).unwrap_or(false) {
            self.move_cursor(true);
        }

        self.events.push(ElementEvent::PageChanged); // TODO: Optimize
    }

    pub fn move_cursor(&mut self, offset: bool) {
        let mut cursor = self.cursor;

        loop {
            if offset {
                cursor = cursor.saturating_add(1);
            } else {
                cursor = cursor.saturating_sub(1);
            }

            if self.elements.get(cursor).map(|c| c.is_active()).unwrap_or(false) {
                self.cursor = cursor;
                self.events.push(ElementEvent::PageChanged);
                break;
            }

            if cursor == 0 || cursor + 1 >= self.elements.len() {
                break;
            }
        }
    }

    // TODO: Optimize
    pub fn fetch_element_mut(&mut self, label: &str) -> Option<&mut Box<dyn Element>> {
        for element in &mut self.elements {
            if std::ptr::eq(element.label(), label) {
                return Some(element);
            }
        }

        return None;
    }

    pub fn fetch_button_mut(&mut self, label: &str) -> Option<&mut Button> {
        return self.fetch_element_mut(label).and_then(|e| e.as_any().downcast_mut::<Button>());
    }
}
