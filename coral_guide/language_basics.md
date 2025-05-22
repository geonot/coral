# Language Basics - Part 1

This section delves into the fundamental building blocks of the Coral language, including its syntax, how to use comments, declare variables and constants, and the basic data types you'll be working with.

## General Syntax

Coral's syntax is meticulously designed to be both readable and easy to write, drawing significant inspiration from Python's clarity.

*   **Readability:** Like Python, Coral can use indentation to define code blocks (e.g., in control structures, function definitions). This promotes clean and visually organized code. However, for single statements or simple blocks, explicit markers like curly braces `{}` might also be supported for flexibility, similar to languages like C# or Swift.
*   **Case Sensitivity:** Coral is a case-sensitive language. This means `myVariable` and `myvariable` would be treated as two distinct variables.
*   **Semicolons:** Semicolons (`;`) at the end of statements are generally optional in Coral, much like in Python. The interpreter can infer statement endings based on newlines. However, they can be used to separate multiple statements on a single line if desired.

    ```coral
    let message = "Hello" // Semicolon is optional
    print(message); // Semicolon can be used
    ```

## Comments

Comments are crucial for explaining code. Coral supports both single-line and multi-line comments.

*   **Single-line comments:** Start with `//`. Anything from `//` to the end of the line is ignored by the interpreter.

    ```coral
    // This is a single-line comment.
    let x = 10; // This comment explains the variable assignment.
    ```

*   **Multi-line comments:** Enclosed between `/*` and `*/`. This allows for comments that span multiple lines.

    ```coral
    /*
      This is a multi-line comment.
      It can span across several lines and is useful
      for more detailed explanations or temporarily disabling
      blocks of code.
    */
    let y = 20;
    ```

## Variables and Constants

Variables are used to store data that can change, while constants store data that, once set, cannot be altered.

### Declaration

*   **Variables:** Declared using the `let` keyword.

    ```coral
    let name = "Coral";
    let age = 1;
    ```

*   **Constants:** Declared using the `const` keyword. Constants must be initialized at the time of declaration.

    ```coral
    const PI = 3.14159;
    const SITE_NAME = "Coral Lang";
    ```

### Mutability

*   `let`: Bindings introduced by `let` are re-assignable by default, offering flexibility similar to Python variables.

    ```coral
    let count = 5;
    print(count); // Output: 5
    count = 10;   // 'count' can be reassigned
    print(count); // Output: 10
    ```
    (Note: While `let` allows reassignment, for true Rust-like mutable *data*, Coral might introduce a separate keyword like `mut` or specific mutable types if we want to go deeper into that paradigm, but for now, `let` allows rebinding).

*   `const`: Bindings introduced by `const` are immutable. Once a value is assigned to a constant, it cannot be changed. This helps ensure that certain values remain fixed throughout your program, contributing to predictability and safety.

    ```coral
    const GREETING = "Hello";
    // GREETING = "Hi"; // This would cause an error
    ```

### Type System

Coral is a **strongly typed** language, meaning that every variable and constant has a specific type, and operations between incompatible types are disallowed, catching many potential errors at compile-time or before runtime.

*   **Type Inference:** Coral features powerful type inference. In many cases, you don't need to explicitly state the type of a variable; the compiler can deduce it from the initial value.

    ```coral
    let answer = 42;        // 'answer' is inferred as Integer
    let message = "Welcome";  // 'message' is inferred as String
    let is_ready = true;    // 'is_ready' is inferred as Boolean
    ```

*   **Optional Type Annotations:** For clarity, or when the type cannot be easily inferred (e.g., function parameters, uninitialized variables that will be assigned later), you can provide explicit type annotations using a colon (`:`). This is a feature that brings in Rust-like safety and explicitness where desired.

    ```coral
    let version: String = "1.0";
    let user_id: Integer;
    // user_id = "some_id"; // This would cause a type error
    user_id = 12345;      // Correct
    ```

### Naming Conventions

To maintain consistency and readability, Coral suggests the following naming conventions:

*   **Variables:** Use `snake_case` (all lowercase, with words separated by underscores).
    ```coral
    let current_user = "admin";
    let items_in_cart = 5;
    ```
*   **Constants:** Use `UPPER_CASE` (all uppercase, with words separated by underscores).
    ```coral
    const MAX_USERS = 100;
    const DEFAULT_TIMEOUT = 5000;
    ```

## Basic Data Types

Coral provides a set of fundamental data types to work with various kinds of information.

### Integers

Integers represent whole numbers.

```coral
let count = 10;
let negative_value = -5;
let items_on_page = 25;

// For readability with large numbers, you can use underscores as digit separators.
let large_number = 1_000_000;
let another_large_one = 1_234_567_890;
```

### Floating-Point Numbers

Floating-point numbers represent numbers with a decimal point, or numbers expressed in scientific notation.

```coral
let price = 29.99;
let temperature = -5.5;
let pi_approx = 3.1415926535;
let planck_constant = 6.626e-34; // Scientific notation
```

