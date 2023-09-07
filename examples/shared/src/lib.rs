use ripsy::endpoint;

#[endpoint(mutation)]
pub async fn add(n: u32) -> Result<String, String> {
    if false {
        work()?;
    }
    Ok(n.to_string())
}

fn work() -> Result<(), String> {
    Err("err".to_string())
}
