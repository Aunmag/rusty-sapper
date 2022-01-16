use crate::event::Event;
use crate::event::EventData;
use crate::event::EventManager;
use crate::event::EVENT_SIZE;
use crate::field::Field;
use crate::game::Game;
use crate::net::LocalMessage;
use crate::net::Message;
use crate::net::NetHandler;
use crate::net::NO_SENDER;
use crate::sapper::Sapper;
use crate::sapper::SapperBehavior;
use crate::utils;
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

pub struct Client {
    pub game: Game,
    sender: Sender<Message>,
    receiver: Receiver<Message>,
    thread: Option<JoinHandle<()>>,
    pub error: Option<String>,
}

impl Client {
    pub async fn new(address: SocketAddr) -> Result<Self, String> {
        let (client_sender, runner_receiver) = mpsc::channel(CHANNELS_BUFFER_SIZE);
        let (runner_sender, client_receiver) = mpsc::channel(CHANNELS_BUFFER_SIZE);

        let stream = TcpStream::connect(address)
            .await
            .map_err(|e| format!("{}", e))?;

        let thread = std::thread::Builder::new()
            .name("client".to_owned())
            .spawn(move || block_on(Self::run(stream, runner_sender, runner_receiver)))
            .map_err(|e| format!("{}", e))?;

        return Ok(Self {
            game: Game::new(Field::new(0, 0.0), Vec::new()),
            sender: client_sender,
            receiver: client_receiver,
            thread: Some(thread),
            error: None,
        });
    }

    #[allow(warnings)] // TODO: Resolve
    async fn run(stream: TcpStream, sender: Sender<Message>, receiver: Receiver<Message>) {
        let mut stream_reading = stream;
        let mut stream_writing = stream_reading.clone(); // TODO: Avoid cloning
        let mut sender = sender;
        let mut receiver = receiver;
        let mut error = None;

        {
            let stream_listening =
                Client::run_stream_listening(&mut stream_reading, &mut sender).fuse();

            let receiver_listening =
                Client::run_receiver_listening(&mut receiver, &mut stream_writing).fuse();

            pin_mut!(stream_listening, receiver_listening);

            select! {
                result = stream_listening => match result {
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

        #[allow(clippy::let_underscore_drop)] // TODO: Resolve
        if let Some(error) = error {
            let _ = sender
                .send(Message::Local(LocalMessage::Error(error)))
                .await; // TODO: Maybe handle result
        }

        utils::log(&"[CLIENT] Has terminated gracefully".to_string());
    }

    async fn run_stream_listening(
        stream: &mut TcpStream,
        sender: &mut Sender<Message>,
    ) -> Result<(), String> {
        loop {
            let mut message = vec![0; EVENT_SIZE];

            match stream.read_exact(&mut message).await {
                Ok(()) => {
                    match sender
                        .send(Message::Event(Event {
                            data: EventData::decode(&message),
                            source: None,
                            target: None,
                        }))
                        .await
                    {
                        Ok(()) => {
                            utils::log(&format!("[CLIENT] << {:?}", EventData::decode(&message)));
                        }
                        Err(error) => {
                            return Err(format!("{}", error));
                        }
                    }
                }
                Err(error) => {
                    return Err(format!("{}", error));
                }
            }
        }
    }

    async fn run_receiver_listening(
        receiver: &mut Receiver<Message>,
        stream: &mut TcpStream,
    ) -> Result<(), String> {
        loop {
            match receiver.recv().await {
                Some(Message::Event(event)) => match stream.write_all(&event.data.encode()).await {
                    Ok(()) => {
                        utils::log(&format!("[CLIENT] >> {:?}", event.data));
                    }
                    Err(error) => {
                        return Err(format!("{}", error));
                    }
                },
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
                    return Err(NO_SENDER.to_owned());
                }
            }
        }
    }
}

impl Drop for Client {
    #[allow(clippy::let_underscore_drop)] // TODO: Resolve
    fn drop(&mut self) {
        let _ = block_on(self.sender.send(Message::Local(LocalMessage::Stop))); // TODO: Handle error

        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}

impl NetHandler for Client {
    fn before_update(&mut self) {
        // TODO: Consider async
        loop {
            match self.receiver.try_recv() {
                Ok(Message::Event(event)) => {
                    self.game
                        .events
                        .fire(event.data, event.source, event.target);
                }
                Ok(Message::Local(LocalMessage::Connection(_))) => {
                    unreachable!();
                }
                Ok(Message::Local(LocalMessage::Stop)) => {
                    unreachable!();
                }
                Ok(Message::Local(LocalMessage::Error(error))) => {
                    self.error = Some(error);
                }
                Err(TryRecvError::Closed) => {
                    self.error = Some(NO_SENDER.to_owned());
                }
                Err(TryRecvError::Empty) => {
                    break;
                }
            }
        }
    }

    #[allow(clippy::let_underscore_drop)] // TODO: Resolve
    fn send(&mut self, event: Event) {
        let _ = block_on(self.sender.send(Message::Event(event))); // TODO: Handle error
    }

    fn on_sapper_connect_response(&mut self, id: u8) -> bool {
        if let Some(sapper) = self.game.get_sapper_mut(id) {
            sapper.behavior = SapperBehavior::Player;
            return true;
        } else {
            return false;
        }
    }

    fn on_sapper_spawn(&mut self, id: u8, position: u16) -> bool {
        self.game
            .sappers
            .push(Sapper::new(id, SapperBehavior::Remote, position, 0.0));

        return true;
    }

    fn on_field_create(&mut self, size: u8) -> bool {
        self.game.field = Field::new(size, 0.0);
        return true;
    }

    fn on_cell_discover(&mut self, position: u16, mines_around: u8) -> bool {
        if let Some(cell) = self.game.field.get_cell_mut(position) {
            cell.mines_around = Some(mines_around);
            return true;
        }

        return false;
    }

    fn on_cell_explode(&mut self, position: u16) -> bool {
        if let Some(cell) = self.game.field.get_cell_mut(position) {
            cell.is_exploded = true;
            return true;
        }

        return false;
    }

    fn get_game_mut(&mut self) -> &mut Game {
        return &mut self.game;
    }

    fn get_events_mut(&mut self) -> &mut EventManager {
        return &mut self.game.events;
    }

    fn is_server(&self) -> bool {
        return false;
    }
}
