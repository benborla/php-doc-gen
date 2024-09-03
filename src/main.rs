use dotenvy::dotenv;
use fancy_regex::Regex;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde_json::json;
use std::env;
use std::fs;
use std::io;
use std::process;
use thiserror::Error;

#[derive(Error, Debug)]
enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Regex error: {0}")]
    Regex(#[from] fancy_regex::Error),
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Environment variable error: {0}")]
    Env(#[from] env::VarError),
    #[error("API response error: {0}")]
    ApiResponse(Box<str>),
}

#[derive(Clone, Debug)]
struct Method {
    visibility: String,
    name: String,
    parameters: String,
    body: String,
    docblock: Option<String>,
    start_position: usize,
}

/// Parses a PHP file and extracts method information
///
/// # Arguments
///
/// * `file_path` - The path to the PHP file to parse
/// * `pb` - A progress bar to update during parsing
///
/// # Returns
///
/// A Result containing a vector of Method structs or an AppError
fn parse_php_file(file_path: &str, pb: &ProgressBar) -> Result<Vec<Method>, AppError> {
    pb.set_message("Parsing PHP file...");
    let contents = fs::read_to_string(file_path)?;
    let method_regex = Regex::new(
        r"(?ms)(/\*\*.*?\*/\s*)?\s*(public|protected|private)?\s*function\s+(\w+)\s*\((.*?)\)\s*\{(.*?)\n\s*\}",
    )?;

    let captures: Vec<_> = method_regex.captures_iter(&contents).collect();
    pb.set_length(captures.len() as u64);

    let methods = captures
        .into_iter()
        .filter_map(|cap_result| cap_result.ok())
        .map(|cap| {
            pb.inc(1);
            let docblock = cap.get(1).map(|m| m.as_str().to_string());
            let visibility = cap
                .get(2)
                .map_or("public".to_string(), |m| m.as_str().to_string());
            let name = cap
                .get(3)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let parameters = cap
                .get(4)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let body = cap
                .get(5)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();
            let start_position = cap.get(0).map(|m| m.start()).unwrap_or(0);

            Method {
                visibility,
                name,
                parameters,
                body,
                docblock,
                start_position,
            }
        })
        .collect();

    pb.finish_with_message("PHP file parsed successfully");
    Ok(methods)
}

/// Generates or updates docblocks for a list of methods
///
/// # Arguments
///
/// * `methods` - A slice of Method structs to generate docblocks for
/// * `client` - An HTTP client for making API requests
/// * `api_key` - The API key for authentication
/// * `pb` - A progress bar to update during docblock generation
///
/// # Returns
///
/// A Result containing a vector of generated docblocks as strings or an AppError
async fn generate_bulk_documentation(
    methods: &[Method],
    client: &Client,
    api_key: &str,
    pb: &ProgressBar,
) -> Result<Vec<String>, AppError> {
    pb.set_message("Generating docblocks...");
    let api_url = "https://api.anthropic.com/v1/messages";

    let methods_str = methods
        .iter()
        .enumerate()
        .map(|(i, method)| {
            format!(
                "Method {}:\n\
                Visibility: {}\n\
                Name: {}\n\
                Parameters: {}\n\
                Body:\n{}\n\
                Existing docblock (if any):\n{}\n",
                i + 1,
                method.visibility,
                method.name,
                method.parameters,
                method.body,
                method.docblock.as_deref().unwrap_or("None")
            )
        })
        .collect::<Vec<String>>()
        .join("\n---\n");

    let prompt = format!(
        "Generate PHP docblocks for the following {} methods. For each method, provide a concise description, \
        @param tags for each parameter, and @return tag if applicable. If there's an existing docblock, \
        improve it if it's vague or incomplete. Separate each docblock with '---'.\n\n{}",
        methods.len(),
        methods_str
    );

    pb.set_message("Sending request to Claude AI...");
    let response = client
        .post(api_url)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&json!({
            "model": "claude-3-sonnet-20240229",
            "max_tokens": 1500,
            "messages": [{"role": "user", "content": prompt}]
        }))
        .send()
        .await?;

    if response.status().is_success() {
        pb.set_message("Processing AI response...");
        let response_body: serde_json::Value = response.json().await?;
        let content = response_body["content"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|obj| obj["text"].as_str())
            .ok_or_else(|| {
                AppError::ApiResponse("Failed to extract content from API response".into())
            })?;

        let docblocks: Vec<String> = content
            .split("---")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if docblocks.len() != methods.len() {
            println!("Warning: Mismatch between number of methods and generated docblocks.");
            println!("Number of methods: {}", methods.len());
            println!("Number of generated docblocks: {}", docblocks.len());
            println!("AI response content:");
            println!("{}", content);

            let adjusted_docblocks = if docblocks.len() < methods.len() {
                let mut padded = docblocks;
                padded.extend(
                    std::iter::repeat(String::from("/** Generated docblock */"))
                        .take(methods.len() - padded.len()),
                );
                padded
            } else {
                docblocks.into_iter().take(methods.len()).collect()
            };

            println!("Adjusted number of docblocks to match methods. Some docblocks may be missing or incomplete.");
            pb.finish_with_message("Docblocks generated with warnings");
            Ok(adjusted_docblocks)
        } else {
            pb.finish_with_message("Docblocks generated successfully");
            Ok(docblocks)
        }
    } else {
        Err(AppError::ApiResponse(
            format!("API request failed with status: {}", response.status()).into(),
        ))
    }
}

/// Updates the PHP file with generated docblocks
///
/// # Arguments
///
/// * `file_path` - The path to the PHP file to update
/// * `methods` - A slice of Method structs containing the updated docblocks
/// * `pb` - A progress bar to update during file update
///
/// # Returns
///
/// A Result indicating success or an AppError
fn update_php_file(file_path: &str, methods: &[Method], pb: &ProgressBar) -> Result<(), AppError> {
    pb.set_message("Updating PHP file...");
    pb.set_length(methods.len() as u64);

    let mut contents = fs::read_to_string(file_path)?;
    let mut offset = 0;

    for method in methods.iter() {
        pb.inc(1);
        let insert_position = method.start_position + offset;

        if let Some(docblock) = &method.docblock {
            contents.insert_str(insert_position, &format!("\n{}\n", docblock));
            offset += docblock.len() + 2; // +2 for the newline characters
        }
    }

    fs::write(file_path, contents)?;

    pb.finish_with_message("PHP file updated successfully");
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

    let pb_style = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .unwrap()
        .progress_chars("##-");

    let pb = ProgressBar::new(100);
    pb.set_style(pb_style);

    let methods = parse_php_file(file_path, &pb)?;

    println!(
        "Generating or updating docblocks for {} methods in file: {}",
        methods.len(),
        file_path
    );

    let client = Client::new();
    let api_key = env::var("CLAUDE_API_KEY")?;

    let docblocks = generate_bulk_documentation(&methods, &client, &api_key, &pb).await?;

    let mut updated_methods = methods.clone();
    for (method, docblock) in updated_methods.iter_mut().zip(docblocks.iter()) {
        method.docblock = Some(docblock.clone());
    }

    update_php_file(file_path, &updated_methods, &pb)?;

    pb.finish_and_clear();
    println!("All tasks completed. Check the console for any warnings.");

    Ok(())
}
