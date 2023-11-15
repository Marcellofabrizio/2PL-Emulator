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
                    // Adicionar a operação na história final
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
                self.locks.remove_locks(transaction);
            }
            Operation::Abort(transaction) => {
                // Ainda não sei se deveria ter um comportamento diferente aqui
                self.locks.remove_locks(transaction);
            }
            _ => return,
        }
    }

    pub fn can_process_delayed(&mut self, resource: &String) -> bool {
        let mut is_still_delayed = false;

        if let Some(info) = self.locks.lock_table.get_mut(resource) {
            is_still_delayed = info.exclusive_owner.is_none() && info.shared_owners.is_empty();
        }

        return is_still_delayed;
    }
}
