use chrono::Local;
use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
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

fn load_store(path: &Path) -> TodoStore {
    if path.exists() {
        let data = fs::read_to_string(path).expect("Failed to read store file");
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        TodoStore {
            next_id: 1,
            ..Default::default()
        }
    }
}

fn save_store(store: &TodoStore, path: &Path) {
    let data = serde_json::to_string_pretty(store).expect("Failed to serialize store");
    fs::write(path, data).expect("Failed to write store file");
}

fn add_todo(
    store: &mut TodoStore,
    title: String,
    priority: Priority,
    due: Option<String>,
) -> u32 {
    let id = store.next_id;
    store.next_id += 1;
    let todo = Todo {
        id,
        title,
        completed: false,
        priority,
        due_date: due,
        created_at: Local::now().format("%Y-%m-%d").to_string(),
    };
    store.todos.push(todo);
    id
}

fn mark_done(store: &mut TodoStore, id: u32) -> bool {
    if let Some(todo) = store.todos.iter_mut().find(|t| t.id == id) {
        todo.completed = true;
        true
    } else {
        false
    }
}

fn remove_todo(store: &mut TodoStore, id: u32) -> bool {
    let len_before = store.todos.len();
    store.todos.retain(|t| t.id != id);
    store.todos.len() < len_before
}

fn filter_todos<'a>(store: &'a TodoStore, filter: &ListFilter) -> Vec<&'a Todo> {
    store
        .todos
        .iter()
        .filter(|t| match filter {
            ListFilter::All => true,
            ListFilter::Done => t.completed,
            ListFilter::Pending => !t.completed,
        })
        .collect()
}

