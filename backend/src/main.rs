use axum::{response::Html, routing::get, Router};
use tokio::net::TcpListener;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build an axum router
    let app = Router::new().route("/", get(handler));

    // Create a tokio-based TCP listener
    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    println!("Listening on {}", listener.local_addr()?);

    // Run the axum app with tokio
    axum::serve(listener, app).await?;

    Ok(())
}

// Handler for the root route
async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}
