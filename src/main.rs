use dotenvy::dotenv;
use regex::Regex;
use reqwest;
use serde_json::json;
use std::env;
use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process;
use std::time::Duration;
use tokio::time::sleep;

struct Method {
    name: String,
    parameters: Vec<String>,
    body: String,
}

fn parse_php_file(file_path: &str) -> Result<Vec<Method>, Box<dyn Error>> {
    let contents = fs::read_to_string(file_path)?;
    let method_regex =
        Regex::new(r"(?m)^\s*public\s+function\s+(\w+)\s*\((.*?)\)\s*\{([\s\S]*?)\n\s*\}")?;

    let mut methods = Vec::new();

    for cap in method_regex.captures_iter(&contents) {
        let name = cap[1].to_string();
        let parameters = cap[2].split(',').map(|s| s.trim().to_string()).collect();
        let body = cap[3].trim().to_string();

        methods.push(Method {
            name,
            parameters,
            body,
        });
    }

    Ok(methods)
}

async fn generate_documentation(
    method: &Method,
    client: &reqwest::Client,
) -> Result<String, Box<dyn Error>> {
    let api_url = "https://api.anthropic.com/v1/messages";
    let api_key = std::env::var("CLAUDE_API_KEY").expect("CLAUDE_API_KEY not set");

    let prompt = if method.name.starts_with("render") {
        format!(
            "Write a single brief sentence describing the following render method in a PHP web application:\n\
            Name: {}\n\
            Parameters: {}\n\
            Focus only on what view or template this method renders, without any additional details.",
            method.name,
            method.parameters.join(", ")
        )
    } else {
        format!(
            "Provide a brief and direct documentation for the following PHP method:\n\
            Name: {}\n\
            Parameters: {}\n\
            Body:\n{}\n\
            Include a short description (1-2 sentences), list the parameters with very brief explanations, \
            and mention the return value if applicable. Keep it concise.",
            method.name,
            method.parameters.join(", "),
            method.body
        )
    };

    let mut retries = 0;
    let max_retries = 3;
    let base_delay = Duration::from_secs(5);

    loop {
        let response = client
            .post(api_url)
            .header("x-api-key", &api_key)
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
            let documentation = response_body["content"]
                .as_array()
                .and_then(|arr| arr.first())
                .and_then(|obj| obj["text"].as_str())
                .ok_or("Failed to extract documentation from API response")?
                .trim()
                .to_string();

            if documentation.is_empty() {
                return Err("Extracted documentation is empty".into());
            }

            return Ok(documentation);
        } else if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            if retries >= max_retries {
                return Err("Max retries reached due to rate limiting".into());
            }
            let delay = base_delay * 2u32.pow(retries as u32);
            println!("Rate limit hit. Retrying in {} seconds...", delay.as_secs());
            sleep(delay).await;
            retries += 1;
        } else {
            return Err(format!("API request failed with status: {}", response.status()).into());
        }
    }
}

fn save_documentation_to_file(input_file: &str, documentation: &str) -> Result<(), Box<dyn Error>> {
    let input_path = Path::new(input_file);
    let file_stem = input_path.file_stem().unwrap().to_str().unwrap();
    let output_file = input_path.with_file_name(format!("{}.md", file_stem));

    let mut file = fs::File::create(output_file)?;
    file.write_all(documentation.as_bytes())?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_php_file>", args[0]);
        process::exit(1);
    }

    let file_path = &args[1];

    let methods = match parse_php_file(file_path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error parsing PHP file: {}", e);
            process::exit(1);
        }
    };

    println!(
        "Generating documentation for public methods in file: {}",
        file_path
    );

    let mut all_documentation = String::new();
    let client = reqwest::Client::new();

    for method in methods {
        match generate_documentation(&method, &client).await {
            Ok(documentation) => {
                println!("Generated documentation for {}", method.name);
                all_documentation.push_str(&format!("## {}\n\n{}\n\n", method.name, documentation));
            }
            Err(e) => eprintln!("Error generating documentation for {}: {}", method.name, e),
        }
        // Add a delay between requests to avoid rate limiting
        sleep(Duration::from_secs(1)).await;
    }

    if !all_documentation.is_empty() {
        match save_documentation_to_file(file_path, &all_documentation) {
            Ok(()) => println!("Documentation saved successfully."),
            Err(e) => eprintln!("Error saving documentation to file: {}", e),
        }
    } else {
        println!(
            "No documentation was generated. Make sure your PHP file contains public methods."
        );
    }

    Ok(())
}
