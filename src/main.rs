mod networking;

use crate::networking::{Recipient, Session};

mod application;
use application::{ActiveEventLoop, Application, EventLoop, GeneralEvent};

use color_eyre::Result;
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
        // event_loop.request_redraw();

        match event {
            a => {
                println!("{a:?}");
                event_loop.stop();
                event_loop.exit();
            }
        }
    }

    fn init(&mut self, event_loop: &mut ActiveEventLoop) {
        event_loop.request_redraw();
    }

    fn render(&self, frame: &mut Frame) {
        //     let vertical_split =
        //         layout::Layout::horizontal([Constraint::Percentage(100), Constraint::Percentage(33)])
        //             .split(frame.area());
        //
        //     let left_split =
        //         layout::Layout::vertical([Constraint::Percentage(100), Constraint::Percentage(33)])
        //             .split(vertical_split[0]);
        //     let message_block = Block::bordered()
        //         .title("messages")
        //         .border_style(Style::new().fg(Color::Red));
        //     let users_block = Block::bordered()
        //         .title("users")
        //         .border_style(Style::new().fg(Color::Red));
        //
        //     let input_block = Block::new();
        //     let messages = Paragraph::new(
        //     "gfdjklgdjhgjlkfdglhfdgjfldkgfd
        //     gfdjklgdjhgjlkfdglhfdgjfldkgfdaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaadg
        //     gfdjklgdjhgjlkfdglhfdgjfldkgfdfdg
        //     gfdjklgdjhgjlkfdglhfdgjfldkgfdfdgdf
        //     2025-02-16T15:54:01fdsf
        //     async fn dsa
        //     (arg: Type) -> RetType {
        //         todo!();
        //     }",
        // )
        // .wrap(ratatui::widgets::Wrap { trim: true });
        //
        //     frame.render_widget(&message_block, left_split[0]);
        //     frame.render_widget(messages, message_block.inner(left_split[0]));
        //     frame.render_widget(input_block, left_split[1]);
        //     frame.render_widget(users_block, vertical_split[1]);
    }
}
