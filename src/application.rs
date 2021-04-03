use std::{collections::VecDeque, time::SystemTime};

use chrono::{DateTime, Local, Utc};
use ds_libs::Application;
use serde::{Deserialize, Serialize};
use tui::widgets::ListItem;

pub mod context;

/// The maximum number of chat messages to keep in the history.
pub const MAX_CHAT_MESSAGES: usize = 10;
pub const MAX_MESSAGE_SIZE: usize = 100;

/// The backend data for a basic chat app.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChatApp {
    pub(crate) messages: VecDeque<Message>,
    update_id: usize,
}

/// One message.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Message {
    pub text: String,
    pub sent_time: DateTime<Utc>,
    pub sender: String,
}

impl<'a> Into<ListItem<'a>> for Message {
    fn into(self) -> ListItem<'a> {
        ListItem::new(format!(
            "{} - {}: {}",
            self.sent_time.with_timezone(&Local).format("%I:%M%P"),
            self.sender,
            self.text
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ChatCommand {
    /// Post the given message in the chat.
    Post(Message),
    /// Get the history of the chat. Will only return up to [MAX_CHAT_MESSAGES]. If the
    /// the id is the same as the servers, there are no new messages and [ChatResponse::NoUpdate]
    /// will be returned.
    GetLatest(usize),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ChatResponse {
    /// The post was successful.
    PostOk,
    /// The current history of the chat.
    Latest(Vec<Message>, usize),
    /// The history matches what you already have.
    NoUpdate,
}

impl ChatApp {
    /// Construct an empty chat.
    pub fn new() -> ChatApp {
        ChatApp {
            messages: VecDeque::with_capacity(MAX_CHAT_MESSAGES),
            update_id: 0,
        }
    }
}

impl Default for ChatApp {
    fn default() -> Self {
        ChatApp::new()
    }
}

impl Message {
    /// Construct a new message with the send_time set to the current time.
    pub fn new(sender: String, text: String) -> Message {
        Message {
            sender,
            text,
            sent_time: SystemTime::now().into(),
        }
    }
}

impl Application for ChatApp {
    type Command = ChatCommand;

    type Res = ChatResponse;

    fn process(&mut self, request: Self::Command) -> Self::Res {
        match request {
            ChatCommand::Post(post) => {
                // Check if we have too many chat messages
                if self.messages.len() >= MAX_CHAT_MESSAGES {
                    self.messages.pop_front();
                }

                self.messages.push_back(post);
                self.update_id += 1;

                ChatResponse::PostOk
            }
            ChatCommand::GetLatest(id) => {
                if id == self.update_id {
                    ChatResponse::NoUpdate
                } else {
                    ChatResponse::Latest(self.messages.clone().into(), self.update_id)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_message() {
        let mut chat = ChatApp::new();

        let message = Message::new("sender".to_string(), "test".to_string());
        assert_eq!(
            ChatResponse::PostOk,
            chat.process(ChatCommand::Post(message.clone()))
        );

        if let ChatResponse::Latest(log, _) = chat.process(ChatCommand::GetLatest(0)) {
            assert_eq!(1, log.len());
            assert_eq!(message, log[0]);
        } else {
            panic!("Failed to GetLatest");
        }
    }

    #[test]
    fn single_message_with_to_gets() {
        let mut chat = ChatApp::new();

        let message = Message::new("sender".to_string(), "test".to_string());
        assert_eq!(
            ChatResponse::PostOk,
            chat.process(ChatCommand::Post(message.clone()))
        );

        if let ChatResponse::Latest(log, _) = chat.process(ChatCommand::GetLatest(0)) {
            assert_eq!(1, log.len());
            assert_eq!(message, log[0]);
        } else {
            panic!("Failed to GetLatest");
        }

        assert_eq!(
            ChatResponse::NoUpdate,
            chat.process(ChatCommand::GetLatest(1))
        );
    }

    #[test]
    fn two_messages() {
        let mut chat = ChatApp::new();

        let message1 = Message::new("sender1".to_string(), "test1".to_string());
        let message2 = Message::new("sender2".to_string(), "test2".to_string());

        assert_eq!(
            ChatResponse::PostOk,
            chat.process(ChatCommand::Post(message1.clone()))
        );
        assert_eq!(
            ChatResponse::PostOk,
            chat.process(ChatCommand::Post(message2.clone()))
        );

        if let ChatResponse::Latest(log, _) = chat.process(ChatCommand::GetLatest(0)) {
            assert_eq!(2, log.len());
            assert_eq!(message1, log[0]);
            assert_eq!(message2, log[1]);
        } else {
            panic!("Failed to GetLatest");
        }
    }

    #[test]
    fn over_max_messages() {
        let mut chat = ChatApp::new();

        const EXTRA_MESSAGES: usize = 10;

        // generate all of the messages.
        let messages: Vec<_> = (0..(MAX_CHAT_MESSAGES + EXTRA_MESSAGES))
            .map(|x| Message::new(format!("sender {}", x), format!("message {}", x)))
            .collect();

        // Process every message.
        for msg in messages.iter() {
            assert_eq!(
                ChatResponse::PostOk,
                chat.process(ChatCommand::Post(msg.clone()))
            );
        }

        if let ChatResponse::Latest(log, _) = chat.process(ChatCommand::GetLatest(0)) {
            assert_eq!(MAX_CHAT_MESSAGES, log.len());

            assert_eq!(messages[EXTRA_MESSAGES..], log);
        } else {
            panic!("Failed to GetLatest");
        }
    }
}
