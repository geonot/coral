# Language Basics

This chapter delves into the fundamental building blocks of the Coral language. Coral's syntax is designed for clarity and developer productivity, drawing inspiration from Python's readability while incorporating unique features for robustness and expressiveness. The Coral compiler handles significant underlying complexity, allowing for clean and concise code.

## 1. General Syntax

Coral's structure is defined by **colons (`:`) and indentation**. Code blocks, such as those in control flow statements (if, loops) and definitions (functions, classes), are initiated by a colon and then indented.

*   **Statement Termination:** Statements are typically one per line. Semicolons (`;`) are **not** used to terminate statements.
*   **No Curly Braces:** Curly braces (`{}`) are **not** used to delimit blocks; indentation serves this purpose.
*   **Case Sensitivity:** Coral is case-sensitive. `my_variable` and `MY_VARIABLE` are distinct.

```coral
// Example of a simple structure
PI is 3.14159         // A constant assignment
message is "Hello, Coral" // A variable assignment

if PI gt 3:             // A conditional block starts with a colon
    print(message)      // Indentation defines this block
    print('PI is greater than 3') // Further statements in the block
```

## 2. Comments

Comments are essential for explaining code and are ignored by the Coral interpreter.

*   **Single-line comments:** Start with `//`. Everything from `//` to the end of the line is a comment.

    ```coral
    // This is a single-line comment.
    x is 10 // This comment explains the variable assignment.
    ```

*   **Multi-line comments:** Enclosed between `/*` and `*/`.

    ```coral
    /*
      This is a multi-line comment.
      It can span across several lines and is useful
      for more detailed explanations or for temporarily
      disabling blocks of code.
    */
    y is 20
    ```

## 3. Variables and Constants

Variables store data that can change, while constants store data that, once set, should not be altered. The distinction is conventional for constants (`ALL_CAPS`) to guide developers.

### Declaration and Assignment

Coral uses the `is` keyword for all assignments.

*   **Variables:** Declared and assigned using `is`. Variable names are conventionally written in `snake_case`.

    ```coral
    user_name is "CoralUser"
    age is 1
    current_score is 0.0
    ```

*   **Constants:** Declared and assigned using `is`, with the name conventionally in `ALL_UPPERCASE` to signal intent.

    ```coral
    PI is 3.14159
    SITE_NAME is "Coral Lang Official"
    DEFAULT_TIMEOUT is 5000
    ```
    Reassigning a variable is straightforward:
    ```coral
    count is 5
    print(count) // The 'print' function is a built-in for easy output.
    count is 10
    print(count)
    ```

### Type System

Coral is a **strongly typed** language, ensuring type safety. However, it features **complete type inference**. This means you do **not** explicitly declare data types for variables, constants, or function parameters. The Coral compiler intelligently deduces the type from the value assigned or used. This reduces boilerplate and enhances readability, letting the compiler manage type details.

```coral
answer is 42        // 'answer' is inferred as Integer by the compiler.
message is "Welcome"  // 'message' is inferred as a literal String.
is_ready is true    // 'is_ready' is inferred as Boolean.
```
Operations between incompatible types will result in errors, caught by the compiler.

### Naming Conventions

*   **Variables:** `snake_case` (e.g., `current_user`, `items_in_cart`).
*   **Constants:** `ALL_UPPERCASE` (e.g., `MAX_USERS`, `API_KEY`).

## 4. Basic Data Types

Coral provides fundamental data types for various kinds of information.

### Integers

Integers represent whole numbers.

```coral
count is 10
negative_value is -5
// Underscores can be used as digit separators for readability in long numbers.
large_number is 1_000_000
```

### Floating-Point Numbers

Floating-point numbers represent numbers with a decimal point or in scientific notation.

```coral
price is 29.99
temperature is -5.5
PLANCK_CONSTANT is 6.626e-34
```

### Booleans

Booleans represent truth values: `true` or `false`.

```coral
is_active is true
has_permission is false
is_greater is 10 gt 5 // Evaluates to true
```

### Strings

Strings represent sequences of characters. Coral distinguishes between literal strings and strings that support interpolation for embedding values.

*   **Literal Strings (Double Quotes):** Enclosed in double quotes (`"`). These are literal: any characters within them, including curly braces, are treated as part of the string itself. Use these when you don't need to embed variable values.

    ```coral
    greeting_literal is "Hello, Coral!"
    path_example is "C:/Users/Default/My Documents/{project_folder}/" // Here, {project_folder} is literal.
    empty_string is ""
    ```

