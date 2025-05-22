# Functional Programming in Coral

Coral is a multi-paradigm language that, alongside its strong Object-Oriented features, deeply embraces the principles of Functional Programming (FP). This allows developers to leverage FP techniques for writing clearer, more predictable, and often more concise code. Functional constructs can also significantly simplify the development of concurrent and parallel applications.

This chapter explores how Coral supports key functional programming concepts.

## 1. First-Class Functions

In Coral, functions are "first-class citizens." This means they are treated like any other value (e.g., integers, strings, or objects). Specifically:

*   **Functions can be assigned to variables:** You can store a function in a variable and then use that variable to call the function.
*   **Functions can be passed as arguments to other functions:** This is fundamental to higher-order functions, enabling powerful abstractions.
*   **Functions can be returned as results from other functions:** This allows for the creation of functions that generate other functions (closures).

```coral
// A regular named function
fn say_hello(name: String) {
    print(f"Hello, {name}!");
}

// Assigning a function to a variable
let greeter_func = say_hello;
greeter_func("Coral FP User"); // Output: Hello, Coral FP User!

// A function that takes another function as an argument
// (fn(Int)->Int is a type hint for a function taking an Int and returning an Int)
fn apply_operation(func: fn(Integer)->Integer, value: Integer) -> Integer {
    return func(value);
}

let double = |x: Integer| x * 2; // An anonymous function (lambda)
let triple = |x: Integer| x * 3;

print(f"Applying double to 5: {apply_operation(double, 5)}");   // Output: Applying double to 5: 10
print(f"Applying triple to 5: {apply_operation(triple, 5)}");   // Output: Applying triple to 5: 15

// A function that returns a function
fn get_multiplier(factor: Integer) -> fn(Integer)->Integer {
    return |n: Integer| n * factor;
}

let times_four = get_multiplier(4);
print(f"Multiplying 6 by 4: {times_four(6)}"); // Output: Multiplying 6 by 4: 24
```

## 2. Immutability

Immutability, the concept that data should not change after it's created, is a cornerstone of functional programming. It helps prevent side effects, makes state management simpler, and improves predictability.

Coral supports and encourages immutability in several ways:

*   **Constants:** Using `const` to declare bindings ensures that once a value is assigned, the binding cannot be changed.

    ```coral
    const PI = 3.14159;
    const APP_NAME = "Coral Explorer";
    // PI = 3.14; // This would result in an error
    ```

*   **Programming Style:** While Coral's `let` bindings might allow reassignment, and some object attributes might be mutable for practical OOP integration, a functional style in Coral encourages creating new data structures with modified values rather than altering existing ones in place. This is especially true when dealing with collections or complex data.

    ```coral
    // Less functional: modifying a list (if lists are mutable by default)
    // let my_list = [1, 2, 3];
    // my_list.append(4); // Modifies my_list

    // More functional: creating a new list
    let old_list = [1, 2, 3]; // Assume this is an immutable list or treated as such
    let new_list = old_list + [4]; // Creates a new list [1, 2, 3, 4]
                                   // (Assuming '+' for list concatenation or a dedicated function)
    ```

*   **Immutable Data Structures (Conceptual):** For robust FP, Coral's standard library would ideally provide a suite of immutable collection types (e.g., immutable lists, maps, sets). Operations on these collections would always return new collections, leaving the originals unchanged. While specific immutable collections are not detailed here, their presence would be a natural fit for Coral's FP philosophy.

Adhering to immutability where possible leads to code that is easier to reason about, debug, and test, especially in concurrent scenarios.

## 3. Pure Functions

A pure function is a function that adheres to two main properties:

1.  **Deterministic:** Its return value is always the same for the same set of input arguments.
2.  **No Side Effects:** It does not cause any observable changes outside of its own scope. This means it doesn't modify external variables, perform I/O operations (like printing to the console or writing to a file), or change the state of any mutable objects passed to it.

**Benefits of Pure Functions:**

*   **Testability:** Easy to test as you only need to check input against output.
*   **Predictability:** Their behavior is consistent and reliable.
*   **Memoization:** Results can be cached (memoized) for given inputs, as the output will never change.
*   **Concurrency:** Safe to run in parallel without causing race conditions related to shared state.

**Examples:**

```coral
// Pure function:
// - Always returns the same output for the same inputs.
// - Has no side effects.
fn pure_sum(a: Integer, b: Integer) -> Integer {
    return a + b;
}

print(pure_sum(5, 3)); // Output: 8
print(pure_sum(5, 3)); // Output: 8 (always the same)

// Impure function:
// - Modifies an external variable (side effect).
let call_count = 0; // External state

fn impure_add_and_count(a: Integer, b: Integer) -> Integer {
    call_count += 1; // Side effect: modifies 'call_count'
    print(f"This function has been called {call_count} times."); // Side effect: I/O
    return a + b;
}

print(impure_add_and_count(5, 3));
// Output:
// This function has been called 1 times.
// 8

print(impure_add_and_count(5, 3));
// Output:
// This function has been called 2 times.
// 8
```
While not all functions can be pure (e.g., functions that perform I/O), striving to use pure functions for data transformation and business logic can significantly improve code quality.

## 4. Higher-Order Functions (HOFs)

Higher-Order Functions are functions that operate on other functions, either by taking them as arguments or by returning them (or both). HOFs are a powerful tool for abstraction and creating reusable code.

Coral supports HOFs naturally due to its first-class functions. Common HOFs, often found in standard libraries or as methods on collection types, include `map`, `filter`, and `reduce`.

Let's imagine Coral collections (like a hypothetical `List` type) have these methods:

