# PHP Doc Generator

PHP Doc Generator is a Rust-based tool that automatically generates documentation for PHP methods using the Claude AI model. It parses PHP files, extracts public methods, and creates concise, AI-generated documentation in Markdown format.

## Features

- Parses PHP files to extract all public methods
- Generates concise documentation for each method using Claude AI
- Provides brief descriptions for render methods
- Implements rate limiting with exponential backoff to handle API request limits
- Saves the generated documentation to a Markdown file

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
