use crate::networking::{self, Session};
use crossterm::event::{self};
use ratatui::{DefaultTerminal, Frame};
use std::error::Error;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};
type Res<T> = Result<T, Box<dyn Error>>;
pub trait Application {
    fn handle_event(&mut self, event_loop: &mut EventLoop, event: GeneralEvent);

    fn init(&mut self, terminal: DefaultTerminal);

    fn render(&self, frame: &mut Frame);

    fn redraw(&self, mut terminal: DefaultTerminal) -> Res<()> {
        terminal.draw(|frame| self.render(frame))?;
        Ok(())
    }
}
#[derive(Debug)]
pub enum GeneralEvent {
    Networking(networking::Event),
    Input(crossterm::event::Event),
    Exit,
}

pub struct EventLoop {
    exit_requested: Arc<RwLock<bool>>,
    sender: Option<Sender<GeneralEvent>>,
}

impl EventLoop {
    pub fn new() -> Self {
        EventLoop {
            exit_requested: Arc::new(RwLock::new(false)),
            sender: None,
        }
    }
    pub fn run_app<T: Application>(&mut self, application: &mut T) -> Res<()> {
        let exit = self.exit_requested.clone();
        {
            let mut exit_write = exit.write().unwrap();
            *exit_write = false;
        }
        let (sender, receiver) = channel();

        self.sender = Some(sender.clone());
        self.start_input_listener(sender.clone());

        application.init(ratatui::init());

        for event in receiver.into_iter() {
            if *exit.read().unwrap() {
                break;
            }
            application.handle_event(self, event);
        }
        ratatui::restore();

        Ok(())
    }
    pub fn start_network_session(&mut self, name: &str, socket: &str) -> Res<Session> {
        let (session, network_receiver) = Session::new(name, socket)?;
        let Some(sender) = self.sender.clone() else {
            panic!("Butafejű")
        };

        Ok(session)
    }
    pub fn stop(&mut self) {
        {
            let mut exit_writer = self.exit_requested.write().unwrap();
            *exit_writer = true
        }
    }
    fn start_input_listener(&self, event_sender: Sender<GeneralEvent>) -> JoinHandle<()> {
        let exit = self.exit_requested.clone();

        thread::spawn(move || loop {
            if *exit.read().unwrap() {
                break;
            }
            let _ = event_sender
                .send(GeneralEvent::Input(event::read().unwrap()))
                .unwrap();
        })
    }

    fn wrap_network(
        &self,
        network_receiver: Receiver<networking::Event>,
        general_sender: Sender<GeneralEvent>,
    ) -> JoinHandle<()> {
        let exit = self.exit_requested.clone();
        thread::spawn(move || {
            for event in network_receiver.into_iter() {
                if *exit.read().unwrap() {
                    break;
                }
                general_sender
                    .send(GeneralEvent::Networking(event))
                    .unwrap();
            }
        })
    }
}
