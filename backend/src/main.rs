use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::Json;
use axum::routing::get;
use axum::Router;
use http::Method;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define the CORS layer
    let cors = CorsLayer::new()
        // Alternatively, use Any to allow all origins (less secure, for dev only)
        .allow_origin(Any)
        .allow_methods([Method::GET]);

    // Build an axum router
    let app = Router::new()
        .route("/api/v1/spanish/word/{text}", get(look_up_spanish))
        .layer(cors);

    // Create a tokio-based TCP listener
    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    println!("Listening on {}", listener.local_addr()?);

    // Run the axum app with tokio
    axum::serve(listener, app).await?;

    Ok(())
}

// Handler for the root route
async fn look_up_spanish(Path(text): Path<String>) -> Result<Json<Value>, StatusCode> {
    let response = Client::new()
        .get(format!(
            "https://en.wiktionary.org/api/rest_v1/page/definition/{}",
            text
        ))
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let status = response.status();

    if status.is_success() {
        let data: Value = response
            .json()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let definitions = parse_wiktionary(data);

        Ok(Json(json! {
            {
                "status": "ok",
                "definitions": definitions
            }
        }))
    } else if status.as_u16() == 404 {
        // Still return a `200` status code here,
        // so that the user can handle this at the
        // response level instead of catching HTTP errors
        Ok(Json(json! {
            {
                "status": "not found"
            }
        }))
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Dictionary {
    part_of_speech: String,
    definitions: Vec<Definition>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Definition {
    definition: String,
    #[serde(default)]
    examples: Vec<Example>,
    #[serde(default)]
    parsed_examples: Vec<ParsedExample>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Example(String);

#[derive(Debug, Serialize, Deserialize)]
struct ParsedExample {
    example: String,
    translation: String,
}

fn parse_wiktionary(mut data: Value) -> Option<Vec<Dictionary>> {
    let spanish_section = data.get_mut("es")?.take();

    serde_json::from_value(spanish_section).ok()
}
