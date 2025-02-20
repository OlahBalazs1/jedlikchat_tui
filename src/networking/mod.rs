//! Module for connecting to a server running Csicsay's chat program
//! Usage: Make a new session with Session::new()

use std::{
    error::Error,
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
    sync::mpsc::{channel, Sender, TryRecvError},
    thread::{self, JoinHandle},
};

type EventHandler = &'static (dyn FnMut(Event) + Send + Sync);
pub struct Session<'a> {
    name: &'a str,
    socket: TcpStream,
    receive_join: Option<JoinHandle<()>>,
    event_handler: EventHandler,
    stop_sender: Option<Sender<()>>,
}

pub enum Event {
    Quit,
    UsersList(Vec<String>),
    MessageSent(MessageInformation),
    MessageReceived(MessageInformation),
}
pub struct MessageInformation {
    sender: String,
    recipient: Recipient,
    message: String,
}
impl MessageInformation {
    pub fn is_all(&self) -> bool {
        self.recipient == Recipient::All
    }
}

#[derive(PartialEq, Eq, Clone)]
pub enum Recipient {
    All,
    Id(String),
    This,
}

impl<'a> Session<'a> {
    pub fn new(
        name: &'a str,
        socket: &str,
        event_handler: EventHandler,
    ) -> Result<Self, Box<dyn Error>> {
        let address = socket.to_socket_addrs().unwrap().next().unwrap();
        let mut connection = TcpStream::connect(address)?;
        let _ = connection.write(format!("ID:{}", name).as_bytes());
        let mut active_connection = Session {
            name,
            stop_sender: None,
            socket: connection,
            receive_join: None,
            event_handler,
        };

        Ok(Session {
            receive_join: Some(active_connection.start_receiving()),
            ..active_connection
        })
    }
    fn start_receiving(&mut self) -> JoinHandle<()> {
        let mut socket = self.socket.try_clone().unwrap();
        let (stop_sender, stop_reciever) = channel();
        self.stop_sender = Some(stop_sender);
        let event_handler = self.event_handler;

        thread::spawn(move || loop {
            match stop_reciever.try_recv() {
                Ok(_) => {
                    event_handler(Event::Quit);
                }
                Err(TryRecvError::Disconnected) => {
                    panic!()
                }
                Err(TryRecvError::Empty) => {}
            }
            let mut buf = [0; 1024];
            match socket.read(&mut buf) {
                Ok(n) => {
                    if n == 0 {
                        panic!("Server probably unreachable, idk I'm not flash")
                    }
                }
                Err(e) => panic!("{e}"),
            }
            let raw_message = String::from_utf8(buf.to_vec()).unwrap();
            for line in raw_message.split("\n") {
                let mut message = line.split(":");
                match message.next().unwrap() {
                    "MSG" => {
                        let information;
                        let mut sender_data =
                            message.next().unwrap().split(" ").collect::<Vec<&str>>();
                        if *sender_data.last().unwrap() == "(ALL)" {
                            sender_data.pop();
                            information = MessageInformation {
                                sender: sender_data.iter().fold("".to_owned(), |init, i| init + i),
                                recipient: Recipient::This,
                                message: message.next().unwrap().to_string(),
                            };
                        } else {
                            information = MessageInformation {
                                sender: sender_data.iter().fold("".to_owned(), |init, i| init + i),
                                recipient: Recipient::This,
                                message: message.next().unwrap().to_string(),
                            };
                        }
                        event_handler(Event::MessageReceived(information))
                    }
                    "USERS" => event_handler(Event::UsersList(
                        message
                            .next()
                            .unwrap()
                            .split(",")
                            .map(|i| i.to_owned())
                            .collect(),
                    )),
                    _ => {}
                }
            }
        })
    }
    pub fn send(&self, recipient: Recipient, message: &str) -> Result<usize, Box<dyn Error>> {
        let message = match recipient.clone() {
            Recipient::All => format!("ALL:{message}"),
            Recipient::Id(id) => format!("SEND:{id}:{message}"),
            Recipient::This => unreachable!(),
        };
        (self.event_handler)(Event::MessageSent(MessageInformation {
            sender: self.name.to_string(),
            recipient: recipient.clone(),
            message: message.clone(),
        }));
        Ok(self.socket.try_clone().unwrap().write(message.as_bytes())?)
    }

    pub fn stop(&mut self) {
        let Some(ref sender) = self.stop_sender else {
            unreachable!()
        };
        sender.send(());
    }
}
