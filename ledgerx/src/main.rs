struct Account {
    id: String,
    balance: i64,
} 
fn main(){
    let mut ram = Account {
        id: String::from("ram"),
        balance:0,
    };

    ram.balance += 100;
    println!("Balance {}",ram.balance);

}