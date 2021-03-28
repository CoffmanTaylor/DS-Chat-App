use std::{net::Ipv6Addr, str::FromStr, time::SystemTime};

use chat_application::{
    context::{self, Ctx},
    ChatCommand, ChatResponse, Message,
};
use crossterm::event::{EventStream, KeyCode, KeyModifiers};
use ds_libs::{address::Address, Context, HandleMessage, HandleTimer, InitializeNode};
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

    let mut ctx = Context::new(node_address, &mut ctx);

    node.init(&mut ctx);

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
                                if node.command.is_none() {
                                    let text = interface.clear_input();
                                    node.command = Some(ChatCommand::Post(Message{sender:name.clone(), text, sent_time: SystemTime::now().into()}));
                                    node.send_command(&mut ctx);
                                }
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
                        node.handle_message(&mut ctx, res);

                        // Check if the client got a response.
                        match node.response.take() {
                            Some(ChatResponse::Latest(history, _)) => {
                                node.command = None;
                                interface.set_history(history);
                            },
                            Some(_) => {
                                node.command = None;
                            },
                            _ => {},
                        }
                    },
                    context::Event::Request(_) => {
                        // Clients don't handle Requests.
                    },
                    context::Event::ResendTimer(t) => {
                        node.handle_timer(&mut ctx, t);
                    },
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