fn main() {
    let cli = Cli::parse();
    let path = store_path();

    match cli.command {
        Commands::Add {
            title,
            priority,
            due,
        } => {
            let mut store = load_store(&path);
            let id = add_todo(&mut store, title.clone(), priority, due);
            save_store(&store, &path);
            println!("Added todo #{}: {}", id, title);
        }
        Commands::List { filter } => {
            let store = load_store(&path);
            let todos = filter_todos(&store, &filter);

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
            let mut store = load_store(&path);
            if mark_done(&mut store, id) {
                save_store(&store, &path);
                println!("Marked todo #{} as done.", id);
            } else {
                eprintln!("Todo #{} not found.", id);
                std::process::exit(1);
            }
        }
        Commands::Remove { id } => {
            let mut store = load_store(&path);
            if remove_todo(&mut store, id) {
                save_store(&store, &path);
                println!("Removed todo #{}.", id);
            } else {
                eprintln!("Todo #{} not found.", id);
                std::process::exit(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn empty_store() -> TodoStore {
        TodoStore {
            next_id: 1,
            todos: Vec::new(),
        }
    }

    fn temp_path() -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("todo-cli-test-{}.json", std::process::id()));
        path
    }

    // -- add_todo tests --

    #[test]
    fn add_todo_assigns_incrementing_ids() {
        let mut store = empty_store();
        let id1 = add_todo(&mut store, "First".into(), Priority::Low, None);
        let id2 = add_todo(&mut store, "Second".into(), Priority::High, None);
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(store.next_id, 3);
    }

    #[test]
    fn add_todo_stores_fields_correctly() {
        let mut store = empty_store();
        add_todo(
            &mut store,
            "Buy milk".into(),
            Priority::High,
            Some("2026-03-01".into()),
        );
        assert_eq!(store.todos.len(), 1);
        let todo = &store.todos[0];
        assert_eq!(todo.title, "Buy milk");
        assert_eq!(todo.priority, Priority::High);
        assert_eq!(todo.due_date.as_deref(), Some("2026-03-01"));
        assert!(!todo.completed);
    }

    #[test]
    fn add_todo_defaults_to_not_completed() {
        let mut store = empty_store();
        add_todo(&mut store, "Task".into(), Priority::Medium, None);
        assert!(!store.todos[0].completed);
    }

    // -- mark_done tests --

    #[test]
    fn mark_done_existing_todo() {
        let mut store = empty_store();
        add_todo(&mut store, "Task".into(), Priority::Medium, None);
        assert!(mark_done(&mut store, 1));
        assert!(store.todos[0].completed);
    }

    #[test]
    fn mark_done_nonexistent_returns_false() {
        let mut store = empty_store();
        assert!(!mark_done(&mut store, 99));
    }

    #[test]
    fn mark_done_idempotent() {
        let mut store = empty_store();
        add_todo(&mut store, "Task".into(), Priority::Low, None);
        assert!(mark_done(&mut store, 1));
        assert!(mark_done(&mut store, 1));
        assert!(store.todos[0].completed);
    }

    // -- remove_todo tests --

    #[test]
    fn remove_existing_todo() {
        let mut store = empty_store();
        add_todo(&mut store, "Task".into(), Priority::Medium, None);
        assert!(remove_todo(&mut store, 1));
        assert!(store.todos.is_empty());
    }

    #[test]
    fn remove_nonexistent_returns_false() {
        let mut store = empty_store();
        assert!(!remove_todo(&mut store, 99));
    }

    #[test]
    fn remove_only_target_todo() {
        let mut store = empty_store();
        add_todo(&mut store, "Keep".into(), Priority::Low, None);
        add_todo(&mut store, "Remove".into(), Priority::High, None);
        assert!(remove_todo(&mut store, 2));
        assert_eq!(store.todos.len(), 1);
        assert_eq!(store.todos[0].title, "Keep");
    }

    // -- filter_todos tests --

    #[test]
    fn filter_pending_excludes_done() {
        let mut store = empty_store();
        add_todo(&mut store, "Pending".into(), Priority::Low, None);
        add_todo(&mut store, "Done".into(), Priority::Low, None);
        mark_done(&mut store, 2);
        let result = filter_todos(&store, &ListFilter::Pending);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "Pending");
    }

    #[test]
    fn filter_done_excludes_pending() {
        let mut store = empty_store();
        add_todo(&mut store, "Pending".into(), Priority::Low, None);
        add_todo(&mut store, "Done".into(), Priority::Low, None);
        mark_done(&mut store, 2);
        let result = filter_todos(&store, &ListFilter::Done);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "Done");
    }

    #[test]
    fn filter_all_returns_everything() {
        let mut store = empty_store();
        add_todo(&mut store, "A".into(), Priority::Low, None);
        add_todo(&mut store, "B".into(), Priority::High, None);
        mark_done(&mut store, 2);
        let result = filter_todos(&store, &ListFilter::All);
        assert_eq!(result.len(), 2);
    }

    // -- persistence tests --

    #[test]
    fn save_and_load_round_trip() {
        let path = temp_path();
        let mut store = empty_store();
        add_todo(
            &mut store,
            "Persist me".into(),
            Priority::High,
            Some("2026-12-31".into()),
        );
        mark_done(&mut store, 1);

        save_store(&store, &path);
        let loaded = load_store(&path);

        assert_eq!(loaded.next_id, 2);
        assert_eq!(loaded.todos.len(), 1);
        assert_eq!(loaded.todos[0].title, "Persist me");
        assert_eq!(loaded.todos[0].priority, Priority::High);
        assert!(loaded.todos[0].completed);
        assert_eq!(loaded.todos[0].due_date.as_deref(), Some("2026-12-31"));

        fs::remove_file(&path).ok();
    }

    #[test]
    fn load_nonexistent_returns_empty_store() {
        let path = PathBuf::from("/tmp/todo-cli-does-not-exist.json");
        let store = load_store(&path);
        assert_eq!(store.next_id, 1);
        assert!(store.todos.is_empty());
    }

    // -- Priority display tests --

    #[test]
    fn priority_display() {
        assert_eq!(Priority::High.to_string(), "high");
        assert_eq!(Priority::Medium.to_string(), "medium");
        assert_eq!(Priority::Low.to_string(), "low");
    }
}
