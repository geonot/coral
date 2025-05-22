# Advanced Topics

This chapter delves into several advanced topics crucial for building robust, well-organized, and practical applications in Coral. We'll explore Coral's error handling philosophy in more detail, how code is structured into modules and projects, and conceptual ideas about how Coral might interoperate with other programming languages. These features are designed to contribute to Coral's goals of being a modern, efficient, and clean language where the compiler and runtime manage significant complexity.

## 1. Error Handling in Depth

Coral adopts an explicit, value-based error handling mechanism that is consistent across all function and method calls, promoting clarity and predictability.

### Recap: The `(result, error_details)` Tuple

As established, every function and method in Coral returns a 2-tuple: `(result, error_details)`.

*   `result`: The actual result of the function's successful operation.
*   `error_details`: A 2-tuple `(error_id, error_description_string)`.
    *   `error_id`: Integer; `0` for success, non-zero for error.
    *   `error_description_string`: String with error details, empty for success.

### Error Philosophy

Coral's explicit, value-based error handling offers several benefits:

*   **Clarity and Explicitness:** Error paths are part of the function's explicit contract. No hidden exceptions disrupt control flow.
*   **Predictable Control Flow:** Error checking uses standard conditional statements (`if error_details.0 neq 0:`).
*   **Functional Harmony:** Errors are treated as a type of output value, fitting well with FP principles.
*   **Discourages Errors for Control Flow:** Reduces the temptation to use exceptions for non-error situations.
*   **Consistency:** The same mechanism applies everywhere.

While this system requires explicit error checks, this tradeoff is made for utmost clarity and predictability in understanding program behavior. Future Coral development might introduce syntax sugar or compiler analyses to streamline common error handling patterns without sacrificing explicitness.

### Standard Error IDs (Conceptual)

Coral's standard library would likely define common `error_id` values (e.g., `0` Success, `2` Not Found, `4` Invalid Argument).

### Custom Error IDs

Applications should define their own `error_id` ranges for domain-specific errors.

```coral
// Example custom error IDs
USER_SERVICE_BASE_ERROR_ID is 2000
ERROR_USER_NOT_FOUND is USER_SERVICE_BASE_ERROR_ID + 1      // 2001
ERROR_INVALID_USER_DATA is USER_SERVICE_BASE_ERROR_ID + 2   // 2002
```

### Propagating Errors

Functions often propagate errors from calls they make.

```coral
def fetch_raw_data(data_id):
    // ...
    if data_source_unavailable: // Example condition
        return (null, (503, "Data source is currently unavailable"))
    // fetched_data = ...
    return (fetched_data, (0, ""))

def parse_data(raw_data_str):
    // ...
    if parsing_fails: // Example condition
        return (null, (601, "Invalid data format encountered"))
    // parsed_data_obj = ...
    return (parsed_data_obj, (0, ""))

def process_data(data_id):
    (raw_data, fetch_err) is fetch_raw_data(data_id)
    if fetch_err.0 neq 0:
        return (null, fetch_err) // Propagate the exact error

    (processed_data, parse_err) is parse_data(raw_data)
    if parse_err.0 neq 0:
        wrapped_message is 'Data processing failed: parsing error - {parse_err.1}'
        PROCESSING_PARSE_ERROR_ID is 701
        return (null, (PROCESSING_PARSE_ERROR_ID, wrapped_message)) // Return a new, contextual error

    return (processed_data, (0, ""))

// Calling process_data
(final_data, process_err) is process_data("id_123")
if process_err.0 neq 0:
    print('Processing error ID {process_err.0}: {process_err.1}')
else:
    print('Successfully processed data.') // Assuming final_data is used
```

### The `result` Field When an Error Occurs

When `error_details.0` is non-zero, the `result` part of the tuple is often `null` or a specific "no value" marker. Callers should generally not rely on `result` if an error is indicated.

### Error Handling in Actors

Asynchronous calls to actor methods return a `Future`. This `Future` resolves to the standard `(result, error_details)` tuple. Error handling remains consistent: check the future's resolution, then check the `error_id` in the resolved tuple.

