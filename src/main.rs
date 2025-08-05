use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: coral <file>");
        return;
    }

    let file_path = &args[1];
    let code = fs::read_to_string(file_path).expect("Failed to read file");

    let mut lexer = coral::lexer::Lexer::new(code, file_path.clone());
    let tokens = lexer.tokenize().unwrap();

    let mut parser = coral::parser::Parser::new(tokens, file_path.clone());
    let mut ast = parser.parse().unwrap();

    let mut resolver = coral::resolver::TypeResolver::new();
    resolver.resolve_program(&mut ast).unwrap();

    let mut codegen = coral::codegen::LLVMCodegen::new(file_path.clone());
    let llvm_ir = codegen.compile_program(&ast).unwrap();

    println!("{}", llvm_ir);
}

#[cfg(test)]
mod tests {
    use std::process::Command;
    use std::fs;

    #[test]
    fn test_full_features() {
        let test_name = "full_features";
        let coral_file = format!("tests/{}.cor", test_name);
        let expected_file = format!("tests/{}.expected", test_name);
        let ir_file = format!("tests/{}.ll", test_name);
        let obj_file = format!("tests/{}.o", test_name);
        let runtime_obj_file = "runtime/runtime.o";
        let executable_file = format!("tests/{}", test_name);

        // Compile Coral to LLVM IR
        let output = Command::new("cargo")
            .args(&["run", "--bin", "coral", "--", &coral_file])
            .output()
            .expect("Failed to compile Coral file");
        
        assert!(output.status.success(), "Coral compiler failed: {}", String::from_utf8_lossy(&output.stderr));
        fs::write(&ir_file, output.stdout).expect("Failed to write LLVM IR file");

        // Compile LLVM IR to object file
        let llc_output = Command::new("llc")
            .args(&["-filetype=obj", &ir_file, "-o", &obj_file])
            .output()
            .expect("Failed to run llc");
        
        assert!(llc_output.status.success(), "llc failed");

        // Compile C runtime to object file
        let cc_output = Command::new("cc")
            .args(&["-c", "runtime/runtime.c", "-o", runtime_obj_file])
            .output()
            .expect("Failed to compile runtime");

        assert!(cc_output.status.success(), "cc failed");

        // Link object files
        let link_output = Command::new("cc")
            .args(&[&obj_file, runtime_obj_file, "-o", &executable_file])
            .output()
            .expect("Failed to link object files");

        assert!(link_output.status.success(), "linker failed");

        // Run executable and check output
        let run_output = Command::new(format!("./{}", executable_file))
            .output()
            .expect("Failed to run executable");

        let expected_output = fs::read_to_string(expected_file).expect("Failed to read expected output file");
        assert_eq!(String::from_utf8_lossy(&run_output.stdout), expected_output);
    }
}