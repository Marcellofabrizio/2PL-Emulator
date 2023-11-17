use crate::Operation;
use std::collections::HashMap;

#[derive(Debug)]
pub struct LockTable {
    pub lock_table: HashMap<String, LockInfo>,
}

impl LockTable {
    pub fn new() -> Self {
        Self {
            lock_table: HashMap::new(),
        }
    }

    pub fn acquire_shared_lock(&mut self, transaction: &u32, resource: &String) -> bool {
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
                    shared_owners: vec![*transaction],
                    exclusive_owner: None,
                },
            );
            return true;
        }
    }

    pub fn acquire_exclusive_lock(&mut self, transaction: &u32, resource: &String) -> bool {
        if let Some(info) = self.lock_table.get_mut(resource) {
            let needs_upgrade =
                info.shared_owners.len() == 1 && info.shared_owners[0] == *transaction;

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
                    exclusive_owner: Some(*transaction),
                },
            );
            return true;
        }
    }

    pub fn remove_locks(&mut self, transaction: &u32) -> Vec<Operation> {
        let mut resources_to_unlock = vec![];
        for (resource, info) in self.lock_table.iter_mut() {
            for owner in &info.shared_owners {
                if owner == transaction {
                    resources_to_unlock
                        .push(Operation::UnlockShared(*transaction, resource.to_owned()));
                }
            }
            if let Some(owner) = info.exclusive_owner {
                if owner == *transaction {
                    resources_to_unlock.push(Operation::UnlockExclusive(
                        *transaction,
                        resource.to_owned(),
                    ));
                }
            }
            info.remove_all(transaction);
        }

        return resources_to_unlock;
    }
}

#[derive(Debug)]
pub struct LockInfo {
    pub shared_owners: Vec<u32>,
    pub exclusive_owner: Option<u32>,
}

impl LockInfo {
    fn add_shared_owner(&mut self, shared_owner: &u32) {
        self.shared_owners.push(shared_owner.to_owned());
    }

    fn add_exclusive_owner(&mut self, exclusive_owner: &u32) {
        self.exclusive_owner = Some(exclusive_owner.to_owned());
    }

    fn remove_all(&mut self, transaction: &u32) {
        self.shared_owners.retain(|t| t != transaction);

        if self.exclusive_owner == Some(*transaction) {
            self.exclusive_owner = None;
        }
    }
}
