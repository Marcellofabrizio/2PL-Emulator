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

impl LockInfo {
    fn add_shared_owner(&mut self, shared_owner: u32) {
        self.shared_owners.push(shared_owner);
    }

    fn add_exclusive_owner(&mut self, exclusive_owner: u32) {
        self.exclusive_owner = Some(exclusive_owner);
    }

    fn remove_all(&mut self, transaction: u32) {
        self.shared_owners.retain(|&t| t != transaction);
        if self.exclusive_owner.is_some_and(|t| t == transaction) {
            self.exclusive_owner = None;
        }
    }
}

#[derive(Debug)]
struct LockTable {
    lock_table: HashMap<String, LockInfo>,
}

impl LockTable {
    fn acquire_shared_lock(&mut self, transaction: u32, resource: &String) -> bool {
        if let Some(info) = self.lock_table.get_mut(resource) {
            if info.exclusive_owner.is_some() {
                return false;
            } else {
                info.add_shared_owner(transaction);
                return true;
            }
        } else {
            self.lock_table.insert(
                resource.clone(),
                LockInfo {
                    shared_owners: vec![transaction],
                    exclusive_owner: None,
                },
            );
            return true;
        }
    }

    fn acquire_exclusive_lock(&mut self, transaction: u32, resource: &String) -> bool {
        if let Some(info) = self.lock_table.get_mut(resource) {
            let needs_upgrade =
                info.shared_owners.len() == 1 && info.shared_owners[0] == transaction;

            if info.exclusive_owner.is_none() && (info.shared_owners.is_empty() || needs_upgrade) {
                info.add_exclusive_owner(transaction);
                return true;
            } else {
                return false;
            }
        } else {
            self.lock_table.insert(
                resource.clone(),
                LockInfo {
                    shared_owners: vec![],
                    exclusive_owner: Some(transaction),
                },
            );
            return true;
        }
    }

    fn remove_locks(&mut self, transaction: u32) {
        for (_, info) in self.lock_table.iter_mut() {
            info.remove_all(transaction);
        }
    }
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

    let mut locks = LockTable {
        lock_table: HashMap::new(),
    };
    let mut delayed_operations: Vec<Operation> = vec![];

    for op in operations {
        println!("Operation {:?}", op);
        // TODO Antes de executar cada operação, precisa ver se alguma das que tão em delay pode
        // finalmente executar
        match op {
            Operation::Read(transaction, ref resource) => {
                if locks.acquire_shared_lock(transaction, &resource) {
                    // Adicionar a operação na história final
                } else {
                    delayed_operations.push(op);
                }
            }
            Operation::Write(transaction, ref resource) => {
                if locks.acquire_exclusive_lock(transaction, &resource) {
                    // Adicionar a operação na história final
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
}
