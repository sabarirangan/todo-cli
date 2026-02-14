# todo-cli

A simple command-line todo application built in Rust.

## Features

- Add todos with priority levels (high, medium, low) and optional due dates
- List todos filtered by status (pending, done, all)
- Mark todos as completed
- Remove todos
- Persistent storage via JSON (`~/.todo-cli.json`)

## Installation

```sh
cargo install --path .
```

## Usage

```sh
# Add a todo
todo-cli add "Buy groceries" --priority high --due 2026-02-20
todo-cli add "Read a book" --priority low

# List pending todos (default)
todo-cli list

# List all or completed todos
todo-cli list --filter all
todo-cli list --filter done

# Mark a todo as done
todo-cli done 1

# Remove a todo
todo-cli remove 2
```

## Building from source

```sh
git clone https://github.com/sabarirangan/todo-cli.git
cd todo-cli
cargo build --release
```