*   **Interpolated Strings (Single Quotes):** Enclosed in single quotes (`'`). These strings allow embedding expressions (variables, constants, or other calculations) directly within them using curly braces `{}`.

    ```coral
    user_name is "Alice"
    score is 100
    welcome_message is 'Welcome, {user_name}! Your score is {score}.'
    print(welcome_message) // Output: Welcome, Alice! Your score is 100.

    ITEM_COUNT is 3
    TOTAL_COST is 99.50
    summary is 'You have {ITEM_COUNT} items. Total: ${TOTAL_COST}' // The $ is literal here.
    print(summary) // Output: You have 3 items. Total: $99.50
    ```

*   **Concatenation:** Literal strings can be concatenated using the `+` operator. For combining with interpolated values or for more complex constructions, using a single interpolated string is often cleaner.

    ```coral
    first_part is "Coral"
    second_part is " Lang"
    combined_literal is first_part + second_part // Results in "Coral Lang"

    version_number is "1.0"
    full_title is 'Coral Language v{version_number}' // Preferred for mixing variables
    ```

*   **Length:** String length can typically be accessed using a `.length` property (this is a common convention; the exact mechanism is part of the String type's definition in the standard library).

    ```coral
    message is "Hello"
    message_length is message.length // message_length would be 5
    print('The message "{message}" has {message_length} characters.')
    ```

## 5. Operators

Operators are special symbols or keywords that perform operations on values (operands).

### Arithmetic Operators

Standard arithmetic operations:

*   `+` (Addition)
*   `-` (Subtraction)
*   `*` (Multiplication)
*   `/` (Division - typically results in a float if either operand is float, or if the division is not exact)
*   `%` (Modulo/Remainder)

```coral
a is 10
b is 3
print('a + b = {a + b}')     // Output: a + b = 13
print('10 / 4 = {10 / 4}')   // Output: 10 / 4 = 2.5
print('a % b = {a % b}')     // Output: a % b = 1
```

### Comparison Operators

Used to compare two values, returning a Boolean (`true` or `false`).

*   `eq` (Equal to)
*   `neq` (Not equal to)
*   `lt` (Less than)
*   `gt` (Greater than)
*   `lte` (Less than or equal to)
*   `gte` (Greater than or equal to)

```coral
x is 5
y is 10
print('x eq y: {x eq y}')    // Output: x eq y: false
print('x neq y: {x neq y}')  // Output: x neq y: true
print('x lt y: {x lt y}')    // Output: x lt y: true
```

### Logical Operators

Used to combine or invert Boolean expressions.

*   `and` (Logical AND: `true` if both operands are `true`)
*   `or` (Logical OR: `true` if at least one operand is `true`)
*   `not` (Logical NOT: inverts the Boolean value)
*   `xor` (Logical XOR: `true` if operands are different)

```coral
is_logged_in is true
has_admin_rights is false
print('Logged in AND Admin: {is_logged_in and has_admin_rights}') // Output: false
print('NOT Admin: {not has_admin_rights}')                        // Output: true
```

### Bitwise Operators

For low-level manipulation of bits in integers.

*   `lsh` (Left Shift: `x lsh y` shifts bits of `x` left by `y` positions)
*   `rsh` (Right Shift: `x rsh y` shifts bits of `x` right by `y` positions)
    *(Note: Other bitwise operators like bitwise AND, OR, XOR would typically exist, e.g., `band`, `bor`, `bxor`, `bnot`.)*

```coral
value is 4          // Binary 0100
left_shifted is value lsh 1 // Binary 1000 (Decimal 8)
print('4 lsh 1 is {left_shifted}')
```

### Assignment Operator

The primary assignment operator is `is`. There are **no** compound assignment operators (like `+=`, `-=`). Updates are done by re-assigning, which maintains clarity.

```coral
num is 10
num is num + 5 // num is now 15
print(num)

text is "Hello"
text is text + ", Coral!" // text is now "Hello, Coral!"
print(text)
```

## 6. Control Flow

Control flow statements dictate the order of execution. Coral uses colons and indentation.

### If/Else Statements

Execute code blocks based on conditions.

```coral
temperature is 25
if temperature gt 30:
    print('It is very hot!')
elif temperature gt 20:  // 'elif' for 'else if'
    print('It is warm.')
else:
    print('It is cool.')
```

### Loops

Loops execute a block of code repeatedly. The primary loop construct in Coral is `iter`.

*   **`iter collection_or_range:`**
    The `iter` keyword iterates over a sequence. Common sequences include ranges (e.g., `0..5`) or lists. The current item in each iteration is implicitly available via the keyword `it`.

    ```coral
    // Iterating over a range: 0 up to (but not including) 5
    print('Numbers in range 0..5:')
    iter 0..5:
        print('Number: {it}') // Prints 0, 1, 2, 3, 4

    // Iterating over a list (list syntax is `[item1, item2, ...]`)
    NAMES is ["Alice", "Bob", "Charlie"]
    print('Greeting names:')
    iter NAMES:
        print('Hello, {it}')
    ```
    **Note on "while" loops:** Coral prioritizes `iter` for most looping constructs. Traditional `while condition:` loops can be expressed using `iter` with conditional `break` statements or by iterating over a sequence generated based on a condition. This unified approach simplifies the language's syntax.

### Loop Control Statements

*   **`break`:** Immediately exits the current `iter` loop.

    ```coral
    iter 0..100:
        if it eq 3:
            print('Breaking at 3.')
            break
        print(it) // Prints 0, 1, 2
    ```

*   **`continue`:** Skips the rest of the current iteration and proceeds to the next one.

    ```coral
    iter 0..5:
        if it eq 2:
            print('(Skipping 2)')
            continue
        print(it) // Prints 0, 1, (Skipping 2), 3, 4
    ```

## 7. Functions

Functions are reusable blocks of code that perform specific tasks.

### Definition

Functions are defined using the `def` keyword, followed by the function name, parameters in parentheses (without type annotations), and a colon. The function body is indented. Types are inferred by the compiler.

```coral
def greet(name):
    message is 'Hello, {name}!'
    print(message)
    // No explicit return, defaults to (true, (0, ""))

def add(a, b):
    sum_val is a + b
    return (sum_val, (0, "")) // Explicit success return
```

### Return Values and Error Handling

A cornerstone of Coral's robustness and clarity is its **universal tuple return convention**. All functions and methods, whether built-in or user-defined, implicitly or explicitly return a 2-tuple: `(result, error_details)`.

*   `result`: The actual result of the function's operation. Its type is inferred.
*   `error_details`: This is itself a 2-tuple: `(error_id, error_description_string)`.
    *   `error_id`: An integer. `0` signifies success. Any non-zero value indicates an error.
    *   `error_description_string`: A string providing more information about the error. For success (when `error_id` is `0`), this is an empty string `""`.

This consistent approach means error handling is always explicit and predictable, a key aspect of Coral's design where the language structure itself promotes robust code.

**Common Return Patterns:**

1.  **Successful operation with a value:**
    ```coral
    def calculate_area(width, height):
        if width lte 0 or height lte 0:
            // Return 'null' (or a specific "no value" marker) for the result part on error
            return (null, (101, "Width and height must be positive."))
        area is width * height
        return (area, (0, "")) // Result is 'area', error_id is 0
    ```

2.  **Operation with no specific result value (e.g., performing an action):**
    If a function completes without an explicit `return` statement, it automatically returns `(true, (0, ""))` to indicate successful completion of its action.

    ```coral
    def log_message(message_text):
        print('LOG: {message_text}')
        // Implicitly returns (true, (0, ""))
    ```

3.  **Operation fails:**
    Return a specific `error_id` (non-zero) and a descriptive `error_description_string`. The `result` part of the tuple should usually be `null` or a sensible default if `null` is not appropriate for the function's potential return type.

    ```coral
    def find_user_data(user_id):
        // ... logic to find user ...
        if user_not_found: // some condition representing failure
            return (null, (404, 'User not found with ID: {user_id}'))
        // ...
        // user_data_object = ...
        return (user_data_object, (0, ""))
    ```

### Calling Functions and Handling Returns

When calling a function, you destructure the returned tuple to access the result and error details. This makes error checking an explicit step.

```coral
(area_val, area_error_info) is calculate_area(10, 5)

if area_error_info.0 neq 0: // Check the error_id
    print('Error calculating area: ID={area_error_info.0}, Message="{area_error_info.1}"')
else:
    print('Calculated Area is: {area_val}')


(user_data, user_find_err) is find_user_data(123)
if user_find_err.0 eq 0:
    // Assuming user_data is an object or map with a 'name' key/attribute
    print('Found user data for: {user_data.name}')
else:
    print('Failed to find user: {user_find_err.1}')

// For functions that return (true, (0,"")) on implicit success:
(op_status, log_op_err) is log_message("System initialized.")
if log_op_err.0 neq 0:
    print('Logging failed: {log_op_err.1}')
// 'op_status' would be true if logging was successful.
```

### Anonymous Functions (Lambdas/Closures) - Brief Note

While full `def` statements provide maximum clarity for function definitions, especially with Coral's tuple return system, the language might conceptually support a very concise syntax for simple, single-expression anonymous functions (lambdas) for specific use cases like arguments to some higher-order functions. Any such syntax would still adhere to the `(result, error_details)` return principle. For this guide, `def` is used for all function definitions to maintain clarity.

This covers the core language basics for Coral. These rules aim to provide a consistent, expressive, and robust foundation, where the compiler handles much of the complexity, allowing developers to write clean and efficient code. Always refer to the official Coral documentation for the most complete and up-to-date specifications.
