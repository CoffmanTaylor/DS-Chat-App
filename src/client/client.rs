use std::task::Poll;

use chat_application::Message;
use crossterm::event::{Event, EventStream, KeyCode, KeyModifiers};
use futures::{
    executor::block_on,
    select,
    stream::{self},
    Stream, StreamExt,
};
use interface::Interface;

mod interface;

fn main() {
    let mut interface = Interface::new();

    let mut terminal_events = key_events().fuse();
    let mut messages = messages();

    block_on(async {
        loop {
            select! {
                event = terminal_events.select_next_some() => {
                    match event {
                        Event::Resize(..) => interface.render(),
                        Event::Key(key) => {
                            match key.code {
                                KeyCode::Esc => {
                                    break;
                                },
                                KeyCode::Backspace => {
                                    interface.pop_input();
                                },
                                KeyCode::Enter => {
                                    // TODO: send the request.
                                    interface.clear_input();
                                },
                                KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                                    // Brake on a ctrl-c
                                    break;
                                },
                                KeyCode::Char(c) => {
                                    interface.push_input(c);
                                },
                                _ => {},
                            }
                        },
                        _ => {},
                    }
                },
            };
        }
    });

    interface.close();
}

fn key_events() -> impl Stream<Item = Event> {
    // get the reader
    EventStream::new().map(|e| e.unwrap())
}

fn messages() -> impl Stream<Item = Message> {
    stream::empty()
}
