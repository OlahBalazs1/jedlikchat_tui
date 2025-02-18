//! Module for connecting to a server running Csicsay's chat program 
//! Usage: start by making an inactive Session with Session::new(). Next, define the functions that will handle events with the provided methods. Activate the session by connecting to the server with the Session::connect() method.
//! You can send and recieve messages with an ActiveSession 

// TODO: unfuck everything up

use std::{
    error::Error,
    io::{Read, Write},
    net::{SocketAddr, TcpStream, ToSocketAddrs},
    sync::{Arc, RwLock},
    thread::{self, JoinHandle},
};
pub struct Session {
    quit_handler: Option<&'static (dyn Fn() + Send + Sync)>,
    message_sent_handler: Option<&'static (dyn Fn(MessageInformation) + Send + Sync)>,
    message_received_handler: Option<&'static (dyn Fn(MessageInformation) + Send + Sync)>,
    users_received_handler: Option<&'static (dyn Fn(Vec<String>) + Send + Sync)>,
}
impl Session UwU :3 {
    pub fn new() -> Self {
        Self {
            quit_handler: None,
            message_sent_handler: None,
            message_received_handler: None,
            users_received_handler: None,
        }
    }
    pub fn connect<'a>(
        self,
        name: &'a str,
        socket: &str,
    ) -> Result<(ActiveSession<'a>, JoinHandle<()>), Box<dyn Error>> {
        let address = socket.to_socket_addrs().unwrap().next().unwrap();
        let mut connection = TcpStream::connect(address)?;
        let _ = connection.write(format!("ID:{}", name).as_bytes());
        let active_session = ActiveSession {
            name,
            stop: Arc::new(false.into()),
            socket: connection,
            quit_handler: self.quit_handler,
            message_sent_handler: self.message_sent_handler,
            message_received_handler: self.message_received_handler,
            users_received_handler: self.users_received_handler,
        };

        let handle = active_session.start_receiving();

        Ok((active_session, handle))
    }
    pub fn with_quit_handler(
        self,
        quit_handler: &'static (dyn Fn() + std::marker::Send + Sync),
    ) -> Self {
        Self {
            quit_handler: Some(quit_handler),
            ..self
        }
    }
    pub fn with_message_sent_handler(
        self,
        message_sent_handler: &'static (dyn Fn(MessageInformation) + Send + Sync),
    ) -> Self {
        Self {
            message_sent_handler: Some(message_sent_handler),
            ..self
        }
    }
    pub fn with_message_received_handler(
        self,
        message_received_handler: &'static (dyn Fn(MessageInformation) + Send + Sync),
    ) -> Self {
        Self {
            message_received_handler: Some(message_received_handler),
            ..self
        }
    }
    pub fn with_users_received_handler(
        self,
        users_received_handler: &'static (dyn Fn(Vec<String>) + Send + Sync),
    ) -> Self {
        Self {
            users_received_handler: Some(users_received_handler),
            ..self
        }
    }
}

pub struct ActiveSession<'a> {
    stop: Arc<RwLock<bool>>,
    name: &'a str,
    socket: TcpStream,
    quit_handler: Option<&'static (dyn Fn() + Send + Sync)>,
    message_sent_handler: Option<&'static (dyn Fn(MessageInformation) + Send + Sync)>,
    message_received_handler: Option<&'static (dyn Fn(MessageInformation) + Send + Sync)>,
    users_received_handler: Option<&'static (dyn Fn(Vec<String>) + Send + Sync)>,
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

impl<'a> ActiveSession<'a> {
    fn start_receiving(&self) -> JoinHandle<()> {
        let mut socket = self.socket.try_clone().unwrap();
        let stop = self.stop.clone();
        let message_received_handler = self.message_received_handler;
        let users_received_handler = self.users_received_handler;
        let quit_handler = self.quit_handler;

        thread::spawn(move || loop {
            let stop = stop.read().unwrap();
            if let Some(quit_handler) = quit_handler {
                if *stop {
                    quit_handler();

                    panic!();
                }
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
                        if let Some(message_received_handler) = message_received_handler {
                            let information;
                            let mut sender_data =
                                message.next().unwrap().split(" ").collect::<Vec<&str>>();
                            if *sender_data.last().unwrap() == "(ALL)" {
                                sender_data.pop();
                                information = MessageInformation {
                                    sender: sender_data
                                        .iter()
                                        .fold("".to_owned(), |init, i| init + i),
                                    recipient: Recipient::This,
                                    message: message.next().unwrap().to_string(),
                                };
                            } else {
                                information = MessageInformation {
                                    sender: sender_data
                                        .iter()
                                        .fold("".to_owned(), |init, i| init + i),
                                    recipient: Recipient::This,
                                    message: message.next().unwrap().to_string(),
                                };
                            }
                            message_received_handler(information)
                        }
                    }
                    "USERS" => {
                        if let Some(users_received_handler) = users_received_handler {
                            users_received_handler(
                                message
                                    .next()
                                    .unwrap()
                                    .split(",")
                                    .map(|i| i.to_owned())
                                    .collect(),
                            )
                        }
                    }
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
        if let Some(message_sent_handler) = self.message_sent_handler {
            message_sent_handler(MessageInformation {
                sender: self.name.to_string(),
                recipient: recipient.clone(),
                message: message.clone(),
            })
        }
        Ok(self.socket.try_clone().unwrap().write(message.as_bytes())?)
    }
}
