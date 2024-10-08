use ratatui::widgets::ListState;
use crossterm::event::KeyCode;
use crossterm::event::Event;
use std::time::Duration;
use crossterm::event;
use ratatui::widgets::List;
use ratatui::Frame;
use std::str::FromStr;
use colored::Colorize;

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::collections::HashMap;
use regex::Regex;

mod tui;

struct ParseLineError;

#[derive(Debug, Clone)]
struct Line {
    uuid: String,
    time: String,
    text: String
}

#[derive(PartialEq, Default, Debug, Clone)]
enum RunningState {
    #[default] Run,
    Done
}

#[derive(Default, Debug, Clone)]
struct LogSet {
    uuid: String,
    time: String,
    lines: Vec<Line>
}

impl LogSet {
    fn from_lines(lines: Vec<Line>) -> Self {
        Self {
            uuid: "abc".to_string(),
            time: "abc".to_string(),
            lines 
        }
    }
}

#[derive(Default, Debug, Clone)]
struct Model {
    running_state: RunningState,
    log_sets: Vec<LogSet>,
    current_item: ListState,
}

enum ParseError {
    ParseLineError
}

type LineResult = Result<Line, ParseError>;

impl FromStr for Line {
    type Err = ParseError;
    fn from_str(s: &str) -> LineResult {
        let splits: Vec<&str> = s.split("[").collect();
        let time: Vec<&str> = splits[1].split(" ").collect();
        let re: Regex = Regex::new(r"\[([0-9a-f]{32})\]").unwrap();
        let (_, [uuid]) = re.captures(s).ok_or(ParseError::ParseLineError)?.extract();
        Ok(Line {
            uuid: uuid.to_string(),
            time: time[0].to_string(),
            text: s.to_string()
        })
    }
}

fn lines() -> Result<Vec<LogSet>, std::io::Error> {
    let lines = read_lines("rails.log")?;

    {
        let mut logs: HashMap<String, Vec<Line>> = HashMap::new();

        lines
            .map_while(Result::ok)
            .filter_map(|line| Line::from_str(&line).ok())
            .for_each(|line|
                {
                    logs.entry(line.uuid.clone()).and_modify(|lines|
                        lines.push(line.clone())
                    ).or_insert_with(|| vec![line]);
                }
            );

        let mut values: Vec<&Vec<Line>> = logs
            .values()
            .collect();

        values.sort_by_key(|log| log[0].time.clone());
        let log_sets = values.into_iter()
            .map(|lines|
                LogSet::from_lines(lines.to_vec())
            )
            .collect();
        Ok(log_sets)
        // values.into_iter().for_each(|lines| {
        //     let uuid = &lines[0].uuid;
        //     let a = u8::from_str_radix(&uuid[0..=1], 16).unwrap().clamp(127, 255);
        //     let b = u8::from_str_radix(&uuid[2..=3], 16).unwrap().clamp(127, 255);
        //     let c = u8::from_str_radix(&uuid[4..=5], 16).unwrap().clamp(127, 255);
        //     for line in lines {
        //         println!("{}", line.text.truecolor(a, b, c));
        //     }
        // });
    }
}

#[derive(PartialEq)]
enum Message {
    NextSet,
    PrevSet,
    Quit,
}

fn handle_key(key: event::KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('j') => Some(Message::NextSet),
        KeyCode::Char('k') => Some(Message::PrevSet),
        KeyCode::Char('q') => Some(Message::Quit),
        _ => None,
    }
}

fn handle_event(_: &Model) -> color_eyre::Result<Option<Message>> {
    if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                return Ok(handle_key(key));
            }
        }
    }
    Ok(None)
}

fn update(model: &mut Model, msg: Message) -> Option<Message> {
    match msg {
        Message::Quit => {
            // You can handle cleanup and exit here
            model.running_state = RunningState::Done;
        }
        Message::NextSet => {
            model.current_item.select_next();
        }
        Message::PrevSet => {
            model.current_item.select_previous();
        }
    };
    None
}

fn main() -> color_eyre::Result<()> {
    tui::install_panic_hook();
    let mut terminal = tui::init_terminal()?;
    let mut model = Model::default();
    model.log_sets = lines().unwrap();
    while model.running_state != RunningState::Done {
        // Render the current view
        terminal.draw(|f| view(&mut model, f))?;

        // Handle events and map to a Message
        let mut current_msg = handle_event(&model)?;

        // Process updates as long as they return a non-None message
        while current_msg.is_some() {
            current_msg = update(&mut model, current_msg.unwrap());
        }
        // model.running_state = RunningState::Done;
    }

    tui::restore_terminal()?;
    Ok(())
}

fn view(model: &mut Model, frame: &mut Frame) {
    let items: Vec<String> = model.log_sets.iter().map(|ls| ls.lines[0].text.clone()).collect();
    let list = List::new(items)
        // .block(Block::bordered().title("List"))
        // .style(Style::default().fg(Color::White))
        // .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol(">>");
        // .repeat_highlight_symbol(true)
        // .direction(ListDirection::BottomToTop);

    // frame.render_widget(list, frame.area());
    frame.render_stateful_widget(list, frame.area(), &mut model.current_item);
}

// The output is wrapped in a Result to allow matching on errors.
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
