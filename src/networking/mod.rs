use cancel_token::CancelToken;
use std::{
    error::Error,
    io::{BufRead, BufReader, Write},
    net::{TcpStream, ToSocketAddrs},
    sync::mpsc::{channel, Receiver, Sender},
    thread::{self, JoinHandle},
};

pub struct Session {
    name: String,
    socket: TcpStream,
    receive_join: Option<JoinHandle<()>>,
    event_sender: Sender<Event>,
    cancel_token: CancelToken,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Event {
    Quit,
    UsersList(Vec<String>),
    MessageSent(MessageInformation),
    MessageReceived(MessageInformation),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MessageInformation {
    pub sender: String,
    pub recipient: Recipient,
    pub message: String,
}
impl MessageInformation {
    pub fn is_all(&self) -> bool {
        self.recipient == Recipient::All
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Recipient {
    All,
    Id(String),
    This,
}

impl Session {
    pub fn new(
        name: &str,
        socket: &str,
        cancel_token: CancelToken,
    ) -> Result<(Self, Receiver<Event>), Box<dyn Error>> {
        let address = socket.to_socket_addrs().unwrap().next().unwrap();
        let mut connection = TcpStream::connect(address)?;
        let _ = connection.write(format!("ID:{}", name).as_bytes());
        let (event_sender, event_receiver) = channel();
        let mut active_connection = Session {
            name: name.to_string(),
            cancel_token,
            socket: connection,
            receive_join: None,
            event_sender: event_sender.clone(),
        };
        let receive_join = active_connection.start_receiving(event_sender);

        Ok((
            Session {
                receive_join: Some(receive_join),
                ..active_connection
            },
            event_receiver,
        ))
    }
    fn start_receiving(&mut self, sender: Sender<Event>) -> JoinHandle<()> {
        let socket = self.socket.try_clone().unwrap();
        let reader = BufReader::new(socket);

        let exit = self.cancel_token.clone();

        thread::spawn(move || {
            let lines = reader.lines();
            for line in lines {
                if *exit {
                    sender.send(Event::Quit).unwrap();
                    break;
                }
                let line = match line {
                    Ok(line) => line,
                    Err(e) => panic!("{e}"),
                };
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
                        sender.send(Event::MessageReceived(information)).unwrap();
                    }
                    "USERS" => sender
                        .send(Event::UsersList(
                            message
                                .next()
                                .unwrap()
                                .split(",")
                                .map(|i| i.to_owned())
                                .collect(),
                        ))
                        .unwrap(),
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
        (Event::MessageSent(MessageInformation {
            sender: self.name.to_string(),
            recipient: recipient.clone(),
            message: message.clone(),
        }));
        Ok(self.socket.try_clone().unwrap().write(message.as_bytes())?)
    }

    pub fn stop(&self) {
        self.cancel_token.set();
    }
}