```coral
// my_actor is a reference to an actor instance
future_response is my_actor.some_actor_method(args)

// ... later, possibly in an async function ...
(resolved_tuple, future_sys_err) is await future_response // Conceptual await

if future_sys_err.0 eq 0: // Check if the Future itself resolved correctly
    (actual_result, method_err) is resolved_tuple // Unpack the tuple from the actor method
    if method_err.0 neq 0:
        print('Actor method failed: ID={method_err.0}, Desc="{method_err.1}"')
    else:
        print('Actor method succeeded. Result: {actual_result}')
else:
    // System-level error during future resolution or actor communication
    print('System error resolving future: {future_sys_err.1}')
```

## 2. Modules and Code Organization

Coral code is organized into modules for complexity management, reusability, and namespace control. The Coral compiler efficiently manages these modules.

### Concept of Modules

*   **Files as Modules:** Each Coral source file (`.cr`) typically acts as a module. Its name is often derived from the filename (e.g., `my_utils.cr` is the `my_utils` module).
*   **Encapsulation:** Modules encapsulate their code. Definitions within a module are local unless explicitly imported by another module.

### Importing Code

Coral uses an `import` statement, similar to Python.

```coral
// File: string_utils.cr
// --- content of string_utils.cr ---
DEFAULT_MAX_LEN is 100 // Constant

def to_uppercase(input_str):
    // type_of is a hypothetical runtime type check returning (type_string, error_tuple)
    (str_type, type_err) is type_of(input_str)
    if type_err.0 neq 0 or str_type neq "String":
         return (null, (1, "Input must be a string"))
    uppercased_str is input_str + " (TRANSFORMED TO UPPERCASE)" // Simplified transformation
    return (uppercased_str, (0, ""))

def _internal_utility(): // Internal use by convention
    return (true, (0,""))
// --- end of string_utils.cr ---


// File: main.cr
// --- content of main.cr ---
import string_utils // Imports the entire string_utils module

// Alternative import styles (conceptual):
// from string_utils import to_uppercase
// from string_utils import DEFAULT_MAX_LEN as MAX_STRING_LENGTH

def main_program():
    (upper_val, err) is string_utils.to_uppercase("hello coral")
    if err.0 eq 0:
        print(upper_val)
    else:
        print('Error: {err.1}')

    print('Default max length from string_utils: {string_utils.DEFAULT_MAX_LEN}')
    return (true, (0,""))

(run_res, run_err) is main_program()
// --- end of main.cr ---
```

### Module Scope and Visibility

*   **Public by Default (Conventionally):** Top-level definitions in a module are generally accessible when imported.
*   **Internal Use Convention (`_`):** Names prefixed with an underscore (e.g., `_internal_var`, `def _helper():`) are conventionally for internal module use.

### Project Structure (Conceptual)

Larger Coral projects might be organized into directories (packages), potentially with a manifest file (`project_manifest.cor` or `coral.toml`) for metadata, dependencies, and build configurations. The Coral toolchain would handle project compilation and dependency management.

## 3. Interoperability (Conceptual)

Coral might offer interoperability with other languages for practical integration.

### Foreign Function Interface (FFI)

An FFI could allow calling functions from native libraries (e.g., C, C++, Rust) for performance or system access. The Coral compiler would play a crucial role in managing the complexities of data type marshalling (conversion) and mapping Coral's `(result, error)` tuple convention to native error reporting.

```coral
// Conceptual FFI declaration for a C function:
// extern "C" def native_fast_calculation(input_a: Int, input_b: Int) -> Int
// This declares the native function's signature as understood by Coral.

// Calling the FFI-declared function:
// (result_val, ffi_err) is native_fast_calculation(123, 456)
// // 'ffi_err' would capture errors from the FFI call itself or from how the
// // native function's errors (e.g., return codes) are translated by the FFI layer.
// if ffi_err.0 eq 0:
//     print('Native calculation result: {result_val}')
// else:
//     print('FFI call error: {ffi_err.1}')
```

### Python Interoperability

Given Coral's Pythonic inspirations, interoperability with Python could be a long-term goal, though complex due to runtime differences.

### Benefits of Interoperability

*   **Performance:** Use native code for critical sections.
*   **Code Reuse:** Leverage existing libraries.
*   **System Integration:** Interface with OS APIs.

This chapter provided a deeper look into error handling, code organization, and potential interoperability in Coral, highlighting features contributing to robust and practical applications, with the Coral system designed to manage underlying complexities.
