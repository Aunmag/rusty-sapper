use crate::event::Event;
use crate::event::EventData;
use crate::event::EventManager;
use crate::event::EVENT_SIZE;
use crate::game::Game;
use crate::sapper::Sapper;
use crate::sapper::SapperBehavior;
use crate::field::DiscoveryResult;
use crate::net::LocalMessage;
use crate::net::Message;
use crate::net::NetHandler;
use crate::net::NO_SENDER;
use crate::utils;
use async_std::net::TcpListener;
use async_std::net::TcpStream;
use async_std::prelude::*;
use futures::executor::block_on;
use futures::future::FutureExt;
use futures::pin_mut;
use futures::select;
use std::net::SocketAddr;
use std::thread::JoinHandle;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

const CHANNELS_BUFFER_SIZE: usize = 128; // TODO: Learn more and tweak

pub struct Server {
    game: Game,
    sender: Sender<Message>,
    receiver: Receiver<Message>,
    clients: Vec<ServerClient>,
    thread: Option<JoinHandle<()>>,
    pub error: Option<String>,
}

pub struct ServerClient {
    stream: TcpStream,
    address: SocketAddr,
}

impl Server {
    pub async fn new(address: SocketAddr, game: Game) -> Result<Server, String> {
        let (server_sender, runner_receiver) = mpsc::channel(CHANNELS_BUFFER_SIZE);
        let (runner_sender, server_receiver) = mpsc::channel(CHANNELS_BUFFER_SIZE);

        let listener = TcpListener::bind(address)
            .await
            .map_err(|e| format!("{}", e))
            ?;

        let thread = std::thread::Builder::new()
            .name("server".to_string())
            .spawn(move || block_on(Server::run(listener, runner_sender, runner_receiver)))
            .map_err(|e| format!("{}", e))
            ?;

        return Ok(Server {
            game,
            sender: server_sender,
            receiver: server_receiver,
            clients: Vec::new(),
            thread: Some(thread),
            error: None,
        });
    }

    async fn run(listener: TcpListener, sender: Sender<Message>, receiver: Receiver<Message>) {
        let mut listener = listener;
        let mut sender = sender;
        let mut receiver = receiver;
        let mut error = None;

        {
            let connections_listening = Server::run_connections_listening(&mut listener, &mut sender).fuse();
            let receiver_listening = Server::run_receiver_listening(&mut receiver).fuse();
            pin_mut!(connections_listening, receiver_listening);

            select! {
                result = connections_listening => match result {
                    Ok(()) => {}
                    Err(update_error) => {
                        error = Some(update_error);
                    }
                },
                result = receiver_listening => match result {
                    Ok(()) => {}
                    Err(update_error) => {
                        error = Some(update_error);
                    }
                },
            }
        }

        if let Some(error) = error {
            let _ = sender.send(Message::Local(LocalMessage::Error(error))).await; // TODO: Maybe handle result
        }

        utils::log(&"[SERVER] Has terminated gracefully".to_string());
    }

    async fn run_connections_listening(
        listener: &TcpListener,
        sender: &mut Sender<Message>,
    ) -> Result<(), String> {
        loop {
            match listener.accept().await {
                Ok((stream, address)) => {
                    utils::log(&format!("[SERVER] {} has connected", address));

                    match sender.send(Message::Local(LocalMessage::Connection(ServerClient {
                        stream: stream.clone(),
                        address,
                    }))).await {
                        Ok(()) => {}
                        Err(error) => {
                            return Err(format!("{}", error));
                        }
                    }

                    // TODO: Maybe do that inside server
                    match sender.send(Message::Event(Event {
                        data: EventData::SapperConnect,
                        source: Some(address),
                        target: None,
                    })).await {
                        Ok(()) => {}
                        Err(error) => {
                            return Err(format!("{}", error));
                        }
                    }

                    let sender_clone = sender.clone();

                    std::thread::spawn(move || {
                        block_on(Server::run_client_listening(address, stream, sender_clone))
                    });
                }
                Err(error) => {
                    return Err(format!("{}", error));
                }
            }
        }
    }

    async fn run_client_listening(address: SocketAddr, stream: TcpStream, sender: Sender<Message>) {
        // TODO: Find a way to gracefully terminate a client listening
        let mut stream = stream;
        let mut sender = sender;

        loop {
            let mut message = vec![0; EVENT_SIZE];

            match stream.read_exact(&mut message).await {
                Ok(()) => {
                    match sender.send(Message::Event(Event {
                        data: EventData::decode(&message),
                        source: Some(address),
                        target: None,
                    })).await {
                        Ok(()) => {
                            utils::log(&format!(
                                "[SERVER] << {:?} from {}",
                                EventData::decode(&message),
                                address,
                            ));
                        }
                        Err(error) => {
                            utils::log(&format!("[SERVER] {}", error));
                            break;
                        }
                    }
                }
                Err(error) => {
                    // TODO: Send local event to remove client
                    utils::log(&format!(
                        "[SERVER] {} disconnected. Reason: {}",
                        address,
                        error,
                    ));

                    break;
                }
            }
        }
    }

