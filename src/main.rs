use chrono::Local;
use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Priority {
    High,
    Medium,
    Low,
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::High => write!(f, "high"),
            Priority::Medium => write!(f, "medium"),
            Priority::Low => write!(f, "low"),
        }
    }
}

#[derive(Debug, Clone, ValueEnum, PartialEq)]
enum ListFilter {
    All,
    Done,
    Pending,
}

#[derive(Debug, Serialize, Deserialize)]
struct Todo {
    id: u32,
    title: String,
    completed: bool,
    priority: Priority,
    due_date: Option<String>,
    created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct TodoStore {
    next_id: u32,
    todos: Vec<Todo>,
}

#[derive(Parser)]
#[command(name = "todo-cli", about = "A simple CLI todo application")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new todo
    Add {
        /// Title of the todo
        title: String,
        /// Priority level
        #[arg(long, value_enum, default_value_t = Priority::Medium)]
        priority: Priority,
        /// Due date in YYYY-MM-DD format
        #[arg(long)]
        due: Option<String>,
    },
    /// List todos
    List {
        /// Filter todos
        #[arg(long, value_enum, default_value_t = ListFilter::Pending)]
        filter: ListFilter,
    },
    /// Mark a todo as completed
    Done {
        /// ID of the todo to complete
        id: u32,
    },
    /// Remove a todo
    Remove {
        /// ID of the todo to remove
        id: u32,
    },
}

fn store_path() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME environment variable not set");
    PathBuf::from(home).join(".todo-cli.json")
}

fn load_store() -> TodoStore {
    let path = store_path();
    if path.exists() {
        let data = fs::read_to_string(&path).expect("Failed to read store file");
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        TodoStore {
            next_id: 1,
            ..Default::default()
        }
    }
}

fn save_store(store: &TodoStore) {
    let path = store_path();
    let data = serde_json::to_string_pretty(store).expect("Failed to serialize store");
    fs::write(&path, data).expect("Failed to write store file");
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add {
            title,
            priority,
            due,
        } => {
            let mut store = load_store();
            let id = store.next_id;
            store.next_id += 1;
            let todo = Todo {
                id,
                title: title.clone(),
                completed: false,
                priority,
                due_date: due,
                created_at: Local::now().format("%Y-%m-%d").to_string(),
            };
            store.todos.push(todo);
            save_store(&store);
            println!("Added todo #{}: {}", id, title);
        }
        Commands::List { filter } => {
            let store = load_store();
            let todos: Vec<&Todo> = store
                .todos
                .iter()
                .filter(|t| match filter {
                    ListFilter::All => true,
                    ListFilter::Done => t.completed,
                    ListFilter::Pending => !t.completed,
                })
                .collect();

            if todos.is_empty() {
                println!("No todos found.");
                return;
            }

            println!(
                "{:<5} {:<6} {:<8} {:<12} Title",
                "ID", "Done", "Priority", "Due"
            );
            println!("{}", "-".repeat(60));
            for t in todos {
                let done = if t.completed { "[x]" } else { "[ ]" };
                let due = t.due_date.as_deref().unwrap_or("-");
                println!(
                    "{:<5} {:<6} {:<8} {:<12} {}",
                    t.id, done, t.priority, due, t.title
                );
            }
        }
        Commands::Done { id } => {
            let mut store = load_store();
            if let Some(todo) = store.todos.iter_mut().find(|t| t.id == id) {
                todo.completed = true;
                save_store(&store);
                println!("Marked todo #{} as done.", id);
            } else {
                eprintln!("Todo #{} not found.", id);
                std::process::exit(1);
            }
        }
        Commands::Remove { id } => {
            let mut store = load_store();
            let len_before = store.todos.len();
            store.todos.retain(|t| t.id != id);
            if store.todos.len() < len_before {
                save_store(&store);
                println!("Removed todo #{}.", id);
            } else {
                eprintln!("Todo #{} not found.", id);
                std::process::exit(1);
            }
        }
    }
}
