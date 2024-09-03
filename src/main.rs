use dotenvy::dotenv;
use regex::Regex;
use reqwest::Client;
use serde_json::json;
use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process;
use std::time::Duration;
use tokio::time::sleep;
use thiserror::Error;

#[derive(Error, Debug)]
enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Environment variable error: {0}")]
    Env(#[from] env::VarError),
    #[error("API response error: {0}")]
    ApiResponse(String),
    #[error("Parse error: {0}")]
    Parse(String),
}

struct Method {
    name: String,
    parameters: Vec<String>,
    body: String,
    docblock: Option<String>,
}

fn parse_php_file(file_path: &str) -> Result<Vec<Method>, AppError> {
    let contents = fs::read_to_string(file_path)?;
    let method_regex = Regex::new(r"(?m)(/\*\*[\s\S]*?\*/\s*)?\s*public\s+function\s+(\w+)\s*\((.*?)\)\s*\{([\s\S]*?)\n\s*\}")?;

    let methods = method_regex
        .captures_iter(&contents)
        .map(|cap| {
            let docblock = cap.get(1).map(|m| m.as_str().to_string());
            let name = cap[2].to_string();
            let parameters = cap[3]
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            let body = cap[4].trim().to_string();

            Method {
                name,
                parameters,
                body,
                docblock,
            }
        })
        .collect();

    Ok(methods)
}

async fn generate_documentation(
    method: &Method,
    client: &Client,
    api_key: &str,
) -> Result<String, AppError> {
    let api_url = "https://api.anthropic.com/v1/messages";

    let prompt = format!(
        "Generate a PHP docblock for the following method:\n\
        Name: {}\n\
        Parameters: {}\n\
        Body:\n{}\n\
        Existing docblock (if any):\n{}\n\
        Provide a concise description, @param tags for each parameter, and @return tag if applicable. \
        If there's an existing docblock, improve it if it's vague or incomplete.",
        method.name,
        method.parameters.join(", "),
        method.body,
        method.docblock.as_deref().unwrap_or("None")
    );

    let max_retries = 3;
    let base_delay = Duration::from_secs(5);

    for retries in 0..max_retries {
        let response = client
            .post(api_url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&json!({
                "model": "claude-3-sonnet-20240229",
                "max_tokens": 300,
                "messages": [{"role": "user", "content": prompt}]
            }))
            .send()
            .await?;

        if response.status().is_success() {
            let response_body: serde_json::Value = response.json().await?;
            let docblock = response_body["content"]
                .as_array()
                .and_then(|arr| arr.first())
                .and_then(|obj| obj["text"].as_str())
                .ok_or_else(|| AppError::ApiResponse("Failed to extract docblock from API response".into()))?
                .trim()
                .to_string();

            if docblock.is_empty() {
                return Err(AppError::ApiResponse("Generated docblock is empty".into()));
            }

            return Ok(docblock);
        } else if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            if retries == max_retries - 1 {
                return Err(AppError::ApiResponse("Max retries reached due to rate limiting".into()));
            }
            let delay = base_delay * 2u32.pow(retries as u32);
            println!("Rate limit hit. Retrying in {} seconds...", delay.as_secs());
            sleep(delay).await;
        } else {
            return Err(AppError::ApiResponse(format!("API request failed with status: {}", response.status())));
        }
    }

    Err(AppError::ApiResponse("Failed to generate documentation after max retries".into()))
}

fn update_php_file(file_path: &str, methods: &[Method]) -> Result<(), AppError> {
    let mut contents = fs::read_to_string(file_path)?;

    for method in methods.iter().rev() {
        let method_regex = Regex::new(&format!(
            r"(?m)(/\*\*[\s\S]*?\*/\s*)?\s*public\s+function\s+{}\s*\((.*?)\)\s*\{{",
            regex::escape(&method.name)
        ))?;

        if let Some(mat) = method_regex.find(&contents) {
            let start = mat.start();
            let end = mat.end();

            let updated_method = format!(
                "{}\npublic function {}({})",
                method.docblock.as_ref().unwrap_or(&String::new()),
                method.name,
                method.parameters.join(", ")
            );

            contents.replace_range(start..end, &updated_method);
        }
    }

    fs::write(file_path, contents)?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenv().ok();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_php_file>", args[0]);
        process::exit(1);
    }

    let file_path = &args[1];

    let mut methods = parse_php_file(file_path)?;

    println!(
        "Generating or updating docblocks for public methods in file: {}",
        file_path
    );

    let client = Client::new();
    let api_key = env::var("CLAUDE_API_KEY")?;

    for method in &mut methods {
        match generate_documentation(method, &client, &api_key).await {
            Ok(docblock) => {
                println!("Generated docblock for {}", method.name);
                method.docblock = Some(docblock);
            }
            Err(e) => eprintln!("Error generating docblock for {}: {}", method.name, e),
        }
        sleep(Duration::from_secs(1)).await;
    }

    update_php_file(file_path, &methods)?;
    println!("PHP file updated successfully with new docblocks.");

    Ok(())
}
