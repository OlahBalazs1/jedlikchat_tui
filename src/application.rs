use crate::networking::{self, Session};
use cancel_token::CancelToken;
use crossterm::event::{self};
use ratatui::{DefaultTerminal, Frame};
use std::error::Error;
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;
type Res<T> = Result<T, Box<dyn Error>>;
pub trait Application {
    fn handle_event(&mut self, event_loop: &mut ActiveEventLoop, event: GeneralEvent);

    fn init(&mut self, event_loop: &mut ActiveEventLoop);

    fn render(&mut self, frame: &mut Frame);

    fn redraw(&mut self, mut terminal: DefaultTerminal) -> Res<()> {
        terminal.draw(|frame| self.render(frame))?;
        Ok(())
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum GeneralEvent {
    Networking(networking::Event),
    Input(crossterm::event::Event),
    RedrawRequested,
    Exit,
}

pub struct ActiveEventLoop {
    cancel_token: CancelToken,
    event_sender: Sender<GeneralEvent>,
    network_handle: Option<JoinHandle<()>>,
    network_session: Option<Session>,
    input_handle: Option<JoinHandle<()>>,
    event_receiver: Option<Receiver<GeneralEvent>>,
}

impl ActiveEventLoop {
    pub fn stop_network_session(&mut self) {
        if let Some(session) = self.network_session.take() {
            session.stop();
        }
    }
    fn set_exit_flag(&self) {
        self.cancel_token.set();
    }
    pub fn request_redraw(&self) {
        self.event_sender.send(GeneralEvent::RedrawRequested).unwrap_or_else(|_| {
            eprintln!("Couldn't request redraw, there is something wrong with the event loop");
            self.cancel_token.set()});
    }
    pub fn new() -> Self {
        let (event_sender, event_receiver) = channel();
        let mut active_loop = Self {
            cancel_token: CancelToken::new(),
            event_sender,
            network_handle: None,
            network_session: None,
            input_handle: None,
            event_receiver: Some(event_receiver),
        };
        active_loop.start_input_listener();
        active_loop
    }

    fn start_application(mut self, application: &mut impl Application) {
        application.init(&mut self);
        let receiver = self.event_receiver.take().unwrap();
        let mut terminal = ratatui::init();

        for event in receiver.iter() {
            match event.clone() {
                GeneralEvent::Exit => {
                    self.set_exit_flag();
                    self.stop_network_session();
                    break;
                }
                GeneralEvent::RedrawRequested => {
                    let _ = terminal.draw(|frame| application.render(frame));
                }
                event => {
                    let _ = application.handle_event(&mut self, event);
                }
            };
        }
        self.input_handle.take().unwrap().join().unwrap_or_else(|_| eprintln!("Couldn't join on handle"));
        if let Some(network_handle) = self.network_handle.take() {
            network_handle.join().unwrap_or_else(|_| eprintln!("Couldn't join on handle"));
        }
    }
    pub fn exit(&self) {
        self.set_exit_flag();
        self.event_sender.send(GeneralEvent::Exit).unwrap_or_else(|_| {
            eprintln!("Couldn't request redraw, there is something wrong with the event loop");
            self.cancel_token.set()});
        ratatui::restore();
    }
    pub fn start_network_session(&mut self, name: &str, socket: &str) -> Res<()> {
        let (session, network_receiver) = Session::new(name, socket, self.cancel_token.clone())?;
        self.network_handle = Some(self.wrap_network(network_receiver, self.event_sender.clone()));
        self.network_session = Some(session);
        Ok(())
    }
    fn start_input_listener(&mut self) {
        let event_sender = self.event_sender.clone();
        let exit = self.cancel_token.clone();

        self.input_handle = Some(thread::spawn(move || loop {
            if *exit {
                break;
            }
            let is_event = match event::poll(Duration::from_millis(50)) {
                Ok(is_event) => is_event,
                Err(_) => panic!(),
            };
            if is_event {
                let event = event::read().unwrap();
                let _ = event_sender.send(GeneralEvent::Input(event)).unwrap();
            }
        }))
    }

    fn wrap_network(
        &self,
        network_receiver: Receiver<networking::Event>,
        general_sender: Sender<GeneralEvent>,
    ) -> JoinHandle<()> {
        let exit = self.cancel_token.clone();
        thread::spawn(move || loop {
            if *exit {
                break;
            }
            match network_receiver.recv_timeout(Duration::from_millis(50)) {
                Ok(event) => general_sender
                    .send(GeneralEvent::Networking(event))
                    .unwrap(),
                Err(RecvTimeoutError::Disconnected) => {
                    break;
                }
                Err(RecvTimeoutError::Timeout) => {}
            }
        })
    }
}

pub struct EventLoop {}
impl EventLoop {
    pub fn new() -> Self {
        EventLoop {}
    }
    pub fn run_app<T: Application>(&mut self, application: &mut T) -> Res<()> {
        ActiveEventLoop::new().start_application(application);

        ratatui::restore();

        Ok(())
    }
}
