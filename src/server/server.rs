use chat_application::{
    context::{self, Ctx},
    ChatApp,
};
use ds_libs::{address::Address, HandleMessage, InitializeNode};
use futures::StreamExt;
use simple_server::user::Server;
use std::{env, net::ToSocketAddrs};

use anyhow::{anyhow, Result};

fn parse_address<Node>(s: &str) -> Result<Address<Node>> {
    for ip_port in s.to_socket_addrs()? {
        if let std::net::SocketAddr::V6(ip_port) = ip_port {
            return Ok(Address::new((*ip_port.ip(), ip_port.port())))
        }
    }

    Err(anyhow!(
        "Failed to produce an IPv6:port pair from the provided address: {}",
        s
    ))
}

#[tokio::main]
async fn main() {
    // Get the local address.
    let args: Vec<_> = env::args().collect();

    if args.len() != 2 {
        eprintln!(
            "You must provide only 1 argument: <local IPv6 address and port. Ex: [::1]:8080>"
        );
        return;
    }

    let node_address = match parse_address(&args[1]) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Failed to get the local address: {:?}", e);
            return;
        }
    };

    // Construct the server.
    let mut node = Server::new(ChatApp::new());

    // Construct the context.
    let mut ctx = Ctx::new(node_address.id()).await;
    let mut event_stream = ctx.event_stream().boxed().fuse();
    let mut ctx = ds_libs::Context::new(node_address, &mut ctx);

    // Init the server.
    node.init(&mut ctx);

    while let Some(event) = event_stream.next().await {
        match event {
            // Servers don't handle Responses or ResendTimers.
            context::Event::Response(_) | context::Event::ResendTimer(_) => {}
            context::Event::Request(req) => {
                node.handle_message(&mut ctx, req);
            }
        }
    }
}
