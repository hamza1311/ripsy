use shared::add;

#[tokio::main]
async fn main() {
    ripsy::client::init("http://localhost:3000");
    let r: Result<String, String> = add(2).await.unwrap();
    println!("{r:?}"); // Ok("2")
}
