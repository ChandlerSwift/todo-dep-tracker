use serde::{Deserialize, Serialize};
use std::{fmt, io};
use std::fs::OpenOptions;
use std::io::{prelude::*, ErrorKind};
use std::{env, fs::File, path::Path};

#[derive(Serialize, Deserialize, Debug, Default)]
struct TodoItem {
    title: String,
    details: String,
    completed: bool,
    children: Vec<TodoItem>,
}

impl TodoItem {
    fn fmt_with_indentation(&self, f: &mut fmt::Formatter<'_>, recursion_level: usize) -> fmt::Result {
        for _ in 0..recursion_level {
            f.write_str("    ")?;
        }
        if self.completed {
            f.write_str("[x] ")?;
        } else {
            f.write_str("[ ] ")?;
        }
        f.write_str(&self.title)?;
        f.write_str("\n")?;
        for child in &self.children {
            if recursion_level < 5 { // TODO: configurable
                child.fmt_with_indentation(f, recursion_level+1)?;
            }
        }
        Ok(())
    }

    fn complete(&mut self) {
        self.completed = true;
        for child in &mut self.children {
            child.complete();
        }
    }
}

impl fmt::Display for TodoItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_indentation(f, 0)
    }
}


fn main() {
    let path = match env::args().nth(1) {
        Some(path) => path,
        // https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html
        None => {
            env::var("XDG_DATA_HOME").unwrap_or(env::var("HOME").unwrap() + "/.local/share") + "/todo-dep-tracker.json"
        }
    };
    let path = Path::new(&path);
    println!("DEBUG: using file {}", path.display());

    let mut todoitems: Vec<TodoItem>; // TODO: Define TodoList type that wraps this?

    let mut file = match OpenOptions::new().read(true).write(true).open(path) {
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {
                println!("File not found; creating empty list.");
                todoitems = Vec::new();
                File::create(path).expect("Failed to create file")
            }
            _ => panic!("couldn't open {}: {:?}", path.display(), e),
        },
        Ok(mut file) => {
            let mut s = String::new();
            match file.read_to_string(&mut s) {
                Err(why) => panic!("couldn't read {}: {}", path.display(), why),
                Ok(len) => {
                    println!("DEBUG: Read {} bytes", len);
                    todoitems = serde_json::from_str(&s).expect("Could not decode save file");
                }
            };
            file
        }
    };

    let stdin = io::stdin();
    loop {
        println!("To run any command, type the command symbol, followed by the target."); // TODO: help page
        println!("+<task>: Create new top-level task");
        println!("d<n>: Delete task <n> and all child tasks");
        println!("c<n>: Mark task <n> and all child tasks as complete"); // TODO; ranges?
        println!("i<n>: Mark task <n> and all parent tasks as incomplete"); // TODO: this is going to require some significant refactoring if we want to reference back up the tree.
        println!("q: save and quit");
        println!("Qn: save without quitting");
        for (_, todoitem) in todoitems.iter().enumerate() {
            print!("{}", todoitem);
        }
        let mut buf = String::new();
        stdin.read_line(&mut buf).unwrap();
        match buf.chars().nth(0) {
            None => println!("No character provided."),
            Some('+') => {
                let ti = TodoItem {
                    title: buf[1..buf.len()].trim().to_string(),
                    details: String::new(),
                    completed: false,
                    children: Vec::new(),
                };
                todoitems.push(ti)
            },
            Some('d') => {
                let index = buf[1..buf.len()].trim().parse();
                match index {
                    Ok(index) => {
                        if index < todoitems.len() {
                            todoitems.remove(index);
                        } else {
                            println!("Out of range")
                        }
                    },
                    Err(e) => println!("Could not parse int. Not removed. {}", e),
                }
            },
            Some('c') => {
                let index: Result<usize, _> = buf[1..buf.len()].trim().parse();
                match index {
                    Ok(index) => {
                        if index < todoitems.len() {
                            todoitems[index].complete();
                        } else {
                            println!("Out of range")
                        }
                    },
                    Err(e) => println!("Could not parse int. Not marked as complete. {}", e),
                }
            },
            Some('q') => break,
            Some(c) => println!("Unknown command {}", c),
        };
    }

    file.rewind().unwrap();
    file.set_len(0).unwrap(); // TODO: instead of these; consider reopening with write/truncate set
    writeln!(file, "{}", serde_json::to_string_pretty(&todoitems).unwrap()).unwrap();
}
