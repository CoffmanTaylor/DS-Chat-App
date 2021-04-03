# Chat App
This is a very basic chat system meant to demonstrate the functionality of different distributed systems. Version [v0.1.0](https://github.com/CoffmanTaylor/DS-Chat-App/releases/tag/v0.1.0) is implemented on top of a basic single server many client system.

# Design
There are several connected modules that make up the Chat App. There is [ds-libs](https://github.com/CoffmanTaylor/DS-libs) that defines basic abstractions about a distributed system. There is [model-checking](https://github.com/CoffmanTaylor/model-checking), which is a general purpose model checking framework that is used for testing distributed systems. And v0.1.x uses [simple-server](https://github.com/CoffmanTaylor/DS-Simple-Server) to abstract away the communications between the clients and the server. 

This is all brought together in the binaries: `chat-server` and `chat-client`. Which run the host the server and clients defined in `simple-server` for use in the real world. The hosts are asynchronous programs that respond to user input and network messages and drive their node in the system. Responding to messages or signalling to the UI that a update is required. 

The app runs on a asynchronous distributed systems where messages can be dropped, delayed, and duplicated arbitrarily. The users sends sends a post request to the server, and the server responds with a success message. `ds-libs` defines a generic application, that any distributed system can run, that can only respond to client commands. Therefore, the only way for clients to see if new messages have been posted is to periodically poll the server for the latest messages.

# Installation
You will need Cargo and Rust, tested against Rust 1.51.

For the clients, just run:
```
$ cargo run --bin chat-client <your name> <local address and port, must be IPv6> <server address and port, must be IPv6>
```

for the server, just run:
```
$ cargo run --bin chat-server <local address and port, must be IPv6>
```