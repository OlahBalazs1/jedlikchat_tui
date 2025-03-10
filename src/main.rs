mod networking;

use crate::networking::{Recipient, Session};

mod application;
use application::{ActiveEventLoop, Application, EventLoop, GeneralEvent};

use color_eyre::owo_colors::OwoColorize;
use color_eyre::Result;
use networking::Event;
use ratatui::layout::Flex;
use ratatui::prelude::*;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Padding, Paragraph};
use ratatui::{layout, DefaultTerminal, Frame};

use tui_input::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut event_loop = EventLoop::new();
    let mut app = App::new();

    event_loop.run_app(&mut app);

    Ok(())
}

#[derive(Clone,Copy)]
enum ConnectingSelected{
    Name,
    Ip,
    Port,
    Connect
}

#[derive(Clone,Copy)]
enum ConnectedSelected{
    Messages,
    Users,
    Recipient,
    Send,
}

#[derive(Clone,Copy)]
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
    recipient_input: Input

}

impl App {
    fn new() -> Self {
        Self {
            messages: vec![],
            session: None,
            users: vec![],
            terminal: None,
            state: AppState::Connected(ConnectedSelected::Messages),
            username_input: "".into(),
            ip_input: "".into(),
            port_input: "".into(),
            message_input: "".into(),
            recipient_input: "".into()
        }
    }

    fn current_input(&mut self) -> Option<&mut Input>{
        match self.state{
            AppState::Connected(selected) => match selected {
                ConnectedSelected::Messages => Some(&mut self.message_input),
                ConnectedSelected::Recipient => Some(&mut self.recipient_input),
                _ => None
            }
            AppState::ConnectingToNetwork(selected) => match selected {
                ConnectingSelected::Name => Some(&mut self.username_input),
                ConnectingSelected::Ip => Some(&mut self.ip_input),
                ConnectingSelected::Port => Some(&mut self.port_input),
                _ => None
            }
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
        let selected = Style::new().fg(ratatui::style::Color::LightGreen);
        let unselected = Style::new().fg(ratatui::style::Color::Green);
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
                    ConnectingSelected::Name => {name_block = name_block.style(selected)}
                    ConnectingSelected::Ip => ip_block = ip_block.style(selected),
                    ConnectingSelected::Port => port_block = port_block.style(selected),
                    ConnectingSelected::Connect => connect_block = connect_block.style(selected),
                }

                frame.render_widget(name_block, name_area);
                frame.render_widget(ip_block, ip_area);
                frame.render_widget(port_block, port_area);
                frame.render_widget(connect_block, connect_area);
                frame.render_widget(block, centered_area);
            }
            AppState::Connected(select) => {
                let [left_area, users_area] = Layout::horizontal([
                    Constraint::Percentage(100), Constraint::Percentage(20)
                ]
                ).areas(frame.area());
                let [message_area, sending_area] = Layout::vertical([Constraint::Percentage(100), Constraint::Percentage(20)]).areas(left_area);
                let [recipient_area, message_send_area] = Layout::horizontal([Constraint::Percentage(20), Constraint::Percentage(100)]).areas(sending_area);
                
                
                let mut message_block = Block::bordered().title("Messages");
                let mut users_block = Block::bordered().title("Users");
                let mut recipient_block = Block::bordered().title("Recipient");
                let mut message_send_block = Block::bordered().title("Send");

                match select {
                    ConnectedSelected::Messages => message_block = message_block.style(selected),
                    ConnectedSelected::Users => users_block = users_block.style(selected),
                    ConnectedSelected::Recipient => recipient_block = recipient_block.style(selected),
                    ConnectedSelected::Send => message_send_block = message_send_block.style(selected),
                }

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
