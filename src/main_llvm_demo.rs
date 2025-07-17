use coral::*;
use coral::codegen::LLVMCodegen;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <file.cor> [--llvm]", args[0]);
        std::process::exit(1);
    }
    
    let filename = &args[1];
    let generate_llvm = args.len() > 2 && args[2] == "--llvm";
    
    println!("=== Coral LLVM Demo: {} ===\n", filename);
     let content = std::fs::read_to_string(filename)
        .map_err(|e| format!("Failed to read file {}: {}", filename, e))?;

    if generate_llvm {
        println!("\n=== Generating LLVM IR ===\n");
        
        // Parse the program for LLVM generation
        let mut lexer = Lexer::new(content.clone(), filename.clone());
        let tokens = lexer.tokenize()
            .map_err(|e| format!("Lexer error: {:?}", e))?;
        
        let mut parser = Parser::new(tokens, filename.clone());
        let program = parser.parse()
            .map_err(|e| format!("Parser error: {:?}", e))?;
        
        println!("✅ Successfully parsed {} statements", program.statements.len());
        
        let mut codegen = LLVMCodegen::new(filename.clone());
        match codegen.compile_program(&program) {
            Ok(ir) => {
                println!("{}", ir);
                
                // Write to file
                let output_file = format!("{}.ll", filename.trim_end_matches(".cor"));
                std::fs::write(&output_file, &ir)?;
                println!("\n✅ LLVM IR written to {}", output_file);
            }
            Err(e) => {
                eprintln!("❌ LLVM codegen failed: {:?}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Run full parse and semantic analysis
        match parse_and_analyze(&content) {
            Ok((_, analysis_result)) => {
                match analysis_result {
                    Ok(()) => println!("✅ Semantic analysis passed"),
                    Err(errors) => {
                        println!("⚠️ Semantic analysis found {} errors:", errors.len());
                        for error in errors {
                            println!("   - {}", error.message);
                        }
                    }
                }
            }
            Err(e) => {
                println!("❌ Failed to parse: {}", e);
            }
        }
        
        println!("\n💡 Use --llvm flag to generate LLVM IR");
    }
    
    Ok(())
}