### Booleans

Booleans represent truth values: `true` or `false`. They are fundamental for conditional logic and comparisons.

```coral
let is_active = true;
let has_permission = false;
let is_greater = 10 > 5; // Evaluates to true
```

### Strings

Strings represent sequences of characters, used for text.

*   **Declaration:** Strings can be created using double quotes (`"`).

    ```coral
    let greeting = "Hello, Coral!";
    let empty_string = "";
    let user_name = "Alice";
    ```

*   **Concatenation:** Strings can be joined together (concatenated) using the `+` operator.

    ```coral
    let first_name = "Coral";
    let last_name = "Lang";
    let full_name = first_name + " " + last_name; // "Coral Lang"
    print(full_name);
    ```

*   **Length:** You can often get the length of a string using a property or method (syntax might vary, common examples include `.length` or `len()`). Let's assume a `.length` property for now.

    ```coral
    let message = "Hello";
    print(message.length); // Output: 5
    ```

*   **String Interpolation:** Coral supports string interpolation for easily embedding expressions within string literals, typically using an `f` prefix before the string (inspired by Python f-strings) or a `${}` syntax (inspired by JavaScript/Kotlin). Let's use `f""` and `{}`.

    ```coral
    let name = "User";
    let score = 100;
    let welcome_message = f"Hello, {name}! Your score is {score}.";
    print(welcome_message); // Output: Hello, User! Your score is 100.

    let item_count = 3;
    let total_cost = 99.50;
    let summary = f"You have {item_count} items. Total: ${total_cost}"; // Note: $ for currency, {} for interpolation
    print(summary); // Output: You have 3 items. Total: $99.5
    ```

This covers the first part of Coral's language basics. In the next part, we'll explore control flow, functions, and more complex data structures.

---

# Language Basics - Part 2

This part continues our exploration of Coral's fundamental features, focusing on operators, control flow mechanisms, and functions.

## 5. Operators

Operators are special symbols or keywords that perform operations on values (operands).

### Arithmetic Operators

These are used for performing mathematical calculations.

*   `+` (Addition)
*   `-` (Subtraction)
*   `*` (Multiplication)
*   `/` (Division)
    *   Coral aims for predictable division. If both operands are integers, it might perform integer division (truncating the result). If one or both are floating-point numbers, it performs floating-point division. This behavior should be clearly defined in the language specification. For now, let's assume standard behavior: `5 / 2` is `2.5`.
*   `%` (Modulo/Remainder)

```coral
let a = 10;
let b = 3;
let c = 2.5;

print(f"a + b = {a + b}");     // Output: a + b = 13
print(f"a - b = {a - b}");     // Output: a - b = 7
print(f"a * b = {a * b}");     // Output: a * b = 30
print(f"a / b = {a / b}");     // Output: a / b = 3 (Assuming integer division for now for simplicity, or 3.333... if float)
print(f"10 / 4 = {10 / 4}");   // Output: 2.5 (If one is float or default is float division)
print(f"a % b = {a % b}");     // Output: a % b = 1
print(f"c * 2 = {c * 2}");     // Output: c * 2 = 5.0
```

### Comparison Operators

Used to compare two values. They return a Boolean (`true` or `false`).

*   `==` (Equal to)
*   `!=` (Not equal to)
*   `<` (Less than)
*   `>` (Greater than)
*   `<=` (Less than or equal to)
*   `>=` (Greater than or equal to)

```coral
let x = 5;
let y = 10;

print(f"x == y: {x == y}");   // Output: x == y: false
print(f"x != y: {x != y}");   // Output: x != y: true
print(f"x < y: {x < y}");    // Output: x < y: true
print(f"x >= 5: {x >= 5}");  // Output: x >= 5: true
```

### Logical Operators

Used to combine or invert Boolean expressions.

*   `&&` (Logical AND): Returns `true` if both operands are `true`.
*   `||` (Logical OR): Returns `true` if at least one operand is `true`.
*   `!` (Logical NOT): Inverts the Boolean value of its operand.

```coral
let is_logged_in = true;
let has_admin_rights = false;

print(f"Logged in AND Admin: {is_logged_in && has_admin_rights}"); // Output: false
print(f"Logged in OR Admin: {is_logged_in || has_admin_rights}");  // Output: true
print(f"NOT Admin: {!has_admin_rights}");                        // Output: true
```

### Assignment Operators

Used to assign values to variables.

*   `=` (Simple assignment)
*   `+=` (Add and assign)
*   `-=` (Subtract and assign)
*   `*=` (Multiply and assign)
*   `/=` (Divide and assign)

```coral
let num = 10;
num += 5; // num is now 15 (equivalent to num = num + 5)
print(num);

let text = "Hello";
text += ", Coral!"; // text is now "Hello, Coral!"
print(text);

num *= 2; // num is now 30
print(num);
```

## 6. Control Flow

