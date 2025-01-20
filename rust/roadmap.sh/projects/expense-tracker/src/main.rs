use std::io::{Result, Write};

use clap::{Parser, Subcommand};
use prettytable::{row, Table};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add an expense with a description and amount
    Add {
        /// Expense's description
        #[arg(short, long)]
        description: String,

        /// Expense's amount
        #[arg(short, long)]
        amount: f64,
    },

    /// Delete an existing expense given its ID
    Delete {
        /// Expense's ID
        #[arg(short, long)]
        id: u32,
    },

    /// List all expenses
    List,
}

/// Represents an expense
#[derive(Serialize, Deserialize, Debug)]
struct Expense {
    id: u32,
    description: String,
    amount: f64,
}

const DATASTORE_PATH: &str = "datastore.json";

fn init_datastore(path: &str) -> Result<()> {
    if !std::path::Path::new(path).exists() {
        std::fs::File::create(path)?.write_all(b"[]")?;
        println!("Datastore initialized at '{}'", path);
        return Ok(());
    }

    println!("Reading from datastore at '{}'", path);

    Ok(())
}

fn read_expenses(path: &str) -> Result<Vec<Expense>> {
    let data = std::fs::read_to_string(path)?;
    let expenses: Vec<Expense> = serde_json::from_str(&data)?;
    Ok(expenses)
}

fn write_expenses(path: &str, expenses: &[Expense]) -> Result<()> {
    let data = serde_json::to_string(expenses)?;
    std::fs::write(path, data)?;
    Ok(())
}

fn add_expense(description: String, amount: f64, path: &str) {
    let mut expenses = match read_expenses(path) {
        Ok(data) => data,
        Err(error) => {
            panic!("Failed to read from datastore: {}", error);
        }
    };

    let next_id = expenses.iter().map(|e| e.id).max().unwrap_or(0) + 1;

    let expense = Expense {
        id: next_id,
        description,
        amount,
    };
    expenses.push(expense);

    if let Err(error) = write_expenses(path, &expenses) {
        panic!("Failed to write to datastore: {}", error);
    }

    println!("Expense added successfully with ID: {}", next_id);
}

fn delete_expense(id: u32, path: &str) {
    let mut expenses = match read_expenses(path) {
        Ok(data) => data,
        Err(error) => {
            panic!("Failed to read from datastore: {}", error);
        }
    };

    let original_len = expenses.len();

    expenses.retain(|expense| expense.id != id);

    if expenses.len() == original_len {
        println!("No expense found with ID: {}", id);
        return;
    }

    if let Err(error) = write_expenses(path, &expenses) {
        panic!("Failed to write to datastore: {}", error);
    }

    println!("Expense with ID: '{}' deleted successfully", id);
}

fn list_expenses(path: &str) {
    let expenses = match read_expenses(path) {
        Ok(data) => data,
        Err(error) => {
            panic!("Failed to read from datastore: {}", error);
        }
    };

    if expenses.is_empty() {
        println!("No expenses found");
        return;
    }

    let mut table = Table::new();

    table.add_row(row!["ID", "Description", "Amount"]);

    for expense in expenses {
        table.add_row(row![
            expense.id,
            expense.description,
            format!("{:.2}", expense.amount)
        ]);
    }

    table.printstd();
}

fn main() {
    let args = Args::parse();

    if let Err(error) = init_datastore(DATASTORE_PATH) {
        panic!("Failed to initialize datastore: {}", error);
    }

    match &args.command {
        Some(Commands::Add {
            description,
            amount,
        }) => {
            add_expense(description.clone(), *amount, DATASTORE_PATH);
        }
        Some(Commands::Delete { id }) => {
            delete_expense(*id, DATASTORE_PATH);
        }
        Some(Commands::List) => {
            list_expenses(DATASTORE_PATH);
        }
        None => {}
    }
}
