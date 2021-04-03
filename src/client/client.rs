use std::{env, net::SocketAddrV6, str::FromStr, time::Duration, time::SystemTime};

use chat_application::{
    context::{self, Ctx},
    ChatCommand, ChatResponse, Message,
};
use crossterm::event::{EventStream, KeyCode, KeyModifiers};
use ds_libs::{address::Address, Context, HandleMessage, HandleTimer, InitializeNode};
use futures::{select, FutureExt, Stream, StreamExt};
use interface::Interface;
use simple_server::user::Client;
use tokio::time::sleep;

mod interface;

#[tokio::main]
async fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() != 4 {
        eprintln!("You must provide only 3 arguments: <your-name> <local IPv6 address and port. Ex: [::1]:8081> <server IPv6 address and port. Ex: [::1]:8080>");
        return;
    }

    let name = args[1].clone();
    let local_address = Address::new(match SocketAddrV6::from_str(&args[2]) {
        Ok(addr) => (addr.ip().clone(), addr.port()),
        Err(_) => {
            eprintln!("Invalid local IPv6 address and port.");
            return;
        }
    });
    let server_address = Address::new(match SocketAddrV6::from_str(&args[3]) {
        Ok(addr) => (addr.ip().clone(), addr.port()),
        Err(_) => {
            eprintln!("Invalid server IPv6 address and port.");
            return;
        }
    });

    let mut interface = Interface::new();

    let mut node = Client::new(server_address, None);
    let mut ctx = Ctx::new(local_address.id()).await;

    let mut terminal_events = key_events().fuse();
    let mut client_events = ctx.event_stream().boxed().fuse();

    let mut ctx = Context::new(local_address, &mut ctx);

    node.init(&mut ctx);

    let mut latest_id = 0;

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
                            Some(ChatResponse::Latest(history, id)) if id > latest_id => {
                                node.command = None;
                                interface.set_history(history);
                                latest_id = id;
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
            _ = sleep(Duration::from_millis(500)).fuse() => {
                // poll the server for the latest history.
                if node.command.is_none() {
                    node.command = Some(ChatCommand::GetLatest(latest_id));
                    node.send_command(&mut ctx);
                }
            }
        };
    }

    interface.close();
}

fn key_events() -> impl Stream<Item = crossterm::event::Event> {
    // get the reader
    EventStream::new().map(|e| e.unwrap())
}
