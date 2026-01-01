use std::collections::HashMap;

// ---------- modelssss ----------

#[derive(Debug)]
struct User {
    id: String,
    balance: i64,
    last_tx_timestamp: u64,
}

#[derive(Debug)]
enum TxType {
    Transfer,
}

#[derive(Debug)]
struct Transaction {
    from: String,
    to: String,
    amount: i64,
    timestamp: u64,
    tx_type: TxType,
}

// ---------- handlign custom erors ----------

#[derive(Debug)]
enum TxError {
    SameSenderReceiver,
    InvalidAmount,
    InsufficientBalance,
    RateLimitExceeded,
    UserNotFound,
}

// ---------- TxGuard engine ----------

struct TxGuard {
    users: HashMap<String, User>,
    min_tx_interval: u64,
}

impl TxGuard {
    fn new(min_tx_interval: u64) -> Self {
        Self {
            users: HashMap::new(),
            min_tx_interval,
        }
    }

    fn add_user(&mut self, id: &str, balance: i64) {
        self.users.insert(
            id.to_string(),
            User {
                id: id.to_string(),
                balance,
                last_tx_timestamp: 0,
            },
        );
    }

    fn validate(&self, tx: &Transaction) -> Result<(), TxError> {
        if tx.from == tx.to {
            return Err(TxError::SameSenderReceiver);
        }

        if tx.amount <= 0 {
            return Err(TxError::InvalidAmount);
        }

        let sender = self.users.get(&tx.from).ok_or(TxError::UserNotFound)?;

        if sender.balance < tx.amount {
            return Err(TxError::InsufficientBalance);
        }

        if tx.timestamp - sender.last_tx_timestamp < self.min_tx_interval {
            return Err(TxError::RateLimitExceeded);
        }

        Ok(())
    }

    fn execute(&mut self, tx: Transaction) -> Result<(), TxError> {
        self.validate(&tx)?;

        let sender = self.users.get_mut(&tx.from).unwrap();
        let receiver = self.users.get_mut(&tx.to).unwrap();

        sender.balance -= tx.amount;
        sender.last_tx_timestamp = tx.timestamp;

        receiver.balance += tx.amount;

        Ok(())
    }
}



fn main() {
    let mut guard = TxGuard::new(10);

    guard.add_user("ram", 100);
    guard.add_user("raghu", 50);

    let tx = Transaction {
        from: "ram".to_string(),
        to: "raghu".to_string(),
        amount: 30,
        timestamp: 20,
        tx_type: TxType::Transfer,
    };

    match guard.execute(tx) {
        Ok(_) => println!("Transaction approved"),
        Err(e) => println!("Transaction rejected: {:?}", e),
    }

    println!("State: {:?}", guard.users);
}
