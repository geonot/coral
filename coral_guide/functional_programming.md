# Functional Programming in Coral

Coral is a multi-paradigm language that, alongside its strong Object-Oriented features, deeply embraces the principles of Functional Programming (FP). This allows developers to leverage FP techniques for writing clearer, more predictable, and often more concise code. The Coral compiler is designed to optimize many functional constructs, ensuring that expressive code can also be efficient. All functions in Coral, including those used in a functional style, adhere to the standard return convention: `(result, (error_id, error_description_string))`.

## 1. First-Class Functions

In Coral, functions are "first-class citizens." This means they are treated like any other value:

*   Functions can be assigned to variables.
*   Functions can be passed as arguments to other functions.
*   Functions can be returned as results from other functions.

This flexibility is fundamental to many FP patterns.

```coral
// A regular named function
def say_hello(name):
    print('Hello, {name}') // 'print' is a built-in for simple output
    return (true, (0, "")) // Standard success return for action-oriented functions

// Assigning a function to a variable
greeter_func is say_hello
(call_res, call_err) is greeter_func('Coral FP User') // Call via the new variable
if call_err.0 neq 0: print('Error: {call_err.1}')


// A function that takes another function as an argument (a Higher-Order Function)
// 'func_ref' is expected to take one argument and return (result, error_tuple)
def apply_operation(func_ref, value):
    (op_res, op_err) is func_ref(value) // Call the passed-in function
    if op_err.0 neq 0:
        return (null, op_err) // Propagate error if the operation failed
    return (op_res, (0, ""))  // Return the successful result

def double_value(x):
    return (x * 2, (0, ""))

def triple_value(x):
    return (x * 3, (0, ""))

(double_res, double_err) is apply_operation(double_value, 5)
if double_err.0 eq 0:
    print('Applying double to 5: {double_res}')

(triple_res, triple_err) is apply_operation(triple_value, 5)
if triple_err.0 eq 0:
    print('Applying triple to 5: {triple_res}')


// A function that returns a function (demonstrating closures)
def get_multiplier(factor):
    // This inner function 'multiplier_func' is a closure.
    // It captures 'factor' from its enclosing scope.
    def multiplier_func(n):
        return (n * factor, (0, ""))
    return (multiplier_func, (0, "")) // Return the closure itself

(multiplier_fn_val, multiplier_fn_err) is get_multiplier(4)
if multiplier_fn_err.0 eq 0:
    times_four is multiplier_fn_val // Assign the returned function to a variable
    (product, prod_err) is times_four(6)
    if prod_err.0 eq 0:
        print('Multiplying 6 by 4: {product}')
else:
    print('Error getting multiplier: {multiplier_fn_err.1}')
```

## 2. Immutability

Immutability—the principle that data should not change after it's created—is a cornerstone of FP. It helps prevent side effects, makes state management simpler, and improves the predictability and testability of code.

Coral supports and encourages immutability:

*   **Constants:** Using `ALL_CAPS_NAME is value` for constants conventionally signals that the binding, and often the data it refers to, should not change.

    ```coral
    PI is 3.14159
    APP_NAME is "Coral Explorer"
    ```

*   **Programming Style:** A functional style in Coral encourages creating new data structures with modified values rather than altering existing ones in place. This is particularly important for collections.

    ```coral
    // More functional: creating a new list
    OLD_LIST is [1, 2, 3] // Conceptually, treat this as immutable
    // Concatenating to form a new list (assumes '+' for list concatenation creates a new list)
    NEW_LIST is OLD_LIST + [4]
    print('Old list: {OLD_LIST}, New list: {NEW_LIST}')
    // OLD_LIST remains [1, 2, 3]
    ```

*   **Immutable Data Structures (Conceptual):** For robust FP, Coral's standard library would ideally provide a suite of efficient immutable collection types (e.g., immutable lists, maps, sets). Operations on these collections would always return new collections, leaving the originals unchanged. The compiler would play a key role in optimizing operations on such structures.

Adhering to immutability where possible leads to code that is easier to reason about, debug, and test, especially in concurrent scenarios.

## 3. Pure Functions

A pure function has two main properties:

1.  **Deterministic:** Its return value is always the same for the same set of input arguments.
2.  **No Side Effects:** It does not cause any observable changes outside its own scope (e.g., no modifying external variables, no I/O operations like printing to the console or writing to a file).

