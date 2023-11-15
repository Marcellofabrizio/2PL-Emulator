use regex::Regex;
use std::fs;
mod lock_table;

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

    let mut locks = lock_table::LockTable::new();
    let mut delayed_operations: Vec<Operation> = vec![];

    let mut final_history: Vec<Operation> = vec![];

    for op in operations {
        println!("Operation {:?}", op);
        // TODO Antes de executar cada operação, precisa ver se alguma das que tão em delay pode
        // finalmente executar
        match op {
            Operation::Read(transaction, ref resource) => {
                if locks.acquire_shared_lock(transaction, &resource) {
                    // Adicionar a operação na história final
                    final_history.push(Operation::LockShared(transaction, resource.to_owned()));
                    final_history.push(op);
                } else {
                    delayed_operations.push(op);
                }
            }
            Operation::Write(transaction, ref resource) => {
                if locks.acquire_exclusive_lock(transaction, &resource) {
                    // Adicionar a operação na história final
                    final_history.push(Operation::LockExclusive(transaction, resource.to_owned()));
                    final_history.push(op);
                } else {
                    delayed_operations.push(op);
                }
            }
            Operation::Commit(transaction) => {
                locks.remove_locks(transaction);
            }
            Operation::Abort(transaction) => {
                // Ainda não sei se deveria ter um comportamento diferente aqui
                locks.remove_locks(transaction);
            }
            _ => return,
        }
        println!("{:?}", locks);
        println!("Operações em espera: {:?}\n", delayed_operations);
    }

    println!("\n");
    println!("História final: {:?}\n", final_history);
}
