use coral::{parse_coral, run_coral};
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <coral_file> [--parse-only]", args[0]);
        eprintln!("  --parse-only: Only parse the file, don't execute");
        process::exit(1);
    }
    
    let filename = &args[1];
    let parse_only = args.len() > 2 && args[2] == "--parse-only";
    
    let input = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", filename, err);
            process::exit(1);
        }
    };
    
    if parse_only {
        // Parse only mode
        match parse_coral(&input) {
            Ok(program) => {
                println!("✓ Successfully parsed {} statements from '{}'", program.statements.len(), filename);
                
                for (i, stmt) in program.statements.iter().enumerate() {
                    println!("Statement {}: {:?}", i + 1, stmt);
                }
            }
            Err(errors) => {
                eprintln!("✗ Parse errors in '{}':", filename);
                for error in errors {
                    eprintln!("  - {}", error);
                }
                process::exit(1);
            }
        }
    } else {
        // Parse and execute mode
        match run_coral(&input) {
            Ok(result) => {
                println!("✓ Program '{}' executed successfully", filename);
                println!("Final result: {}", result);
            }
            Err(error) => {
                eprintln!("✗ Execution error in '{}': {}", filename, error);
                process::exit(1);
            }
        }
    }
}