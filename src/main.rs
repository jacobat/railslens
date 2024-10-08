use std::str::FromStr;
use colored::Colorize;

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::collections::HashMap;

struct ParseLineError;

#[derive(Debug, Clone)]
struct Line {
    uuid: String,
    time: String,
    text: String
}

impl FromStr for Line {
    type Err = ParseLineError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let splits: Vec<&str> = s.split("[").collect();
        let time: Vec<&str> = splits[1].split(" ").collect();

        Ok(Line {
            uuid: splits[2][0..32].to_string(),
            time: time[0].to_string(),
            text: s.to_string()
        })
    }
}

fn main() {
    if let Ok(lines) = read_lines("rails.log") {
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
        values.into_iter().for_each(|lines| {
            let uuid = &lines[0].uuid;
            let a = u8::from_str_radix(&uuid[0..=1], 16).unwrap().clamp(127, 255);
            let b = u8::from_str_radix(&uuid[2..=3], 16).unwrap().clamp(127, 255);
            let c = u8::from_str_radix(&uuid[4..=5], 16).unwrap().clamp(127, 255);
            for line in lines {
                println!("{}", line.text.truecolor(a, b, c));
            }
        });
    }
}

// The output is wrapped in a Result to allow matching on errors.
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