```coral
let numbers: List<Integer> = [1, 2, 3, 4, 5]; // Assuming a generic List type

// map: Applies a function to each element, returning a new list of results.
let squared_numbers = numbers.map(|n: Integer| n * n);
print(f"Squared: {squared_numbers}"); // Output: Squared: [1, 4, 9, 16, 25] (conceptual list print)

// filter: Returns a new list containing only elements for which the predicate function returns true.
let even_numbers = numbers.filter(|n: Integer| n % 2 == 0);
print(f"Evens: {even_numbers}");    // Output: Evens: [2, 4]

// reduce (or fold): Combines elements of a list into a single value using an accumulator and a combining function.
// Syntax: reduce(initial_value, |accumulator, element| -> new_accumulator)
let sum_of_numbers = numbers.reduce(0, |acc: Integer, n: Integer| acc + n);
print(f"Sum: {sum_of_numbers}");      // Output: Sum: 15

// Example: Find the product of all numbers
let product = numbers.reduce(1, |acc, n| acc * n);
print(f"Product: {product}"); // Output: Product: 120
```
If these weren't direct methods, one could define standalone HOFs:
```coral
fn map_list<T, U>(list: List<T>, func: fn(T)->U) -> List<U> {
    let result_list: List<U> = []; // Start with an empty list
    for item in list {
        result_list.add(func(item)); // Conceptual list add
    }
    return result_list;
}
```

## 5. Closures

As seen in the "Language Basics" and "First-Class Functions" sections, Coral supports anonymous functions (lambdas). These anonymous functions are also **closures**.

A closure is a function that can "capture" or "close over" variables from its enclosing lexical scope. This means it remembers the environment in which it was created and can access variables from that environment even if the outer function has finished executing.

```coral
fn make_adder(add_by: Integer) -> fn(Integer)->Integer {
    // 'add_by' is part of the environment of the anonymous function below.
    // The anonymous function "captures" 'add_by'.
    let closure_fn = |x: Integer| -> Integer {
        return x + add_by;
    };
    return closure_fn;
}

let add_five = make_adder(5);   // 'add_by' is 5 for this closure
let add_ten = make_adder(10);  // 'add_by' is 10 for this closure

// When add_five is called, it remembers 'add_by' was 5.
print(f"add_five(3) = {add_five(3)}");     // Output: add_five(3) = 8

// When add_ten is called, it remembers 'add_by' was 10.
print(f"add_ten(7) = {add_ten(7)}");     // Output: add_ten(7) = 17

// Another example: creating a counter
fn make_counter(start_val: Integer) -> fn()->Integer {
    let current_count = start_val; // 'current_count' is captured
                                  // Note: This implies 'current_count' itself needs to be mutable
                                  // within the closure's captured state, which is an advanced topic.
                                  // For simplicity, let's assume Coral closures can capture mutable state
                                  // carefully, or this example might need a more complex setup with refs/cells.
                                  // For now, we focus on capturing the value.

    // For a truly stateful closure that modifies captured vars, Coral might require
    // explicit 'mut' capture or use objects.
    // Let's simplify and assume it captures the *value* 'add_by' which is immutable.
    // The counter example is more complex if the closure *mutates* the captured var.
    // The 'make_adder' example is cleaner as 'add_by' is read-only within the closure.
}
```
Closures are powerful for creating specialized functions on the fly and for patterns like callbacks or state encapsulation in a functional style.

## 6. Pattern Matching (Conceptual)

Pattern matching is a powerful feature found in many functional (and multi-paradigm) languages like Rust, Scala, F#, and Swift. It allows for sophisticated control flow based on the structure and values of data. While the exact syntax in Coral is speculative, its inclusion would greatly enhance functional programming capabilities.

Pattern matching involves:

*   **Matching** a value against a series of patterns.
*   **Deconstructing** data types (like objects, tuples, or lists) into their constituent parts if a pattern matches.
*   **Guards:** Allowing conditional logic within patterns.

**Hypothetical Coral Syntax:**

```coral
// Assume 'Any' is a base type or generic type for demonstration
// Assume Point is a class: class Point { x: Int; y: Int; ... }
// Assume List syntax like [element1, element2]

fn describe_value(value: Any) -> String {
    match value {
        0 => "It's zero.",
        1 | 2 => "It's one or two.", // Multiple values in one pattern
        "hello" => "A greeting text.",
        true => "Boolean true.",
        // Deconstructing a hypothetical List
        [] => "An empty list.",
        [first] => f"A list with one element: {first}.",
        [first, second] => f"A list with two elements: {first} and {second}.",
        [head, ...tail] => f"A list starting with {head} and followed by {tail.length} other elements.", // Rest pattern

        // Deconstructing a Point object (assuming constructor-like pattern)
        Point(0, 0) => "The origin point.",
        Point(x, y) if x == y => f"A point on the diagonal: x=y={x}.", // Pattern with a guard
        Point(x, y) => f"A point at coordinates ({x}, {y}).",

        // Default case (catch-all)
        _ => "Some other value."
    }
}

print(describe_value(0));                  // Output: It's zero.
print(describe_value("hello"));            // Output: A greeting text.
print(describe_value(Point(3, 3)));        // Output: A point on the diagonal: x=y=3.
print(describe_value(Point(1, 5)));        // Output: A point at coordinates (1, 5).
print(describe_value(["apple", "banana"])); // Output: A list with two elements: apple and banana.
print(describe_value([10, 20, 30]));       // Output: A list starting with 10 and followed by 2 other elements.
print(describe_value(false));              // Output: Some other value. (If no specific bool false case)
```
Pattern matching can make complex conditional logic more readable and less error-prone than deeply nested if-else statements, especially when dealing with algebraic data types or varied object structures.

By combining these functional programming features, Coral developers can write expressive, robust, and maintainable code, choosing the best paradigm for each specific task.
