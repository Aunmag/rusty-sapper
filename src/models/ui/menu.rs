use crate::models::ui::element::ElementEvent;
use crate::models::ui::page::Page;
use termwiz::input::InputEvent;
use termwiz::input::KeyCode;
use termwiz::input::KeyEvent;
use termwiz::surface::Surface;

pub struct Menu {
    pages: Vec<Page>,
    pages_history: Vec<usize>,
    events: Vec<ElementEvent>,
}

impl Menu {
    pub fn new() -> Self {
        return Menu {
            pages: Vec::new(),
            pages_history: Vec::new(),
            events: Vec::new(),
        };
    }

    pub fn update(&mut self, input: &InputEvent) {
        if let InputEvent::Key(KeyEvent {key: KeyCode::Escape, ..}) = input {
            if !self.pages_history.is_empty() {
                self.back();
            }
        }

        if let Some(page) = self.get_page_current() {
            page.update(input);
        }
    }

    // TODO: Not mut
    pub fn render(&mut self) -> Surface {
        return self.get_page_current()
            .map(|p| p.render())
            .unwrap_or_else(|| Surface::new(1, 1));
    }

    pub fn add(&mut self, page: Page) {
        self.pages.push(page);
    }

    pub fn open(&mut self, label: &str) {
        for (i, page) in self.pages.iter().enumerate() {
            if std::ptr::eq(page.label, label) {
                if i != *self.pages_history.last().unwrap_or(&0) {
                    self.pages_history.push(i);
                    self.events.push(ElementEvent::MenuChanged);
                }

                break;
            }
        }
    }

    pub fn back(&mut self) {
        if !self.pages_history.is_empty() {
            if let Some(current) = self.get_page_current() {
                current.reset_cursor();
            }

            self.pages_history.pop();
            self.events.push(ElementEvent::MenuChanged);
        }
    }

    pub fn fetch_page_mut(&mut self, label: &str) -> Option<&mut Page> {
        for page in &mut self.pages {
            if std::ptr::eq(page.label, label) {
                return Some(page);
            }
        }

        return None;
    }

    pub fn get_page_current(&mut self) -> Option<&mut Page> {
        return self.pages.get_mut(*self.pages_history.last().unwrap_or(&0));
    }

    pub fn is_on_base_page(&self) -> bool {
        return self.pages_history.len() == 0;
    }

    pub fn pull_events(&mut self) -> Vec<ElementEvent> {
        let mut events = Vec::new();

        events.append(&mut self.events);

        for page in &mut self.pages {
            events.append(&mut page.events);

            for element in &mut page.elements {
                events.append(element.events_mut());
            }
        }

        return events;
    }
}
