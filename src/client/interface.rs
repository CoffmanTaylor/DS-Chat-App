use chat_application::Message;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::{
    io::{self, Stdout},
    time::SystemTime,
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

pub struct Interface {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    history: Vec<Message>,
    input: String,
}

impl Interface {
    pub fn new() -> Interface {
        // Construct the terminal.
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        enable_raw_mode().unwrap();

        let mut out = Interface {
            terminal: Terminal::new(backend).unwrap(),
            input: String::new(),
            history: vec![Message {
                sender: "Taylor".to_string(),
                text: "Hello World!".to_string(),
                sent_time: SystemTime::now().into(),
            }],
        };

        out.terminal.clear().unwrap();
        out.render();

        out
    }

    pub fn set_history(&mut self, history: Vec<Message>) {
        self.history = history;
        self.render();
    }

    pub fn clear_input(&mut self) -> String {
        let out = std::mem::take(&mut self.input);
        self.render();
        out
    }

    pub fn push_input(&mut self, c: char) {
        self.input.push(c);
        self.render();
    }

    pub fn pop_input(&mut self) {
        self.input.pop();
        self.render();
    }

    pub fn close(&mut self) {
        self.terminal.clear().unwrap();
        disable_raw_mode().unwrap();
    }

    pub fn render(&mut self) {
        let input_text = self.input.clone();
        let history = self.history.clone();
        self.terminal
            .draw(|f| {
                // Split the screen in two.
                let sections = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
                    .split(f.size());

                let chat_history =
                    List::new(history.into_iter().map(Message::into).collect::<Vec<_>>())
                        .block(Block::default().title("Chat History").borders(Borders::ALL));
                f.render_widget(chat_history, sections[0]);

                let input =
                    Paragraph::new(input_text + "_").block(Block::default().borders(Borders::ALL));
                f.render_widget(input, sections[1]);
            })
            .unwrap();
    }
}
