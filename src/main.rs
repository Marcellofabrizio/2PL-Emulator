use regex::Regex;
use std::fs;
mod lock_table;
mod scheduler;

#[derive(Debug, Clone)]
pub enum Operation {
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

fn main() {
    let contents = fs::read_to_string("history.txt".to_string()).expect("File not found");

    let re = Regex::new(
        r"(?x)
            (?<command>[rwca]+) # O comando: read, write, commit ou abort
            (?<transaction>\d+) # O número da transação
            ([(\[](?<resource>\S+)[)\]])? # Qualquer texto sem espaço entre parenteses ou colchetes
        ",
    )
    .unwrap();

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

    let mut scheduler = scheduler::Scheduler::new();

    for op in operations {
        println!("Operation {:?}", op);

        scheduler.retry_delayed_operations();
        scheduler.process_operation(op);

        println!("{:?}", scheduler.locks);
        println!("Operações em espera: {:?}\n", scheduler.delayed_operations);
    }

    println!("\n");
    println!("História final: {:?}\n", scheduler.final_history);
}
