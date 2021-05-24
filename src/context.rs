use std::{sync::Arc, task::Poll};

use ds_libs::{amo_application, ManageMessageType, ManageTimerType};

use futures::{
    channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
    future::{pending, BoxFuture},
    stream::{self, FuturesUnordered},
    FutureExt, Stream, StreamExt,
};
use serde::{Deserialize, Serialize};
use simple_server::user::{Client, ResendTimer};
use tokio::{
    net::{ToSocketAddrs, UdpSocket},
    runtime::Handle,
    time::sleep,
};

use crate::{ChatApp, ChatCommand, ChatResponse};

pub struct Ctx<'a> {
    socket: Arc<UdpSocket>,
    timer_sink: UnboundedSender<BoxFuture<'a, ResendTimer>>,
    timer_stream: Option<TimerStream<'a, ResendTimer>>,
}

impl<'a> Ctx<'a> {
    pub async fn new<A>(addr: A) -> Ctx<'a>
    where
        A: ToSocketAddrs,
    {
        let socket = match UdpSocket::bind(addr).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to bind the port: {:?}", e);
                std::process::exit(1);
            }
        };

        let (sender, receiver) = unbounded();

        let timer_stream = Some(TimerStream::new(receiver));

        Ctx {
            socket: Arc::new(socket),
            timer_sink: sender,
            timer_stream,
        }
    }

    pub fn event_stream(&mut self) -> impl Stream<Item = Event> + 'a {
        stream::select(
            self.message_stream(),
            self.timer_stream
                .take()
                .unwrap()
                .map(Event::ResendTimer),
        )
    }

    fn message_stream(&self) -> impl Stream<Item = Event> {
        stream::unfold(Arc::clone(&self.socket), |socket| {
            async move {
                let mut buf = [0; 1024];

                // Get the message
                let (size, _) = socket
                    .recv_from(&mut buf)
                    .await
                    .expect("Failed receiving a message");

                // Parse the message
                let event = bincode::deserialize(&buf[..size])
                    .expect("Failed at deserializing the message");

                Some((event, socket))
            }
        })
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Event {
    Request(amo_application::Request<ChatCommand, Client<ChatApp>>),
    Response(amo_application::Response<ChatResponse>),
    ResendTimer(simple_server::user::ResendTimer),
}

impl<'a> ManageMessageType<amo_application::Request<ChatCommand, Client<ChatApp>>> for Ctx<'a> {
    fn add<Node>(
        &mut self,
        dst: ds_libs::address::Address<Node>,
        msg: amo_application::Request<ChatCommand, Client<ChatApp>>,
    ) {
        let msg =
            bincode::serialize(&Event::Request(msg)).expect("Failed to serialize the message");

        let socket = Arc::clone(&self.socket);
        let address = dst.id();
        Handle::current().spawn(async move {
            socket
                .send_to(&msg, address)
                .await
                .expect("Failed to send message");
        });
    }
}

impl<'a> ManageMessageType<amo_application::Response<ChatResponse>> for Ctx<'a> {
    fn add<Node>(
        &mut self,
        dst: ds_libs::address::Address<Node>,
        msg: amo_application::Response<ChatResponse>,
    ) {
        let msg =
            bincode::serialize(&Event::Response(msg)).expect("Failed to serialize the message");

        let socket = Arc::clone(&self.socket);
        let address = dst.id();
        Handle::current().spawn(async move {
            socket
                .send_to(&msg, address)
                .await
                .expect("Failed to send message");
        });
    }
}

impl<'a> ManageTimerType<simple_server::user::ResendTimer> for Ctx<'a> {
    fn add<Node>(
        &mut self,
        _node: ds_libs::address::Address<Node>,
        timer: simple_server::user::ResendTimer,
        length: std::time::Duration,
    ) {
        self.timer_sink
            .start_send(
                async move {
                    sleep(length).await;
                    timer
                }
                .boxed(),
            )
            .expect("Failed to set timer");
    }
}

struct TimerStream<'a, T> {
    timers: FuturesUnordered<BoxFuture<'a, T>>,
    receiver: UnboundedReceiver<BoxFuture<'a, T>>,
}

impl<'a, T> TimerStream<'a, T> {
    fn new(receiver: UnboundedReceiver<BoxFuture<'a, T>>) -> TimerStream<'a, T>
    where
        T: Send + 'a,
    {
        let timers = FuturesUnordered::new();
        timers.push(pending().boxed());

        TimerStream { timers, receiver }
    }
}

impl<'a, T> Stream for TimerStream<'a, T> {
    type Item = T;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        // Check the receiver
        while let Poll::Ready(f) = self.receiver.poll_next_unpin(cx) {
            if let Some(f) = f {
                self.timers.push(f);
            }
        }

        self.timers.poll_next_unpin(cx)
    }
}

#[cfg(test)]
mod tests {
    use std::{net::Ipv6Addr, str::FromStr, time::Duration};

    use ds_libs::{address::Address, amo_application::Response};
    use tokio::time::timeout;

    use super::*;

    #[tokio::test]
    async fn sending_response_to_self() {
        let _address1 = Address::<()>::new((Ipv6Addr::from_str("::1").unwrap(), 8080));
        let mut ctx1 = Ctx::new(("::1", 8080)).await;
        let address2 = Address::<()>::new((Ipv6Addr::from_str("::1").unwrap(), 8081));
        let ctx2 = Ctx::new(("::1", 8081)).await;

        let res = Response {
            result: ChatResponse::PostOk,
            sequence_number: 1,
        };

        ds_libs::ManageMessageType::add(&mut ctx1, address2, res.clone());

        let event = timeout(
            Duration::from_secs(5),
            Box::pin(ctx2.message_stream()).next(),
        )
        .await;
        assert_eq!(Ok(Some(Event::Response(res))), event);
    }

    #[tokio::test]
    async fn setting_timer() {
        let address1 = Address::<()>::new((Ipv6Addr::from_str("::1").unwrap(), 8080));
        let mut ctx1 = Ctx::new(("::1", 0)).await;

        ds_libs::ManageTimerType::add(
            &mut ctx1,
            address1,
            ResendTimer(1),
            Duration::from_millis(200),
        );

        let event = timeout(Duration::from_secs(1), Box::pin(ctx1.event_stream()).next()).await;
        assert_eq!(Ok(Some(Event::ResendTimer(ResendTimer(1)))), event);
    }
}
