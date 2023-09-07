# Ripsy

RPC between Server/Client written in Rust

## Why Ripsy?

RRPC (Rust Remote Procedure Call) is pronounced as ripsy, thus the name of this crate.

## Example

**client/main.rs**
```rust
use shared::add;

#[tokio::main]
async fn main() {
    ripsy::client::init("http://localhost:3000");
    let r: Result<String, String> = add(2).await.unwrap();
    println!("{r:?}"); // Ok("2")
}
```

**server/main.rs**
```rust
use axum::routing::post;
use ripsy::Bincode;
use shared::add;

#[tokio::main]
async fn main() {
    let app = ripsy::ripsy!(add,);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

**shared/lib.rs**
```rust
use ripsy::endpoint;

#[endpoint(mutation)]
pub async fn add(n: u32) -> Result<String, String> {
    if false {
        work()?; // ? works fine
    }
    Ok(n.to_string())
}

fn work() -> Result<(), String> { Err("err".to_string())  }
```

