use regex::Regex;
use std::fs;
mod lock_table;
mod scheduler;

#[derive(Debug)]
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
            (?<command>[a-z]+) # O comando: read, write, commit ou abort
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

        scheduler.process_operation(op);

        // TODO Antes de executar cada operação, precisa ver se alguma das que tão em delay pode
        // finalmente executar
        /*
            TODO: Mover para uma funcão?
        */
        let delayed_opp = scheduler.delayed_operations.pop();
        match delayed_opp {
            Some(Operation::Read(ref transaction, ref resource)) => {
                if scheduler.can_process_delayed(resource) {
                    println!("Reprocessing {:?}", &delayed_opp);
                    scheduler
                        .process_operation(Operation::Read(*transaction, resource.to_string()));
                } else {
                    println!("Still delayed");
                    scheduler
                        .delayed_operations
                        .push(Operation::Read(*transaction, resource.to_string()));
                }
            }
            Some(Operation::Write(ref transaction, ref resource)) => {
                if scheduler.can_process_delayed(resource) {
                    println!("Reprocessing {:?}", &delayed_opp);
                    scheduler
                        .process_operation(Operation::Write(*transaction, resource.to_string()));
                } else {
                    println!("Still delayed");
                    scheduler
                        .delayed_operations
                        .push(Operation::Write(*transaction, resource.to_string()));
                }
            }
            _ => (),
        };

        println!("{:?}", scheduler.locks);
        println!("Operações em espera: {:?}\n", scheduler.delayed_operations);
    }

    println!("\n");
    println!("História final: {:?}\n", scheduler.final_history);
}