    async fn run_receiver_listening(receiver: &mut Receiver<Message>) -> Result<(), String> {
        loop {
            match receiver.recv().await {
                Some(Message::Event(_)) => {
                    unreachable!();
                }
                Some(Message::Local(LocalMessage::Connection(_))) => {
                    unreachable!();
                }
                Some(Message::Local(LocalMessage::Stop)) => {
                    return Ok(());
                }
                Some(Message::Local(LocalMessage::Error(_))) => {
                    unreachable!();
                }
                None => {
                    return Err(NO_SENDER.to_string());
                }
            }
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = block_on(self.sender.send(Message::Local(LocalMessage::Stop))); // TODO: Handle error

        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}

impl NetHandler for Server {
    fn before_update(&mut self) {
        // TODO: Consider async
        loop {
            match self.receiver.try_recv() {
                Ok(Message::Event(event)) => {
                    self.game.events.fire(event.data, event.source, event.target);
                }
                Ok(Message::Local(LocalMessage::Connection(client))) => {
                    self.clients.push(client);
                }
                Ok(Message::Local(LocalMessage::Stop)) => {
                    unreachable!();
                }
                Ok(Message::Local(LocalMessage::Error(error))) => {
                    self.error = Some(error);
                }
                Err(TryRecvError::Closed) => {
                    self.error = Some(NO_SENDER.to_string());
                }
                Err(TryRecvError::Empty) => {
                    break;
                }
            }
        }
    }

    fn send(&mut self, event: Event) {
        // TODO: Check is this method's approach correct enough

        let encoded = event.data.encode();
        let mut clients = Vec::with_capacity(0);

        std::mem::swap(&mut self.clients, &mut clients);

        clients = clients.into_iter().filter_map(|mut client| {
            if
                event.target.map(|t| t == client.address).unwrap_or(true)
                &&
                event.source.map(|t| t != client.address).unwrap_or(true)
            {
                utils::log(&format!(
                    "[SERVER] >> {:?} to {:?}",
                    event.data,
                    event.target,
                ));

                // TODO: Consider concurrent async
                return block_on(client.stream.write_all(&encoded)).map(|_| client).ok();
            } else {
                return Some(client);
            }
        }).collect::<Vec<_>>();

        std::mem::swap(&mut self.clients, &mut clients);
    }

    fn on_sapper_connect(&mut self, address: SocketAddr) -> bool {
        let new_sapper_id = self.game.sappers.len() as u8;

        self.game.sappers.push(Sapper::new(
            new_sapper_id,
            SapperBehavior::Remote,
            self.game.field.generate_random_position(),
            0.0,
        ));

        self.game.events.fire(
            EventData::SapperConnectResponse { id: new_sapper_id },
            None,
            Some(address),
        );

        for sapper in self.game.sappers.iter() {
            let mut target = None;

            if sapper.get_id() != new_sapper_id {
                target = Some(address);
            }

            if !sapper.is_alive {
                self.game.events.fire(
                    EventData::SapperDie {
                        id: sapper.get_id(),
                    },
                    None,
                    target,
                );
            }

            if sapper.score != 0 {
                self.game.events.fire(
                    EventData::SapperScore {
                        id: sapper.get_id(),
                        score: sapper.score as u16,
                    },
                    None,
                    target,
                );
            }

            self.game.events.fire(
                EventData::SapperSpawn {
                    id: sapper.get_id(),
                    position: sapper.get_position() as u16,
                },
                None,
                target,
            );
        }

        for (position, cell) in self.game.field.get_cells().iter().enumerate() {
            if let Some(mines_around) = cell.mines_around {
                self.game.events.fire(
                    EventData::CellDiscover {
                        position: position as u16,
                        mines_around,
                    },
                    None,
                    Some(address),
                );
            }

            if cell.is_exploded {
                self.game.events.fire(
                    EventData::CellExplode {
                        position: position as u16,
                    },
                    None,
                    Some(address),
                );
            }
        }

        self.game.events.fire(
            EventData::FieldCreate {
                size: self.game.field.get_size() as u8,
            },
            None,
            Some(address),
        );

        return true;
    }

    fn on_sapper_discover(&mut self, id: u8, position: u16) -> bool {
        struct SapperData { id: u8, score: u16 };
        let mut sapper_data = None;

        if let Some(sapper) = self.game.get_sapper_mut(id) {
            sapper_data = Some(SapperData {
                id: sapper.get_id(),
                score: sapper.score as u16,
            });
        }

        if let Some(sapper_data) = sapper_data {
            match self.game.field.discover(position as usize, &mut self.game.events) {
                DiscoveryResult::Success => {
                    self.game.events.fire(
                        EventData::SapperScore {
                            id: sapper_data.id,
                            score: sapper_data.score + 1,
                        },
                        None,
                        None,
                    );
                }
                DiscoveryResult::Failure => {
                    self.game.events.fire(
                        EventData::SapperDie {
                            id: sapper_data.id,
                        },
                        None,
                        None,
                    );
                }
                DiscoveryResult::AlreadyDiscovered => {}
            }

            return true;
        } else {
            return false;
        }
    }

    fn get_game_mut(&mut self) -> &mut Game {
        return &mut self.game;
    }

    fn get_events_mut(&mut self) -> &mut EventManager {
        return &mut self.game.events;
    }

    fn is_server(&self) -> bool {
        return true;
    }
}
