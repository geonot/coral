use coral::*;
use std::env;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        // Parse a specific file
        let filename = &args[1];
        println!("=== Parsing {} ===\n", filename);
        
        let content = fs::read_to_string(filename)
            .map_err(|e| format!("Failed to read file {}: {}", filename, e))?;
        
        match parse_and_analyze(&content) {
            Ok((program, analysis_result)) => {
                println!("✅ Successfully parsed {} statements", program.statements.len());
                
                for (i, stmt) in program.statements.iter().enumerate() {
                    println!("  {}. {:?}", i + 1, stmt.kind);
                }
                
                match analysis_result {
                    Ok(()) => println!("\n✅ Semantic analysis passed"),
                    Err(errors) => {
                        println!("\n⚠️ Semantic analysis found {} errors:", errors.len());
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
        
        return Ok(());
    }
    
    // Default: run built-in tests
    println!("=== Enhanced Coral Language Parser with Semantic Analysis ===\n");
    
    // Test cases including multi-line code and Coral-specific syntax
    let test_cases = vec![
        ("Simple variable", "x is 42"),
        ("Coral assignment with 'is'", "message is 'hello coral'"),
        ("Special literal 'no'", "flag is no"),
        ("Special literal 'yes'", "flag is yes"),
        ("Special literal 'empty'", "collection is empty"),
        ("Special literal 'now'", "timestamp is now"),
        ("Arithmetic", "y is 1 + 2 * 3"),
        ("Function call", "result is add(10, 20)"),
        ("Logical operators", "cmp is true && false || true"),
        ("Boolean literal", "flag is true"),
        ("String literal", "name is \"hello\""),
        ("Nested arithmetic", "complex is (1 + 2) * (3 + 4)"),
        
        // New feature tests - unless/until statements
        ("Unless statement", r#"
            unless flag:
                log 'flag is false'
        "#),
        ("Until loop", r#"
            counter is 0
            until counter == 5:
                counter is counter + 1
        "#),
        
        // New feature tests - iterate with $ symbol
        ("Iterate with $ symbol", r#"
            items is [1, 2, 3]
            iterate items:
                log $
        "#),
        ("Iterate string with $ symbol", r#"
            text is \"hello\"
            iterate text:
                char_value is $
        "#),
        
        // New feature tests - ternary operator
        ("Ternary operator", "result is true ? 42 ! 0"),
        ("Nested ternary", "complex is x > 0 ? y > 0 ? 1 ! -1 ! 0"),
        
        // New feature tests - () list/map literals
        ("Empty list with ()", "empty_list is ()"),
        ("List literal with ()", "numbers is (1, 2, 3, 4)"),
        ("Map literal with ()", "config is (name: 'coral', version: 1)"),
        ("Mixed types in list", "mixed is (42, 'hello', true)"),
        
        // New feature tests - $ identifier in expressions
        ("$ in ternary inside iterate", r#"
            items is [1, 2, 3]
            iterate items:
                processed is $ > 2 ? $ * 2 ! $ + 1
        "#),
        
        // New feature tests - hex and binary integers
        ("Hex integer literal", "hex_value is 0xFF"),
        ("Binary integer literal", "bin_value is b1010"),
        ("Hex in expression", "result is 0x10 + 0x20"),
        
        // Existing tests
        ("Multi-line simple", r#"
            a is 10
            b is 20
        "#),
        ("Multi-line with function", r#"
            fn add(x: i32, y: i32) -> i32:
                return x + y
            result is add(5, 3)
        "#),
        ("Function with default parameter", r#"
            fn greet(name, greeting 'hello') -> string:
                return greeting
        "#),
        ("Object with fields and methods", r#"
            object task:
                definition: string
                processed ? no
                
                complete:
                    processed is yes
        "#),
        ("If expression", r#"
            value is if true:
                42
            else:
                0
        "#),
        ("Block expression with function", r#"
            fn get_complex() -> i32:
                temp is 10
                return temp * 2
            complex is get_complex()
        "#),
        ("Eq conditional block", r#"
            x is 42
            y is 42
            x eq y
                log 'same' !
                log 'different'
        "#),
    ];
    
    let mut passed = 0;
    let mut total = 0;
    
    for (name, code) in test_cases {
        total += 1;
        print!("Testing {}: ", name);
        
        match parse_and_analyze(code) {
            Ok((program, analysis_result)) => {
                println!("✅ PASS - {} statements parsed", program.statements.len());
                if let Err(errors) = analysis_result {
                    println!("   ⚠️  Semantic errors: {}", errors.len());
                    for error in errors.iter().take(3) {
                        println!("      - {}", error.message);
                    }
                } else {
                    println!("   ✓ Semantic analysis passed");
                }
                passed += 1;
            }
            Err(e) => {
                println!("❌ FAIL - {}", e);
            }
        }
    }
    
    println!("\n=== Summary ===");
    println!("Passed: {}/{} ({:.1}%)", passed, total, (passed as f64 / total as f64) * 100.0);
    
    // Performance test with large multi-line code
    println!("\n=== Performance Test ===");
    let large_code = (0..50).map(|i| format!("var{} is {}", i, i * 2)).collect::<Vec<_>>().join("\n");
    let start = std::time::Instant::now();
    
    match parse_and_analyze(&large_code) {
        Ok((program, _)) => {
            let duration = start.elapsed();
            println!("✅ Parsed {} statements in {:?}", program.statements.len(), duration);
        }
        Err(e) => {
            println!("❌ Performance test failed: {}", e);
        }
    }
    
    // Type checking demonstration
    println!("\n=== Type Checking Demo ===");
    let type_error_code = r#"
        x is "hello"
        y is x + true
    "#;
    
    print!("Type error detection: ");
    match parse_and_analyze(type_error_code) {
        Ok((_, analysis_result)) => {
            if let Err(errors) = analysis_result {
                println!("✅ PASS - {} type errors detected", errors.len());
                for error in &errors {
                    println!("   - {}", error.message);
                }
            } else {
                println!("❌ FAIL - No type errors detected");
            }
        }
        Err(parse_error) => {
            // If a parse error occurs, it means the code couldn't even be parsed
            // to reach semantic analysis. This is still a failure for this test.
            println!("❌ FAIL - Parse error: {}", parse_error);
        }
    }
    
    Ok(())
}
