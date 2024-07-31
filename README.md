# Rust Wget Clone

Simple wget clone written in Rust. It is a practice project to explore Rust's capabilities in building command-line tools. This program allows you to download files from the internet with support for pausing and resuming downloads as well as a logging to keep track of them.

## Features

- Download files from URLs
- Pause and resume downloads using `Ctrl+C`
- Progress bar indicating download status
- Log download history with timestamps
- Handles HTTP redirections

## Installation

### Prerequisites

- Rust and Cargo installed. If not, follow the [official installation guide](https://doc.rust-lang.org/book/ch01-01-installation.html).

### Building from Source

1. Clone the repository:

   ```sh
   git clone https://github.com/yourusername/rget.git
   cd rget

2. Build the project:

   ```sh
   cargo build

## Usage

### Running the Program
1. To download a file, run:

   ```sh
   cargo run -- https://example.com/file.zip

2. Press Ctrl+C to pause the download. Run the same command again to resume from where it left off.


Inspired on Matt Gathu's Guide for [Writing CLI apps with Rust](https://mattgathu.dev/2017/08/29/writing-cli-app-rust.html)

