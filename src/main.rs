mod networking;

use crate::networking::{Recipient, Session};

mod application;
use application::{ActiveEventLoop, Application, EventLoop, GeneralEvent};

use color_eyre::Result;
use networking::Event;
use ratatui::layout::Flex;
use ratatui::prelude::*;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Padding, Paragraph};
use ratatui::{layout, DefaultTerminal, Frame};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut event_loop = EventLoop::new();
    let mut app = App::new();

    event_loop.run_app(&mut app);

    Ok(())
}

enum AppState {
    ConnectingToNetwork,
    Connected,
}

struct App {
    messages: Vec<String>,
    users: Vec<String>,
    session: Option<Session>,
    terminal: Option<DefaultTerminal>,
    state: AppState,
}

impl App {
    fn new() -> Self {
        Self {
            messages: vec![],
            session: None,
            users: vec![],
            terminal: None,
            state: AppState::Connected,
        }
    }
}

impl Application for App {
    fn handle_event(&mut self, event_loop: &mut ActiveEventLoop, event: GeneralEvent) {
        event_loop.request_redraw();

        match event {
            GeneralEvent::Input(input) => match input {
                crossterm::event::Event::Key(_) => event_loop.exit(),
                _ => {}
            },
            GeneralEvent::Networking(event) => match event {
                Event::UsersList(users) => self.users = users,
                Event::MessageReceived(message) => {
                    self.messages
                        .push(format!("{}: {}", message.sender, message.message));
                }
                _ => event_loop.exit(),
            },
            _ => {
                event_loop.exit();
            }
        }
    }

    fn init(&mut self, event_loop: &mut ActiveEventLoop) {
        event_loop.request_redraw();
    }

    fn render(&self, frame: &mut Frame) {
        match self.state {
            AppState::ConnectingToNetwork => {
                let block = Block::bordered().padding(Padding::horizontal(1));

                let centered_area = center(frame.area(), 50, 50);

                let name_block = Block::bordered().title("Username");
                let ip_block = Block::bordered().title("IP");
                let host_block = Block::bordered().title("Port");

                let [name_area, ip_area, lower_area] = Layout::vertical([
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                ])
                .margin(1)
                .areas(centered_area);

                let [host_area, _, connect_area] = Layout::horizontal([
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                ])
                .areas(lower_area);

                let connect_block = Block::bordered().title("ConnectPlaceholder");

                frame.render_widget(name_block, name_area);
                frame.render_widget(ip_block, ip_area);
                frame.render_widget(host_block, host_area);
                frame.render_widget(connect_block, connect_area);
                frame.render_widget(block, centered_area);
            }
            AppState::Connected => {
                let [left_area, users_area] = Layout::horizontal([
                    Constraint::Percentage(100), Constraint::Percentage(20)
                ]
                ).areas(frame.area());
                let [message_area, sending_area] = Layout::vertical([Constraint::Percentage(100), Constraint::Percentage(20)]).areas(left_area);
                let [recipient_area, message_send_area] = Layout::horizontal([Constraint::Percentage(20), Constraint::Percentage(100)]).areas(sending_area);
                
                
                let message_block = Block::bordered().title("Messages");
                let users_block = Block::bordered().title("Users");

                let recipient_block = Block::bordered().title("Recipient");
                let message_send_block = Block::bordered().title("Send");

                frame.render_widget(message_block, message_area);
                frame.render_widget(users_block, users_area);
                frame.render_widget(recipient_block, recipient_area);
                frame.render_widget(message_send_block, message_send_area);
            }
        }
    }
}
fn center(area: Rect, horizontal: u16, vertical: u16) -> Rect {
    let horizontal_constraint = Constraint::Percentage(horizontal);
    let vertical_constraint = Constraint::Percentage(vertical);
    let [area] = Layout::horizontal([horizontal_constraint])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical_constraint])
        .flex(Flex::Center)
        .areas(area);
    area
}
