use regex::Regex;
use std::fs;

#[derive(Debug)]
enum Operation {
    LockShared(String, u32, String),
    LockExclusive(String, u32, String),
    Read(String, u32, String),
    Write(String, u32, String),
    UnlockShared(String, u32, String),
    UnlockExclusive(String, u32, String),
    Commit(String, u32),
    Unknown,
}

fn main() {
    let contents = fs::read_to_string("history.txt".to_string()).expect("File not found");

    let re = Regex::new(r"(?P<command>[a-z]+)(?P<transaction>\d+)").unwrap();

    let operations: Vec<Operation> = contents
        .split("-")
        .map(|operation| {
            let parts: Vec<&str> = operation.splitn(3, |o| o == '[' || o == ']').collect();

            let command_capture = re.captures(parts[0]).unwrap();
            let command = &command_capture["command"];
            let transaction = command_capture["transaction"].parse::<u32>().unwrap();

            let resource = match parts.get(1) {
                Some(value) => value.to_owned(),
                None => "",
            };

            match command {
                "ls" => Operation::LockShared(command.to_owned(), transaction, resource.to_owned()),
                "lx" => {
                    Operation::LockExclusive(command.to_owned(), transaction, resource.to_owned())
                }
                "r" => Operation::Read(command.to_owned(), transaction, resource.to_owned()),
                "w" => Operation::Write(command.to_owned(), transaction, resource.to_owned()),
                "us" => {
                    Operation::UnlockShared(command.to_owned(), transaction, resource.to_owned())
                }
                "ux" => {
                    Operation::UnlockExclusive(command.to_owned(), transaction, resource.to_owned())
                }
                "c" => Operation::Commit(command.to_owned(), transaction),
                _ => Operation::Unknown,
            }
        })
        .collect();

    println!("{:?}", operations);
}
