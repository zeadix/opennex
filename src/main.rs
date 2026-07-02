mod terminal;
mod state;
mod config;
mod plugin;
mod template;

use iced::widget::{button, column, container, horizontal_rule, row, scrollable, text, text_input};
use iced::{Element, Length};

fn main() -> iced::Result {
    env_logger::init();
    iced::run("OpenZoo - AI Terminal Manager", update, view)
}

struct State {
    tabs: Vec<Tab>,
    active_tab: usize,
    tab_counter: u32,
}

struct Tab {
    id: u32,
    title: String,
    history: String,
    input: String,
}

#[derive(Debug, Clone)]
enum Message {
    NewTab,
    CloseTab(usize),
    SelectTab(usize),
    InputChanged(String),
    Execute,
    ClearScreen,
}

impl Default for State {
    fn default() -> Self {
        let mut state = State {
            tabs: Vec::new(),
            active_tab: 0,
            tab_counter: 0,
        };
        state.add_tab();
        state
    }
}

impl State {
    fn add_tab(&mut self) {
        self.tab_counter += 1;
        self.tabs.push(Tab {
            id: self.tab_counter,
            title: format!("Terminal {}", self.tab_counter),
            history: "Welcome to OpenZoo Terminal Manager\nType 'help' for available commands\n\n".to_string(),
            input: String::new(),
        });
        self.active_tab = self.tabs.len() - 1;
    }

    fn execute(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            let cmd = tab.input.trim().to_string();
            if cmd.is_empty() {
                return;
            }
            let output = match cmd.as_str() {
                "help" => "Available commands:\n  help  - Show help\n  clear - Clear screen\n  ls    - List files\n  pwd   - Print working directory".to_string(),
                "clear" => {
                    tab.history.clear();
                    tab.input.clear();
                    return;
                }
                "ls" => "Files:\n  src/\n  tests/\n  Cargo.toml".to_string(),
                "pwd" => std::env::current_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
                other if other.starts_with("echo ") => other[5..].to_string(),
                other => format!("Executed: {}", other),
            };
            tab.history.push_str(&format!("$ {}\n{}\n\n", cmd, output));
            tab.input.clear();
        }
    }
}

fn update(state: &mut State, message: Message) {
    match message {
        Message::NewTab => state.add_tab(),
        Message::CloseTab(index) => {
            if state.tabs.len() > 1 {
                state.tabs.remove(index);
                if state.active_tab >= state.tabs.len() {
                    state.active_tab = state.tabs.len() - 1;
                }
            }
        }
        Message::SelectTab(index) => state.active_tab = index,
        Message::InputChanged(value) => {
            if let Some(tab) = state.tabs.get_mut(state.active_tab) {
                tab.input = value;
            }
        }
        Message::Execute => state.execute(),
        Message::ClearScreen => {
            if let Some(tab) = state.tabs.get_mut(state.active_tab) {
                tab.history.clear();
            }
        }
    }
}

fn view(state: &State) -> Element<'_, Message> {
    let tab_row: Element<Message> = state.tabs.iter().enumerate()
        .fold(row![].spacing(4), |r, (i, tab)| {
            let is_active = i == state.active_tab;
            let label = text(&tab.title);
            let tab_btn = if is_active {
                button(label).style(button::primary)
            } else {
                button(label)
            };
            let close_btn = button("x").on_press(Message::CloseTab(i));
            r.push(tab_btn.on_press(Message::SelectTab(i))).push(close_btn)
        })
        .into();

    let new_tab_btn = button("+ New").on_press(Message::NewTab);
    let clear_btn = button("Clear").on_press(Message::ClearScreen);

    let header = column![
        horizontal_rule(1),
        row![tab_row, new_tab_btn].spacing(4),
        horizontal_rule(1),
    ]
    .spacing(4);

    let content: Element<Message> = if let Some(tab) = state.tabs.get(state.active_tab) {
        let output = scrollable(
            text(&tab.history)
                .font(iced::Font::MONOSPACE)
                .width(Length::Fill)
        )
        .height(Length::Fill);

        let input_field = text_input("$ Enter command...", &tab.input)
            .on_submit(Message::Execute)
            .on_input(Message::InputChanged)
            .font(iced::Font::MONOSPACE)
            .width(Length::Fill);

        let execute_btn = button("Execute").on_press(Message::Execute);

        column![
            output,
            horizontal_rule(1),
            row![input_field, execute_btn].spacing(4),
        ]
        .spacing(4)
        .into()
    } else {
        column![text("No tabs")].into()
    };

    let footer = column![
        horizontal_rule(1),
        clear_btn,
    ]
    .spacing(2);

    container(
        column![header, content, footer].spacing(4)
    )
    .padding(8)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
