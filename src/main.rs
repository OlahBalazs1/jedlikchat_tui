mod networking;

use std::sync::mpsc::Receiver;

use crate::networking::{Recipient, Session};
mod utils;
use crate::utils::read;

use color_eyre::Result;
use crossterm::event::{self, Event};
use ratatui::prelude::*;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Paragraph};
use ratatui::{layout, DefaultTerminal, Frame};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();

    let (session, receiver) = Session::new("WalInhabitant", "127.0.0.1:12345")?;
    let mut terminal = ratatui::init();
    let result = run(terminal, receiver);
    ratatui::restore();

    loop {
        match session.send(Recipient::All, &read()) {
            Ok(_) => {}
            Err(e) => panic!("{e}"),
        }
    }
}

fn handle_event(app: &mut App, event: networking::Event) {
    todo!()
}
fn run(mut terminal: DefaultTerminal, event_stream: Receiver<networking::Event>) -> Result<()> {
    for event in event_stream {
        println!("{:?}", event);

        terminal.draw(render)?;
        if matches!(event::read()?, Event::Key(_)) {
            break
        }
    }
    Ok(())
}
fn render(frame: &mut Frame) {
    let vertical_split =
        layout::Layout::horizontal([Constraint::Percentage(100), Constraint::Percentage(33)])
            .split(frame.area());

    let left_split =
        layout::Layout::vertical([Constraint::Percentage(100), Constraint::Percentage(33)])
            .split(vertical_split[0]);
    let message_block = Block::bordered()
        .title("messages")
        .border_style(Style::new().fg(Color::Red));
    let users_block = Block::bordered()
        .title("users")
        .border_style(Style::new().fg(Color::Red));

    let input_block = Block::new();
    let messages = Paragraph::new(
        "gfdjklgdjhgjlkfdglhfdgjfldkgfd
        gfdjklgdjhgjlkfdglhfdgjfldkgfdaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaadg
        gfdjklgdjhgjlkfdglhfdgjfldkgfdfdg
        gfdjklgdjhgjlkfdglhfdgjfldkgfdfdgdf
        2025-02-16T15:54:01fdsf
        async fn dsa
        (arg: Type) -> RetType {
            todo!();
        }",
    )
    .wrap(ratatui::widgets::Wrap { trim: true });

    frame.render_widget(&message_block, left_split[0]);
    frame.render_widget(messages, message_block.inner(left_split[0]));
    frame.render_widget(input_block, left_split[1]);
    frame.render_widget(users_block, vertical_split[1]);
}

struct App {
    messages: Vec<String>,
    users: Vec<String>,
}

impl App {
    fn new() -> Self {
        Self {
            messages: vec![],
            users: vec![],
        }
    }
}
