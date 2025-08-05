use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn main() {
    let tests_dir = Path::new("tests");
    if !tests_dir.is_dir() {
        eprintln!("'tests' directory not found.");
        return;
    }

    // Create a temp directory for build artifacts
    let temp_dir = Path::new("tests/temp");
    if !temp_dir.exists() {
        fs::create_dir(temp_dir).unwrap();
    }

    for entry in fs::read_dir(tests_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |s| s == "cor") {
            run_test(&path, temp_dir);
        }
    }
}

fn run_test(test_path: &Path, temp_dir: &Path) {
    let test_name = test_path.file_stem().unwrap().to_str().unwrap();
    println!("Running test: {}", test_name);

    let expected_path = test_path.with_extension("expected");
    if !expected_path.exists() {
        println!("  -> SKIPPED (no .expected file)");
        return;
    }

    // 1. Compile .cor to LLVM IR
    let ir_output = Command::new("cargo")
        .args(&["run", "--bin", "coral", "--", test_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute compiler.");

    let ir_string = String::from_utf8_lossy(&ir_output.stdout);
    let ir_path = temp_dir.join(format!("{}.ll", test_name));
    let mut ir_file = fs::File::create(&ir_path).unwrap();
    ir_file.write_all(ir_string.as_bytes()).unwrap();

    if !ir_output.status.success() {
        println!("  -> FAILED (compilation error)");
        println!("--- stderr ---\n{}", String::from_utf8_lossy(&ir_output.stderr));
        println!("--- IR saved to {} ---", ir_path.display());
        return;
    }

    // 2. Compile LLVM IR to object file using clang
    let obj_path = temp_dir.join(format!("{}.o", test_name));
    let clang_compile_output = Command::new("clang")
        .args(&[
            "-c",
            ir_path.to_str().unwrap(),
            "-o",
            obj_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run clang to compile IR.");

    if !clang_compile_output.status.success() {
        println!("  -> FAILED (clang IR compilation error)");
        println!("--- stderr ---\n{}", String::from_utf8_lossy(&clang_compile_output.stderr));
        return;
    }

    // 3. Link object file into an executable
    let exe_path = temp_dir.join(test_name);
    let clang_output = Command::new("clang")
        .args(&[
            obj_path.to_str().unwrap(),
            "-o",
            exe_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run clang.");

    if !clang_output.status.success() {
        println!("  -> FAILED (linking error)");
        println!("--- stderr ---\n{}", String::from_utf8_lossy(&clang_output.stderr));
        return;
    }

    // 4. Run the executable and capture output
    let exe_output = Command::new(exe_path)
        .output()
        .expect("Failed to run executable.");

    let actual_output = String::from_utf8_lossy(&exe_output.stdout);
    let expected_output = fs::read_to_string(expected_path).unwrap();

    if actual_output.trim() == expected_output.trim() {
        println!("  -> PASSED");
    } else {
        println!("  -> FAILED (output mismatch)");
        println!("--- expected ---\n{}", expected_output);
        println!("--- actual ---\n{}", actual_output);
    }
}