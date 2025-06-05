use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::Json;
use axum::routing::get;
use axum::Router;
use ego_tree::NodeRef;
use http::Method;
use reqwest::Client;
use scraper::node::{Node, Text};
use scraper::{ElementRef, Html, Selector};
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
            "https://en.wiktionary.org/api/rest_v1/page/html/{}",
            text
        ))
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let status = response.status();

    if status.is_success() {
        let data = response
            .text()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let definitions = parse_wiktionary(&data);

        if let Some(definitions) = definitions {
            Ok(Json(json! {
                {
                    "status": "ok",
                    "html": definitions
                }
            }))
        } else {
            Ok(Json(json! {
                {
                    "status": "not found"
                }
            }))
        }
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

const WORD_ROLES: [&'static str; 11] = [
    "Noun",
    "Verb",
    "Adjective",
    "Adverb",
    "Pronoun",
    "Preposition",
    "Conjunction",
    "Interjection",
    "Determiner",
    "Article",
    "Participle",
];

fn parse_wiktionary(data: &str) -> Option<Vec<String>> {
    parse_wiktionary_layout1(data).or_else(|| parse_wiktionary_layout2(data))
}

// In the first possible Wiktionary HTML layout, the word definitions appear next
// to an element with the text "Spanish", and have the names in `WORD_ROLES`.
fn parse_wiktionary_layout1(data: &str) -> Option<Vec<String>> {
    let mut result = Vec::new();

    let document = Html::parse_document(data);

    let body_selector = Selector::parse("body").unwrap();
    let body = document.select(&body_selector).next().unwrap();

    for sections in body.children() {
        let mut children = sections.children();

        match children
            .next()
            .map(|i| i.value())
            .and_then(|i| i.as_element())
            .and_then(|i| i.id())
        {
            Some(id) if id == "Spanish" => {}
            _ => {
                continue;
            }
        };

        for outer_node in children {
            if let Some(text) = get_first_child_text(outer_node.children().next()) {
                if WORD_ROLES.iter().any(|i| **i == **text) {
                    let subsection_html = ElementRef::wrap(outer_node)?.inner_html();
                    result.push(subsection_html);
                };
            };
        }
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

// In the second possible Wiktionary HTML layout, the word definitions appear
// in special sections with headers starting with "Etymology", for multiple etymologies.
fn parse_wiktionary_layout2(data: &str) -> Option<Vec<String>> {
    let mut result = Vec::new();

    let document = Html::parse_document(data);

    let body_selector = Selector::parse("body").unwrap();
    let body = document.select(&body_selector).next().unwrap();

    for sections in body.children() {
        let mut children = sections.children();

        match children
            .next()
            .map(|i| i.value())
            .and_then(|i| i.as_element())
            .and_then(|i| i.id())
        {
            Some(id) if id == "Spanish" => {}
            _ => {
                continue;
            }
        };

        for outer_node in children {
            let mut children = outer_node.children();

            if let Some(text) = get_first_child_text(children.next()) {
                if text.starts_with("Etymology ") {
                    for outer_node in children {
                        if let Some(text) = get_first_child_text(outer_node.children().next()) {
                            if WORD_ROLES.iter().any(|i| **i == **text) {
                                let subsection_html = ElementRef::wrap(outer_node)?.inner_html();

                                // Replace the headings by lifting them one level up
                                // to get a consistent output with the other layout
                                result.push(
                                    subsection_html
                                        .replace("\u{003C}h4", "\u{003C}h3")
                                        .replace("\u{003C}/h4", "\u{003C}/h3")
                                        .replace("\u{003C}h5", "\u{003C}h4")
                                        .replace("\u{003C}/h5", "\u{003C}/h4"),
                                );
                            };
                        };
                    }
                };
            };
        }
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

fn get_first_child_text<'a>(node: Option<NodeRef<'a, Node>>) -> Option<&'a Text> {
    node.and_then(|node| node.children().next())
        .and_then(|child| child.value().as_text())
}
