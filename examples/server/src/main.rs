use shared::add;

#[tokio::main]
async fn main() {
    let app = ripsy::ripsy!(add,);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
