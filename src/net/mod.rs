pub mod client;
pub mod server;

use crate::event::Event;
use crate::event::EventData;
use crate::event::EventManager;
use crate::game::Game;
use crate::net::server::ServerClient;
use std::net::SocketAddr;
use termwiz::input::InputEvent;

pub const NO_SENDER: &'static str = &"Receiver's sender no longer exists.";

pub enum Message {
    // TODO: Maybe allow send encoded events
    Event(Event),
    Local(LocalMessage),
}

pub enum LocalMessage {
    Connection(ServerClient),
    Stop,
    Error(String),
}

pub trait NetHandler {
    fn before_update(&mut self) {}

    fn update(&mut self, input: Option<&InputEvent>) {
        self.before_update();

        let mut local_events = self.get_game_mut().update(input);

        if !self.is_server() {
            while let Some(event) = local_events.pop() {
                self.send(event);
            }
        }

        let mut suspended = Vec::new();

        while let Some(event) = self.get_events_mut().pop() {
            if let Some(event) = self.on_event(event) {
                suspended.push(event);
            }
        }

        // TODO: Optimize with swap
        self.get_events_mut().fire_all(&mut suspended);
    }

    // TODO: Async, also allow sent multiple events in parallel
    fn send(&mut self, event: Event);

    fn on_event(&mut self, event: Event) -> Option<Event> {
        let was_processed;

        match event.data {
            EventData::SapperConnect => {
                if let Some(address) = event.source {
                    self.on_sapper_connect(address);
                }

                was_processed = true;
            }
            EventData::SapperConnectResponse { id } => {
                was_processed = self.on_sapper_connect_response(id);
            }
            EventData::SapperSpawn { id, position } => {
                was_processed = self.on_sapper_spawn(id, position);
            }
            EventData::SapperMove { id, position } => {
                was_processed = self.on_sapper_move(id, position);
            }
            EventData::SapperDiscover { id, position } => {
                was_processed = self.on_sapper_discover(id, position);
            }
            EventData::SapperScore { id, score } => {
                was_processed = self.on_sapper_score(id, score);
            }
            EventData::SapperDie { id } => {
                was_processed = self.on_sapper_die(id);
            }
            EventData::FieldCreate { size } => {
                was_processed = self.on_field_create(size);
            }
            EventData::CellDiscover { position, mines_around } => {
                was_processed = self.on_cell_discover(position, mines_around);
            }
            EventData::CellExplode { position } => {
                was_processed = self.on_cell_explode(position);
            }
        }

        if was_processed {
            if self.is_server() {
                self.send(event);
            }

            return None;
        } else {
            return Some(event);
        }
    }

    fn on_sapper_connect(&mut self, _address: SocketAddr) -> bool {
        return true;
    }

    fn on_sapper_connect_response(&mut self, _id: u8) -> bool {
        return true;
    }

    fn on_sapper_spawn(&mut self, _id: u8, _position: u16) -> bool {
        return true;
    }

    fn on_sapper_move(&mut self, id: u8, position: u16) -> bool {
        if let Some(sapper) = self.get_game_mut().get_sapper_mut(id) {
            sapper.position = position as usize;
            return true;
        } else {
            return false;
        }
    }

    fn on_sapper_discover(&mut self, _id: u8, _position: u16) -> bool {
        return true;
    }

    fn on_sapper_score(&mut self, id: u8, score: u16) -> bool {
        if let Some(sapper) = self.get_game_mut().get_sapper_mut(id) {
            sapper.score = score;
            return true;
        } else {
            return false;
        }
    }

    fn on_sapper_die(&mut self, id: u8) -> bool {
        if let Some(sapper) = self.get_game_mut().get_sapper_mut(id) {
            sapper.is_alive = false;
            return true;
        } else {
            return false;
        }
    }

    fn on_field_create(&mut self, _size: u8) -> bool {
        return true;
    }

    fn on_cell_discover(&mut self, _position: u16, _mines_around: u8) -> bool {
        return true;
    }

    fn on_cell_explode(&mut self, _position: u16) -> bool {
        return true;
    }

    fn get_game_mut(&mut self) -> &mut Game;

    fn get_events_mut(&mut self) -> &mut EventManager;

    fn is_server(&self) -> bool;
}
