use easy_schedule::prelude::*;

fn main() {
    println!("Easy Schedule Error Handling Example");
    println!("====================================");

    // ✅ Valid parsing examples
    println!("\n✅ Valid parsing:");
    let valid_tasks = vec![
        "wait(10)",
        "interval(30)",
        "at(14:30)",
        "once(2024-12-31 23:59:59 +08)",
    ];

    for task_str in valid_tasks {
        match Task::parse(task_str) {
            Ok(task) => println!("✓ '{task_str}' -> {task}"),
            Err(err) => println!("✗ '{task_str}' -> Error: {err}"),
        }
    }

    // ❌ Invalid parsing examples
    println!("\n❌ Invalid parsing examples:");
    let invalid_tasks = vec![
        "wait abc",           // No parentheses
        "wait(abc)",          // Invalid number
        "wait(10",            // Missing closing parenthesis
        "at(25:70)",          // Invalid time
        "unknown(123)",       // Unknown task type
        "interval()",         // Empty arguments
        "once(invalid-date)", // Invalid date format
    ];

    for task_str in invalid_tasks {
        match Task::parse(task_str) {
            Ok(task) => println!("✓ '{task_str}' -> {task}"),
            Err(err) => println!("✗ '{task_str}' -> Error: {err}"),
        }
    }

    // Recommended usage pattern
    println!("\n🛠️  Recommended usage pattern:");
    let user_input = "wait(invalid)";

    match Task::parse(user_input) {
        Ok(task) => {
            println!("Task created successfully: {task}");
            // Use the task...
        }
        Err(err) => {
            println!("Failed to create task: {err}");
            println!("Please check your input format.");
            // Show help or default behavior...
        }
    }

    // Using From (panics on error)
    println!("\n⚠️  Using From trait (will panic on error):");
    println!("Task::from(\"wait(5)\") -> {}", Task::from("wait(5)"));

    // This would panic:
    // println!("Task::from(\"invalid\") -> {}", Task::from("invalid"));
    println!("Task::from(\"invalid\") would panic with detailed error message");
}