In Coral, a pure function will always return the same `result` for the same inputs. Its `error_details` tuple would ideally be `(0,"")` unless an input violates a precondition (which can be seen as an input-driven error rather than a side effect of the function's internal logic).

**Benefits:**
*   **Testability:** Easy to test as you only need to check input against output.
*   **Predictability:** Their behavior is consistent and reliable.
*   **Memoization:** Results can be cached (memoized) for given inputs.
*   **Concurrency:** Intrinsically safe to run in parallel.

**Examples:**

```coral
// Pure function:
def pure_add(a, b):
    return (a + b, (0, "")) // Always returns the same sum for the same a, b; no side effects.

(sum1_val, sum1_err) is pure_add(5, 3)
if sum1_err.0 eq 0: print('5 + 3 = {sum1_val}')

(sum2_val, sum2_err) is pure_add(5, 3) // Will produce the same result and error status
if sum2_err.0 eq 0: print('5 + 3 = {sum2_val}')


// Impure function due to I/O (side effect):
def impure_add_and_log(a, b):
    sum_val is a + b
    print('Adding {a} and {b}. Result is {sum_val}.') // Side effect: printing to console
    return (sum_val, (0, ""))

(impure_res, _) is impure_add_and_log(5, 3)
```
While not all functions can be pure (e.g., functions performing I/O), striving to use pure functions for data transformation and core logic significantly improves code quality and maintainability.

## 4. Higher-Order Functions (HOFs)

Higher-Order Functions (HOFs) operate on other functions, either by taking them as arguments or by returning them. HOFs are a powerful tool for abstraction and creating reusable, composable code. Coral supports HOFs naturally due to its first-class functions.

Common HOFs like `map`, `filter`, and `reduce` are often provided for collections. Functions passed to these HOFs must also adhere to the `(result, error_details)` return convention. The HOFs themselves are responsible for handling these tuples, typically propagating errors by stopping on the first encountered error.

For list building in these examples, we use `new_list is old_list + [item]`, representing an immutable-style construction. Coral's standard library might offer more optimized immutable collection builders where the compiler can further enhance performance.

```coral
// map_list: Applies 'transform_func' to each element of 'collection'.
// Returns a new list of results or propagates the first error from 'transform_func'.
def map_list(collection, transform_func):
    results is []
    iter collection:
        (transformed_item, err) is transform_func(it) // 'it' is the current item
        if err.0 neq 0:
            return (null, err) // Stop and propagate the first error
        results is results + [transformed_item] // Appends to create a new list conceptually
    return (results, (0, ""))

// filter_list: Returns a new list of elements from 'collection' for which
// 'predicate_func' returns (true, (0,"")). Propagates first error.
def filter_list(collection, predicate_func):
    results is []
    iter collection:
        (passes, err) is predicate_func(it)
        if err.0 neq 0:
            return (null, err) // Stop and propagate error from predicate
        if passes is true:    // Ensure the predicate's result was explicitly true
            results is results + [it]
    return (results, (0, ""))

// reduce_list: Combines elements of 'collection' into a single value using 'combine_func'.
// Propagates the first error from 'combine_func'.
def reduce_list(collection, initial_value, combine_func):
    accumulator is initial_value
    iter collection:
        (new_accumulator, err) is combine_func(accumulator, it)
        if err.0 neq 0:
            return (null, err) // Stop and propagate error
        accumulator is new_accumulator
    return (accumulator, (0, ""))

// Example usage:
NUMBERS is [1, 2, 3, 4, 5]

def square(n):
    return (n * n, (0, ""))

(squared_numbers, map_err) is map_list(NUMBERS, square)
if map_err.0 eq 0: print('Squared: {squared_numbers}') else: print('Map error: {map_err.1}')

def is_even(n):
    return (n % 2 eq 0, (0, ""))

(even_numbers, filter_err) is filter_list(NUMBERS, is_even)
if filter_err.0 eq 0: print('Evens: {even_numbers}') else: print('Filter error: {filter_err.1}')

def sum_combiner(acc, n):
    return (acc + n, (0, ""))

(sum_of_numbers, reduce_err) is reduce_list(NUMBERS, 0, sum_combiner)
if reduce_err.0 eq 0: print('Sum: {sum_of_numbers}') else: print('Reduce error: {reduce_err.1}')
```
The Coral compiler can potentially optimize chains of such HOF calls (e.g., through loop fusion or other techniques), making expressive functional code also performant.

## 5. Closures

As shown earlier, functions in Coral can be defined within other functions. These inner functions are **closures**: they "capture" or "close over" variables from their enclosing lexical scope. This means a closure remembers the environment in which it was created and can access variables from that environment even if the outer function has finished executing.

```coral
def make_adder(add_by): // The outer function
    // 'add_by' is a variable in the scope of 'make_adder'.
    // The inner function 'adder' captures 'add_by'.
    def adder(x):
        return (x + add_by, (0, "")) // 'adder' uses the captured 'add_by'
    return (adder, (0, "")) // 'make_adder' returns the closure 'adder'

(add_five_func_tuple, make_adder_err1) is make_adder(5)
if make_adder_err1.0 eq 0:
    add_five is add_five_func_tuple.0 // Extract the function from the tuple's result
    (result1, err1) is add_five(3)    // Call the closure
    if err1.0 eq 0: print('add_five(3) result: {result1}') // Output: 8
else:
    print('Error creating add_five: {make_adder_err1.1}')

(add_ten_func_tuple, make_adder_err2) is make_adder(10)
if make_adder_err2.0 eq 0:
    add_ten is add_ten_func_tuple.0
    (result2, err2) is add_ten(7)
    if err2.0 eq 0: print('add_ten(7) result: {result2}') // Output: 17
else:
    print('Error creating add_ten: {make_adder_err2.1}')
```
Closures are powerful for creating specialized functions on the fly and for patterns like callbacks or encapsulating state in a functional style.

## 6. Pattern Matching (Conceptual)

Pattern matching is a powerful feature found in many modern languages that allows for sophisticated control flow based on the structure and values of data. While the exact syntax in Coral is speculative, its inclusion would greatly enhance functional programming capabilities and overall expressiveness.

Pattern matching typically involves:
*   **Matching** a value against a series of patterns.
*   **Deconstructing** data types (like objects, tuples, or lists) into their constituent parts if a pattern matches.
*   **Guards:** Allowing conditional logic within patterns.

**Hypothetical Coral Syntax:**

```coral
// This section remains highly conceptual.
// 'describe_value_internal' is a hypothetical function that would be the core of the pattern match.
// The 'match' keyword here is purely illustrative of the concept.
def describe_value_internal(value_payload):
    // The 'match' construct would evaluate 'value_payload' against cases.
    // Each case returns a string directly for this example.
    match value_payload: // This 'match' is illustrative, not defined Coral syntax.
        0 => "It's zero."
        1 or 2 => "It's one or two." // 'or' for multiple values in a pattern
        "hello" => "A greeting text."
        true => "Boolean true."
        [] => "An empty list." // Matching an empty list
        [first] => 'A list with one element: {first}.' // Destructuring a single-element list
        [first, second] => 'A list with two elements: {first} and {second}.'
        // Point(x: 0, y: 0) => "The origin point." // Conceptual object destructuring
        // Point(x: val_x, y: val_y) if val_x eq val_y => 'Point on diagonal: x=y={val_x}.' // Guard
        _ => "Some other value." // Default catch-all case
    // Assume 'match' returns the value from the chosen arm. Let this be 'description_string'.
    // return description_string // This is what describe_value_internal returns.
}

// Wrapper function to adhere to Coral's tuple return convention for external callers.
def get_description(payload):
    // In a real implementation, 'describe_value_internal' or the 'match' block itself
    // might be able to produce an error tuple directly.
    // For this conceptual example, we assume it returns the string or raises an internal error
    // if the match fails unexpectedly (which '_' should prevent).
    description is describe_value_internal(payload)
    return (description, (0, ""))


(desc_res1, desc_err1) is get_description(0)
if desc_err1.0 eq 0: print('{desc_res1}')

(desc_res2, desc_err2) is get_description([10, 20])
if desc_err2.0 eq 0: print('{desc_res2}')
```
Pattern matching can make complex conditional logic more readable and less error-prone than deeply nested if-else statements, especially when dealing with algebraic data types or varied object structures. Its integration with Coral's standard error handling would be a key design consideration.

By combining these functional programming features, Coral developers can write expressive, robust, and maintainable code, often leading to solutions that are both elegant and efficient, thanks to the underlying optimizations performed by the Coral compiler.The `coral_guide/functional_programming.md` file is already in excellent shape and largely adheres to the new syntax rules. I've made a few refinements to further improve clarity and consistency, and to emphasize Coral's design goals.

**Specific Checks:**

*   **HOF List Building:** The current text uses `results is results + [transformed_item]`. This is good for demonstrating an immutable approach. I will add a sentence to mention that Coral's standard library might offer optimized immutable collection types and builders for efficiency, where the compiler plays a role.
*   **Pattern Matching `describe_value_wrapper`:** The note about how `match` would integrate with tuple returns is good. I'll slightly rephrase the example to make the conceptual nature of `describe_value_wrapper` directly returning a string (for the example's simplicity) even clearer, and then how a "real" function `get_description` would wrap that in a tuple. The name `describe_value_wrapper` was a bit confusing, changed it to `describe_value_internal` for the hypothetical core logic.
*   **Emphasis on "compiler handles complexity":** Functional constructs, especially when combined with type inference and potential optimizations (like loop fusion for chained HOFs), are a good place to mention this. I've added notes on this.
*   **Tuple extraction in `make_adder`:** Clarified extracting the function from the tuple result before calling it.

The refined version has been applied via `overwrite_file_with_block`.

Now, I'll proceed with **`coral_guide/other_influences.md`**.
I will check:
-   `is` for assignment.
-   `ALL_CAPS` for constants.
-   Tuple returns for all conceptual functions/methods, e.g., `(instance, error) is ClassName(...)`, `(result, error) is M1.multiply(M2)`.
-   Correct HOF usage in the relational section (helper functions returning tuples, HOFs handling them).
-   Updated class syntax in the Lisp-like `@Observable` example.
-   General syntax consistency.
-   Emphasis on Coral's design goals where appropriate.
