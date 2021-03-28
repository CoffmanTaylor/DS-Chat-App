use std::{net::Ipv6Addr, str::FromStr};

use chat_application::{
    context::{self, Ctx},
    ChatResponse,
};
use crossterm::event::{EventStream, KeyCode, KeyModifiers};
use ds_libs::{address::Address, Context, InitializeNode};
use futures::{select, Stream, StreamExt};
use interface::Interface;
use simple_server::user::Client;

mod interface;

#[tokio::main]
async fn main() {
    let name = "Taylor".to_string();

    let mut interface = Interface::new();
    let node_address = Address::new((Ipv6Addr::from_str("::1").unwrap(), 8080));
    let mut node = Client::new(Address::new_test_id(1), None);
    let mut ctx = Ctx::new(("::1", 8080)).await;

    let mut terminal_events = key_events().fuse();
    let mut client_events = ctx.event_stream().boxed().fuse();

    node.init(&mut Context::new(node_address, &mut ctx));

    loop {
        select! {
            event = terminal_events.select_next_some() => {
                match event {
                    crossterm::event::Event::Resize(..) => interface.render(),
                    crossterm::event::Event::Key(key) => {
                        match key.code {
                            KeyCode::Esc => {
                                break;
                            },
                            KeyCode::Backspace => {
                                interface.pop_input();
                            },
                            KeyCode::Enter => {
                                let _message = interface.clear_input();
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
            event = client_events.select_next_some() => {
                match event {
                    context::Event::Response(res) => {
                        match res.result {
                            ChatResponse::Latest(history, _) => {
                                interface.set_history(history);
                            },
                            _ => {},
                        }
                    },
                    _ => {},
                }
            },
        };
    }

    interface.close();
}

fn key_events() -> impl Stream<Item = crossterm::event::Event> {
    // get the reader
    EventStream::new().map(|e| e.unwrap())
}
