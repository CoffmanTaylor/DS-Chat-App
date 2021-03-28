use chat_application::{
    context::{self, Ctx},
    ChatApp,
};
use ds_libs::{address::Address, HandleMessage, InitializeNode};
use futures::StreamExt;
use simple_server::user::Server;
use std::{net::Ipv6Addr, str::FromStr};

#[tokio::main]
async fn main() {
    // Construct the server.
    let node_address = Address::new((Ipv6Addr::from_str("::1").unwrap(), 8081));
    let mut node = Server::new(ChatApp::new());

    // Construct the context.
    let mut ctx = Ctx::new(("::1", 8081)).await;
    let mut event_stream = ctx.event_stream().boxed().fuse();
    let mut ctx = ds_libs::Context::new(node_address, &mut ctx);

    // Init the server.
    node.init(&mut ctx);

    while let Some(event) = event_stream.next().await {
        match event {
            // Servers don't handle Responses or ResendTimers.
            context::Event::Response(_) | context::Event::ResendTimer(_) => {}
            context::Event::Request(req) => {
                println!("Handling req: {:?}", req);
                node.handle_message(&mut ctx, req);
            }
        }
    }
}
