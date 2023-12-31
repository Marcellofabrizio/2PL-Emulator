use crate::{lock_table::LockTable, Operation};

pub struct Scheduler {
    pub locks: LockTable,
    pub delayed_operations: Vec<Operation>,
    pub final_history: Vec<Operation>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            locks: LockTable::new(),
            delayed_operations: Vec::new(),
            final_history: Vec::new(),
        }
    }

    pub fn process_operation(&mut self, op: Operation) {
        match op {
            Operation::Read(transaction, ref resource) => {
                if self.locks.acquire_shared_lock(&transaction, &resource) {
                    self.final_history
                        .push(Operation::LockShared(transaction, resource.to_owned()));
                    self.final_history.push(op);
                } else {
                    self.delayed_operations.push(op);
                }
            }
            Operation::Write(transaction, ref resource) => {
                if self.locks.acquire_exclusive_lock(&transaction, &resource) {
                    // Adicionar a operação na história final
                    self.final_history
                        .push(Operation::LockExclusive(transaction, resource.to_owned()));
                    self.final_history.push(op);
                } else {
                    self.delayed_operations.push(op);
                }
            }
            Operation::Commit(transaction) => {
                let mut unlocks = self.locks.remove_locks(&transaction);
                self.retry_delayed_operations();
                self.final_history.push(Operation::Commit(transaction));
                self.final_history.append(&mut unlocks);
            }
            Operation::Abort(transaction) => {
                // TODO: Remover as operações dessa transação da história final
                self.locks.remove_locks(&transaction);
            }
            _ => return,
        }
    }

    pub fn show_state(&self) {
        self.locks.show_state();
        if self.delayed_operations.len() > 0 {
            println!("Operações em delay: {:?}", self.delayed_operations);
        }
        println!();
    }

    pub fn show_final_history(&self) {
        println!("História final:");
        for op in &self.final_history {
            println!("\t{:?},", op);
        }
    }

    fn retry_delayed_operations(&mut self) {
        let delayed = self.delayed_operations.clone();
        self.delayed_operations.clear();

        for op in delayed {
            println!("Tentando reprocessar {:?}.", op);
            self.process_operation(op);
        }
    }
}
