use serde::Deserialize;
use serde::Serialize;
use std::net::SocketAddr;

pub const EVENT_SIZE: usize = 4; // (8 + data[16]) / 8

// TODO: Remove clone drive
#[derive(Clone)]
pub struct Event {
    pub data: EventData,
    pub source: Option<SocketAddr>,
    pub target: Option<SocketAddr>,
}

// TODO: Remove clone drive
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum EventData {
    SapperConnect,
    SapperConnectResponse { id: u8 },
    SapperSpawn { id: u8, position: u16 },
    SapperMove { id: u8, position: u16 },
    SapperDiscover { id: u8, position: u16 },
    SapperScore { id: u8, score: u16 },
    SapperDie { id: u8 },
    FieldCreate { size: u8 },
    CellDiscover { position: u16, mines_around: u8 },
    CellExplode { position: u16 },
}

impl EventData {
    pub fn encode(&self) -> Vec<u8> {
        let mut encoded = Vec::with_capacity(EVENT_SIZE);

        // Remove useless 3 bytes since it's enough 1 byte to serialize an enum invariant
        for (i, byte) in bincode::serialize(&self).unwrap().iter().enumerate() {
            if i != 1 && i != 2 && i != 3 {
                encoded.push(*byte);
            }
        }

        // Fill the unused space since we need fixed-size messages
        while encoded.len() < EVENT_SIZE {
            encoded.push(0);
        }

        return encoded;
    }

    pub fn decode(data: &Vec<u8>) -> EventData {
        let mut encoded = Vec::with_capacity(EVENT_SIZE);

        // Restore the 3 missing bytes which we took-off while encoding
        for (i, byte) in data.iter().enumerate() {
            encoded.push(*byte);

            if i == 0 {
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
            }
        }

        return bincode::deserialize(&encoded[..]).unwrap();
    }
}

pub struct EventManager {
    events: Vec<Event>,
}

impl EventManager {
    pub fn new() -> Self {
        return Self {
            events: Vec::new(),
        };
    }

    pub fn fire(
        &mut self,
        data: EventData,
        source: Option<SocketAddr>,
        target: Option<SocketAddr>,
    ) {
        self.events.push(Event {
            data,
            source,
            target,
        });
    }

    pub fn fire_all(&mut self, events: &mut Vec<Event>) {
        self.events.append(events);
    }

    pub fn pull(&mut self) -> Vec<Event> {
        let mut events = Vec::new();
        std::mem::swap(&mut self.events, &mut events);
        return events;
    }

    pub fn pop(&mut self) -> Option<Event> {
        return self.events.pop();
    }
}
