use coral::codegen::LLVMCodegen;
use coral::*;
use std::env;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        let filename = &args[1];
        let content = fs::read_to_string(filename)
            .map_err(|e| format!("Failed to read file {}: {}", filename, e))?;
        
        match parse_and_analyze(&content) {
            Ok((program, analysis_result)) => {
                if let Err(errors) = analysis_result {
                    eprintln!("Semantic analysis failed:");
                    for error in errors {
                        eprintln!("- {}", error.message);
                    }
                    return Err("Semantic analysis failed".into());
                }

                let mut codegen = LLVMCodegen::new("main_module".to_string());
                match codegen.compile_program(&program) {
                    Ok(ir) => {
                        println!("{}", ir);
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("Codegen error: {:?}", e);
                        Err("Codegen failed".into())
                    }
                }
            }
            Err(e) => {
                eprintln!("Parse error: {}", e);
                Err("Parsing failed".into())
            }
        }
    } else {
        eprintln!("Usage: coral <filename>");
        Ok(())
    }
}

