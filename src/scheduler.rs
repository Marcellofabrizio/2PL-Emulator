use std::collections::HashMap;

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
}
