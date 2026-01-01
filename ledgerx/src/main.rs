struct Account {
    id: String,
    balance: i64,
}


fn credit(account: &mut Account, amount: i64) {
    account.balance += amount;
}

fn debit(account: &mut Account, amount: i64) -> Result<(), String> {
    if account.balance < amount {
        return Err(String::from("Insufficient balance"));
    }

    account.balance -= amount;
    Ok(())
}

fn main() {
    let mut alice = Account {
        id: String::from("alice"),
        balance: 0,
    };

    
    credit(&mut alice, 100);

    
    match debit(&mut alice, 40) {
        Ok(_) => println!("Debit successful"),
        Err(e) => println!("Error: {}", e),
    }

    println!("Final Balance: {}", alice.balance);
}
