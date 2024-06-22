# phargs

## Multiple Command Runner

This command line tool allows you to execute multiple commands provided via comma-separated arguments. It features a dry-run mode to preview commands without executing them.

## Features

* Multiple Commands: Run multiple commands in one go.
* Dry Run: Preview the commands that would be executed without actually running them.

## Prerequisites

* Rust and Cargo (latest stable version recommended)

## Building the Project

To build the project, ensure you have Rust and Cargo installed on your system. Clone the repository and navigate to the project directory:

``bash
git clone https://github.com/yamaura/phargs.git
cd phargs
``

Then, build the project using Cargo:

``bash
cargo build --release
``

The executable will be located in `./target/release/`.

## Usage

To run multiple commands, use the following syntax:

``bash
phargs [OPTIONS] -w file1.txt,file2.txt -- echo {}
``

This will print commands like:

``bash
echo file1.txt
echo file2.txt
``


## Error Handling

 It also handles command execution failures and will log errors appropriately. If a command fails, the tool exits with the command's exit code.
