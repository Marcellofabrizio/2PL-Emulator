use regex::Regex;
use std::{collections::HashMap, fs};

#[derive(Debug)]
enum Operation {
    LockShared(u32, String),
    LockExclusive(u32, String),
    Read(u32, String),
    Write(u32, String),
    UnlockShared(u32, String),
    UnlockExclusive(u32, String),
    Commit(u32),
    Abort(u32),
    Unknown,
}

#[derive(PartialEq, Eq, Hash, Debug)]
struct LockInfo {
    shared_owners: Vec<u32>,
    exclusive_owner: Option<u32>,
}

fn main() {
    let contents = fs::read_to_string("history.txt".to_string()).expect("File not found");

    let re =
        Regex::new(r"(?x)
            (?<command>[a-z]+) # O comando: read, write, commit ou abort
            (?<transaction>\d+) # O número da transação
            ([(\[](?<resource>\S+)[)\]])? # Qualquer texto sem espaço entre parenteses ou colchetes
        ").unwrap();

    let operations: Vec<Operation> = contents
        .split("-")
        .map(|operation| {
            let captures = re.captures(operation).unwrap();

            let command = &captures["command"];
            let transaction = captures["transaction"].parse::<u32>().unwrap();
            let resource = captures.name("resource").map_or("", |c| c.as_str());

            match command {
                "r" => Operation::Read(transaction, resource.to_owned()),
                "w" => Operation::Write(transaction, resource.to_owned()),
                "c" => Operation::Commit(transaction),
                "a" => Operation::Abort(transaction),
                _ => Operation::Unknown,
            }
        })
        .collect();

    let mut lock_table: HashMap<String, LockInfo> = HashMap::new();
    let mut delayed_operations: Vec<Operation>;

    for op in operations {
        match op {
            Operation::Read(transaction, resource) => match lock_table.get(&resource) {
                Some(&info) => {
                    if info.exclusive_owner.is_some() {
                        delayed_operations.push(op);
                    } else {
                        info.shared_owners.push(transaction);
                    }
                }
                None => {
                    lock_table.insert(
                        resource,
                        LockInfo {
                            shared_owners: vec![transaction],
                            exclusive_owner: None,
                        },
                    );
                }
            }
            Operation::Write(transaction, resource) => {
                println!("(Write)Transaction = {transaction}\tResource = {resource}");
            }
            Operation::Commit(transaction) => {
                println!("(Commit)Transaction = {transaction}");
            }
            Operation::Abort(transaction) => {
                println!("(Commit)Transaction = {transaction}");
            }
            _ => return,
        }
    }
}
