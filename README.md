# PHP Doc Generator

PHP Doc Generator is a Rust-based tool that automatically enhances documentation for PHP methods using the Claude AI model. It scans PHP files, extracts methods, and either creates new or improves existing PHPDoc blocks directly within the source files.

## Key Features:

1. **PHP File Parsing**: Efficiently parses PHP files to identify public methods and their existing DocBlocks.

2. **AI-Powered Documentation**: Utilizes the Claude AI model to generate or improve PHPDoc blocks for each method.

3. **In-Place Updates**: Modifies the original PHP files, adding or enhancing DocBlocks without creating separate documentation files.

4. **Intelligent Docblock Generation**: Creates concise descriptions, parameter tags, and return tags tailored to each method's signature and body.

5. **Existing Docblock Enhancement**: If a method already has a docblock, the tool will improve it if it's vague or incomplete.

6. **Rate Limiting Handling**: Implements a retry mechanism with exponential backoff to handle API rate limits gracefully.

7. **Error Handling**: Robust error handling and reporting for various stages of the process.

This tool streamlines the documentation process for PHP projects, ensuring that all public methods have clear, accurate, and up-to-date DocBlocks. It's particularly useful for large codebases or when onboarding new developers, as it provides consistent and comprehensive documentation directly within the source code.

## Installation

1. Ensure you have Rust and Cargo installed on your system. If not, you can install them from [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install).

2. Clone the repository:
   ```
   git clone https://github.com/benborla/php-doc-gen.git
   cd php-doc-gen
   ```

3. Build the project:
   ```
   cargo build --release
   ```

## Usage

1. Create a `.env` file in the project root directory with your Claude API key:
   ```
   CLAUDE_API_KEY=your_api_key_here
   ```

2. Run the program with your PHP file as an argument:
   ```
   cargo run --release -- path/to/your/php/file.php
   ```

   This will generate a Markdown file with the same name as your PHP file (but with a .md extension) in the same directory.

## Dependencies

Make sure you have the following dependencies in your `Cargo.toml`:

```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
serde_json = "1.0"
regex = "1.5"
dotenvy = "0.15"
```

## License

MIT License

Copyright (c) 2024 benborla

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Support

If you encounter any problems or have any questions, please open an issue on the [GitHub repository](https://github.com/benborla/php-doc-gen/issues).
