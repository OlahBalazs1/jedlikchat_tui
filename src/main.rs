mod networking;

use crate::networking::{Recipient, Session};

mod application;
use application::{ActiveEventLoop, Application, EventLoop, GeneralEvent};

use color_eyre::owo_colors::OwoColorize;
use color_eyre::Result;
use crossterm::event::KeyCode;
use networking::Event;
use ratatui::layout::Flex;
use ratatui::prelude::*;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Padding, Paragraph};
use ratatui::{layout, DefaultTerminal, Frame};

use tui_input::backend::crossterm::EventHandler;
use tui_input::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut event_loop = EventLoop::new();
    let mut app = App::new();

    event_loop.run_app(&mut app);

    Ok(())
}

#[derive(Clone, Copy)]
enum ConnectingSelected {
    Name,
    Ip,
    Port,
    Connect,
}

#[derive(Clone, Copy)]
enum ConnectedSelected {
    Messages,
    Users,
    Recipient,
    Send,
}

#[derive(Clone, Copy)]
enum AppState {
    ConnectingToNetwork(ConnectingSelected),
    Connected(ConnectedSelected),
}

struct App {
    messages: Vec<String>,
    users: Vec<String>,
    session: Option<Session>,
    terminal: Option<DefaultTerminal>,
    state: AppState,

    username_input: Input,
    ip_input: Input,
    port_input: Input,
    message_input: Input,
    recipient_input: Input,
}

impl App {
    fn new() -> Self {
        Self {
            messages: vec![],
            session: None,
            users: vec![],
            terminal: None,
            state: AppState::Connected(ConnectedSelected::Send),
            username_input: "".into(),
            ip_input: "".into(),
            port_input: "".into(),
            message_input: "".into(),
            recipient_input: "".into(),
        }
    }

    #[inline]
    fn get_current_input_mut(&mut self) -> Option<&mut Input> {
        match self.state {
            AppState::Connected(selected) => match selected {
                ConnectedSelected::Send => Some(&mut self.message_input),
                ConnectedSelected::Recipient => Some(&mut self.recipient_input),
                _ => None,
            },
            AppState::ConnectingToNetwork(selected) => match selected {
                ConnectingSelected::Name => Some(&mut self.username_input),
                ConnectingSelected::Ip => Some(&mut self.ip_input),
                ConnectingSelected::Port => Some(&mut self.port_input),
                _ => None,
            },
        }
    }

    #[inline]
    fn get_current_input(&self) -> Option<&Input> {
        match self.state {
            AppState::Connected(selected) => match selected {
                ConnectedSelected::Send => Some(&self.message_input),
                ConnectedSelected::Recipient => Some(&self.recipient_input),
                _ => None,
            },
            AppState::ConnectingToNetwork(selected) => match selected {
                ConnectingSelected::Name => Some(&self.username_input),
                ConnectingSelected::Ip => Some(&self.ip_input),
                ConnectingSelected::Port => Some(&self.port_input),
                _ => None,
            },
        }
    }
}

impl Application for App {
    fn handle_event(&mut self, event_loop: &mut ActiveEventLoop, event: GeneralEvent) {
        event_loop.request_redraw();

        match event {
            GeneralEvent::Input(event) => {
                match &event {
                    crossterm::event::Event::Key(key) => {
                        if key.code == KeyCode::Esc {
                            event_loop.exit();
                        }
                    }
                    _ => {}
                }
                if let Some(input_field) = self.get_current_input_mut() {
                    let _ = input_field.handle_event(&event);
                }
            }

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
        let selected = Style::new().fg(ratatui::style::Color::LightGreen);
        let unselected = Style::new().fg(ratatui::style::Color::Green);
        let mut selected_block_rect: Option<Rect> = None;
        match self.state {
            AppState::ConnectingToNetwork(select) => {
                let block = Block::bordered().padding(Padding::horizontal(1));

                let centered_area = center(frame.area(), 50, 50);

                let mut name_block = Block::bordered().title("Username").style(unselected);
                let mut ip_block = Block::bordered().title("IP").style(unselected);
                let mut port_block = Block::bordered().title("Port").style(unselected);

                let [name_area, ip_area, lower_area] = Layout::vertical([
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                ])
                .margin(1)
                .areas(centered_area);

                let [port_area, _, connect_area] = Layout::horizontal([
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                ])
                .areas(lower_area);

                let mut connect_block = Block::bordered().title("ConnectPlaceholder");

                match select {
                    ConnectingSelected::Name => {
                        selected_block_rect = Some(name_block.inner(name_area));
                        name_block = name_block.style(selected)
                    }
                    ConnectingSelected::Ip => {
                        selected_block_rect = Some(ip_block.inner(ip_area));
                        ip_block = ip_block.style(selected)
                    }
                    ConnectingSelected::Port => {
                        selected_block_rect = Some(port_block.inner(port_area));
                        port_block = port_block.style(selected)
                    }
                    ConnectingSelected::Connect => {
                        selected_block_rect = None;
                        connect_block = connect_block.style(selected)
                    }
                }

                frame.render_widget(name_block, name_area);
                frame.render_widget(ip_block, ip_area);
                frame.render_widget(port_block, port_area);
                frame.render_widget(connect_block, connect_area);
                frame.render_widget(block, centered_area);
            }
            AppState::Connected(select) => {
                let [left_area, users_area] =
                    Layout::horizontal([Constraint::Percentage(100), Constraint::Percentage(20)])
                        .areas(frame.area());
                let [message_area, sending_area] =
                    Layout::vertical([Constraint::Percentage(100), Constraint::Percentage(20)])
                        .areas(left_area);
                let [recipient_area, message_send_area] =
                    Layout::horizontal([Constraint::Percentage(20), Constraint::Percentage(100)])
                        .areas(sending_area);

                let mut message_block = Block::bordered().title("Messages");
                let mut users_block = Block::bordered().title("Users");
                let mut recipient_block = Block::bordered().title("Recipient");
                let mut message_send_block = Block::bordered().title("Send");

                match select {
                    ConnectedSelected::Messages => {
                        selected_block_rect = Some(message_block.inner(message_area));
                        message_block = message_block.style(selected)
                    }
                    ConnectedSelected::Users => {
                        selected_block_rect = Some(users_block.inner(users_area));
                        users_block = users_block.style(selected)
                    }
                    ConnectedSelected::Recipient => {
                        selected_block_rect = Some(recipient_block.inner(recipient_area));
                        recipient_block = recipient_block.style(selected)
                    }
                    ConnectedSelected::Send => {
                        selected_block_rect = Some(message_send_block.inner(message_send_area));
                        message_send_block = message_send_block.style(selected)
                    }
                }

                frame.render_widget(message_block, message_area);
                frame.render_widget(users_block, users_area);
                frame.render_widget(&recipient_block, recipient_area);
                frame.render_widget(&message_send_block, message_send_area);
                frame.render_widget(
                    self.message_input.value(),
                    message_send_block.inner(message_send_area),
                );
                frame.render_widget(
                    self.recipient_input.value(),
                    recipient_block.inner(recipient_area),
                );
            }
        }
        if let Some(rect) = selected_block_rect {
            let x = self.get_current_input().unwrap().cursor() as u16 + rect.x;
            frame.set_cursor_position((x, rect.y));
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
