use regex::{Match, Regex};
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

    let re =
        Regex::new(r"(?P<command>[a-z]+)(?P<transaction>\d+)(\[(?P<resource>[a-z])\])?").unwrap();

    let operations: Vec<Operation> = re
        .captures_iter(&contents)
        .map(|operation| {
            let command = &operation["command"];
            let transaction = operation["transaction"].parse::<u32>().unwrap();

            let mut resource = "";

            if let Some(value) = operation.get(3) {
                resource = value.as_str();
            }

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
