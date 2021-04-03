use chat_application::{
    context::{self, Ctx},
    ChatApp,
};
use ds_libs::{address::Address, HandleMessage, InitializeNode};
use futures::StreamExt;
use simple_server::user::Server;
use std::{env, net::SocketAddrV6, str::FromStr};

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

    let node_address = Address::new(match SocketAddrV6::from_str(&args[1]) {
        Ok(addr) => (addr.ip().clone(), addr.port()),
        Err(_) => {
            eprintln!("Invalid local IPv6 address and port.");
            return;
        }
    });

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
