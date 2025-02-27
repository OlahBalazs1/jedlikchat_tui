mod networking;

use crate::networking::{Recipient, Session};

mod application;
use application::{ActiveEventLoop, Application, EventLoop, GeneralEvent};

use color_eyre::Result;
use networking::Event;
use ratatui::prelude::*;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Paragraph};
use ratatui::{layout, DefaultTerminal, Frame};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut event_loop = EventLoop::new();
    let mut app = App::new();

    event_loop.run_app(&mut app);

    Ok(())
}

struct App {
    messages: Vec<String>,
    users: Vec<String>,
    session: Option<Session>,
    terminal: Option<DefaultTerminal>,
}

impl App {
    fn new() -> Self {
        Self {
            messages: vec![],
            session: None,
            users: vec![],
            terminal: None,
        }
    }
}

impl Application for App {
    fn handle_event(&mut self, event_loop: &mut ActiveEventLoop, event: GeneralEvent) {
        println!("eventprint");
        event_loop.request_redraw();

        match event {
            GeneralEvent::Input(input) => {
                event_loop.exit();
            }
            GeneralEvent::Networking(event) => match event{
                Event::UsersList(users) => {
                    self.users = users
                }
                Event::MessageReceived(message) => {
                    self.messages.push(format!("{}: {}", message.sender, message.message));
                }
                _ => event_loop.exit(),

            }
            _ => {
                event_loop.exit();
            }
        }
    }

    fn init(&mut self, event_loop: &mut ActiveEventLoop) {
        event_loop.request_redraw();
    }

    fn render(&self, frame: &mut Frame) {

        let vertical_split = Layout::new(Direction::Horizontal, Constraint::from_percentages([100, 33]));
        let message_block = Block::bordered().title("Messages");
        let users_block = Block::bordered().title("Users");
    }
}