Control flow statements allow you to dictate the order in which statements are executed.

### If/Else Statements

Used to execute different blocks of code based on conditions. Coral follows a Python-like indentation for blocks.

```coral
let temperature = 25;

if temperature > 30:
    print("It's very hot!");
elif temperature > 20:  // 'elif' for 'else if'
    print("It's warm.");
else:
    print("It's cool.");

// Example with a single line
if temperature == 25: print("Exactly 25 degrees!");
```

### Loops

Loops are used to execute a block of code repeatedly.

#### For Loops

Ideal for iterating over a sequence (like a range, list, or string).

*   **Ranges:** Coral uses `start..end` for inclusive-start, exclusive-end ranges (like Python's `range(start, end)`), or `start...end` for inclusive-end ranges. Let's assume `x..y` means `x` up to `y-1`.

    ```coral
    // Iterate from 0 up to (but not including) 5
    for i in 0..5:
        print(f"Number: {i}"); // Prints 0, 1, 2, 3, 4

    // Iterate through characters in a string
    let greeting = "Hi";
    for char in greeting:
        print(char); // Prints H, i
    ```

*   **Collections:** You can iterate directly over elements in collections (which we'll cover in more detail later).

    ```coral
    // Assuming 'names' is a list or array-like structure
    let names = ["Alice", "Bob", "Charlie"]; // Fictional list syntax for now
    for name in names:
        print(f"Hello, {name}");
    ```

#### While Loops

Executes a block of code as long as a specified condition is `true`.

```coral
let count = 0;
while count < 3:
    print(f"While count is {count}");
    count += 1; // Important to modify the condition variable

// Output:
// While count is 0
// While count is 1
// While count is 2
```

### Loop Control Statements

*   **`break`:** Immediately exits the current loop.

    ```coral
    for i in 0..100:
        if i == 3:
            break; // Stop the loop when i is 3
        print(i); // Prints 0, 1, 2
    ```

*   **`continue`:** Skips the rest of the current iteration and proceeds to the next one.

    ```coral
    for i in 0..5:
        if i == 2:
            continue; // Skip printing when i is 2
        print(i); // Prints 0, 1, 3, 4
    ```

## 7. Functions

Functions are blocks of reusable code that perform a specific task.

### Definition

Functions are defined using the `fn` keyword. Parameters can have type annotations, and return types are specified using `->`.

```coral
// Function that takes a String parameter and doesn't explicitly return a value
fn greet(name: String) {
    print(f"Hello, {name}!");
}

// Function that takes two Integer parameters and returns an Integer
fn add(a: Integer, b: Integer) -> Integer {
    return a + b;
}

// Calling functions
greet("Coral Developer"); // Output: Hello, Coral Developer!

let sum_result = add(5, 7);
print(f"The sum is: {sum_result}"); // Output: The sum is: 12
```

### Return Values

*   The `return` keyword is used to send a value back from a function.
*   If a function doesn't have an explicit `return` statement, or has a `return` without a value, it implicitly returns a special "nothing" value (often called `nil`, `null`, or `void`-equivalent in other languages; Coral might have `Nil` or `Unit`).

```coral
fn get_greeting(name: String) -> String {
    if name == "":
        return "Hello, anonymous!"; // Explicit return
    // If name is not empty, this line is reached
    return f"Greetings, {name}!";
}

fn log_message(message: String) { // No explicit return type, implies returning 'Nil' or 'Unit'
    print(message);
    // No return keyword needed if nothing to return
}

let personalized_greeting = get_greeting("Evelyn");
print(personalized_greeting); // Output: Greetings, Evelyn!

log_message("System initialized."); // Prints "System initialized."
```

### Parameters

Currently, we're focusing on positional parameters with optional type annotations, as shown in the examples above. Coral might also support named parameters or default parameter values in the future for more Python-like flexibility.

### Anonymous Functions (Lambdas/Closures)

Coral supports concise anonymous functions, often called lambdas or closures, ideal for simple operations or passing behavior as data. The syntax is `|parameters| -> return_type { body }` or `|parameters| expression_body` for a single expression.

```coral
// Lambda that takes two integers and returns their product
let multiply = |x: Integer, y: Integer| -> Integer {
    return x * y;
};

// Shorter form if the body is a single expression (return type can often be inferred)
let square = |n: Integer| n * n;

let product = multiply(6, 7);
print(f"Product: {product}"); // Output: Product: 42

let num_squared = square(5);
print(f"5 squared: {num_squared}"); // Output: 5 squared: 25

// Lambdas are useful for functions that accept other functions as arguments
// (e.g., for list operations like map, filter - to be covered later)
// fn process_numbers(numbers: List<Integer>, operation: |Integer| -> Integer) -> List<Integer> { ... }
```

This concludes the second part of Coral's language basics. With these concepts, you have a solid foundation for writing more complex Coral programs. Next, we'll explore collections, error handling, and more advanced features.
