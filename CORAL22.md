Okay, this is a fantastic and quite unique design for a programming language! I'll structure this into a comprehensive syntax guide based on the information you've provided.

```markdown
# Coral Programming Language - Complete Syntax Guide

## Philosophy
Coral prioritizes **readability, simplicity, and automatic optimization**. The compiler handles complexity so you focus on solving problems.

---

## Table of Contents
1.  [Basic Syntax](#basic-syntax)
    *   [Variables and Constants](#variables-and-constants)
    *   [Comments](#comments-and-documentation)
2.  [Data Types](#data-types)
    *   [Literals](#literals)
    *   [Collections (Arrays/Lists)](#collections-arrayslists)
    *   [Objects and Maps](#objects-and-maps-literals)
3.  [Functions](#functions)
    *   [Function Definition](#function-definition)
    *   [Function Calls](#function-calls)
    *   [Parameter System](#parameter-system)
4.  [Objects](#objects-proper)
    *   [Object Definition](#object-definition)
    *   [Instantiation](#instantiation)
    *   [Method Definition](#method-definition)
    *   [Method Calls](#method-calls)
    *   [Method Chaining](#method-chaining)
5.  [Persistent Storage (`store`)](#persistent-storage-store)
    *   [Store Object Definition](#store-object-definition)
    *   [`make` Method](#make-method)
    *   [Polymorphic Methods](#polymorphic-methods)
6.  [Actor Model (`actor`)](#actor-model-actor)
    *   [Actor Definition](#actor-definition)
    *   [Join Table References (`&`)](#join-table-references-)
    *   [Message Handlers (`@`)](#message-handlers-)
    *   [Actor Communication](#actor-communication)
7.  [Control Flow](#control-flow)
    *   [Conditional Statements](#conditional-statements)
    *   [Loops](#loops)
    *   [Iteration (`across`)](#iteration-across)
8.  [Error Handling](#error-handling)
    *   [Error with Log and Return](#error-with-log-and-return)
    *   [Error with Default Value](#error-with-default-value)
    *   [Error on Instantiation/Operation](#error-on-instantiationoperation)
9.  [String Operations](#string-operations)
    *   [String Literals](#string-literals)
    *   [String Interpolation](#string-interpolation)
10. [Data Conversion (`as`)](#data-conversion-as)
11. [Built-in Functions and Operations](#built-in-functions-and-operations)
12. [Special Operators and Syntax](#special-operators-and-syntax)
13. [Module System (`use`)](#module-system-use)
14. [Advanced Features](#advanced-features)
    *   [Compile-time Evaluation](#compile-time-evaluation)
    *   [Automatic Parallelization](#automatic-parallelization)
    *   [Distributed Computing](#distributed-computing)
15. [Style Guidelines](#style-guidelines)
16. [Complete Example](#complete-example)

---

## 1. Basic Syntax

### Variables and Constants
Variables (and constants, distinction seems based on convention/immutability of assigned value) are declared and assigned using the `is` keyword. Types are inferred.

```coral
message is 'hello coral'
ITERATIONS is 100
threshold is 0.95
PI is 3.1415926535
```

### Comments and Documentation
-   Single-line comments start with `#`.
-   Documentation comments (for tools to generate documentation) start with `##`.

```coral
# This is a single-line comment
age is 30 # Assigning age

## Defines a user object for the system
object user
    name ## The user's full name
    email ## The user's email address
```

---

## 2. Data Types

### Literals
-   **Strings**: Enclosed in single quotes (`'...'`) for single-line or double quotes (`"..."`) for multi-line strings.
-   **Numbers**: Integers (`100`) and floating-point numbers (`0.95`).
-   **Booleans**: Implied `yes` and `no` (e.g., `processed ? no`). Explicit literals not shown but likely `yes`/`no` or `true`/`false`.
-   **Arrays/Lists**: `[item1, item2, ...]`
-   **Empty values**: `[]` (empty list), `empty` (for strings like `subject ? empty`).

### Collections (Arrays/Lists)
Ordered collections of items.

**Definition:**
```coral
primes is [2, 3, 5, 7, 11]
pending_tasks is [] # Empty list
```

**Access:**
Uses `at` keyword or `@` symbol (0-indexed).
```coral
first_prime is primes at 0
second_prime is primes@1
```

**Operations:**
Dual syntax: keyword-based or method-style.
```coral
# Adding elements
push 'todo 1' on pending_tasks
pending_tasks.push 'todo 2'

# Other common operations (implied by .pop in example)
# item is pending_tasks.pop
# count is pending_tasks.length (or .size, .count - specific keyword not shown)
# is_empty is pending_tasks.empty (shown in complete example)
```

### Objects and Maps (Literals)
Simple key-value structures can be defined directly. For more complex objects with methods, see [Objects (Proper)](#objects-proper).

```coral
net_config is
    host is 'localhost'
    port is 5000
    # Access: net_config.host, net_config.port
```
This defines `net_config` as an object/map with `host` and `port` properties.

---

## 3. Functions

### Function Definition
Functions are defined using the `fn` keyword, followed by the function name, the `with` keyword, and a comma-separated list of parameters. Default values for parameters can be specified directly after the parameter name. The last expression evaluated in a function is implicitly returned.

```coral
fn greet with name, greeting 'Hello'  # 'Hello' is default for greeting
    # String interpolation is used here
    '{greeting}, {name}. Welcome to Coral.'

fn compute_total with price, quantity, tax_rate 0.07
    sub_total is price * quantity
    # The result of this expression is returned
    sub_total + (sub_total * tax_rate)
```

### Function Calls
Function calls are made by writing the function name followed by space-separated arguments. Parentheses are not used for standard calls.

**Positional Arguments:**
```coral
greet 'Expert'                    # Uses default greeting 'Hello'
greet 'Dr. Coral', 'Salutations'  # Overrides default greeting
order_value is compute_total 100, 3 # Uses default tax_rate 0.07
```

**Named Arguments:**
Parameters can be passed by name, allowing any order and improving clarity for functions with many parameters.
```coral
order_custom_tax is compute_total
    price 100, quantity 3, tax_rate 0.1
```

### Parameter System
Inside functions (and methods, especially for `store` objects), parameters and object properties can be referenced using `$` notation.

-   `$0, $1, $2, ...`: Positional parameter references.
-   `$name`: Named parameter reference (e.g., `$description`, `$id`). This is also used to refer to the object's own properties within its methods, especially in `store` objects.

```coral
# In a store object method:
# id ? $id  -- $id refers to a passed parameter if 'id' is not set on instance
# log create $description -- $description refers to the object's description property
```

---

## 4. Objects (Proper)

### Object Definition
Complex objects with properties and methods are defined using the `object` keyword. Properties can have default values specified with `?`.

```coral
object datapoint
    value                 # Property with no default (must be provided on instantiation)
    processed ? no        # Property 'processed' defaults to 'no' (boolean-like)
    timestamp ? now       # Property 'timestamp' defaults to the result of 'now' function
```

### Instantiation
Objects are instantiated by calling the object's name as if it were a function, passing arguments for properties that don't have defaults or whose defaults you want to override.

```coral
d1 is datapoint 42          # 'value' is 42, 'processed' and 'timestamp' use defaults
d2 is datapoint 100, yes    # 'value' is 100, 'processed' is 'yes'
d3 is datapoint 100, yes, specific_time_value # All properties provided
```
**Assert Instantiation Success (`!`)**:
The `!` after an object name during instantiation likely means the operation must succeed, or the program will halt/error prominently (similar to a "bang" method in Ruby or an assertion).
```coral
# This instantiation must succeed, otherwise it might panic or throw an uncatchable error
brandon is user! 'brandon', 'brandon@email.com', 'password'
```

### Method Definition
Methods are functions defined within the scope of an object. They can access and modify the object's properties.

```coral
object datapoint
    value
    processed ? no
    timestamp ? now

    process  # Method definition
        processed is yes
        # 'self' or 'this' is implicit
```

### Method Calls
Methods are called using dot notation (`.`).
-   If a method name could also be an attribute name, `!` can be used to force the call.

```coral
d1 is datapoint 42
d1.process      # Calls the process method if 'process' is unambiguously a method
d1.process!     # Forces call if 'process' could also be an attribute

task.complete 2 # Method call with an argument
task1.complete  # Method call with no arguments
```

### Method Chaining
Methods can be chained using `then` or `and` keywords for enhanced readability. This suggests methods might return the object instance (`self`) by convention to allow chaining.

```coral
my_list.push item then .process and .save
# Equivalent to:
# my_list.push item
# my_list.process
# my_list.save
```
The choice between `then` and `and` might be purely stylistic or could imply subtle semantic differences (e.g., error handling or flow control), though the example implies stylistic.

---

## 5. Persistent Storage (`store`)
The `store` keyword is used to define objects that are automatically persisted to a data store. They share many characteristics with regular objects but have special features related to persistence.

### Store Object Definition
Similar to `object` definition, but uses the `store` keyword.
```coral
store task
    description
    priority ? 1        # Default value for priority
    complete ? false    # Default value for complete
```

### `make` Method
A special method named `make` can be defined within a `store` object. It acts like a constructor or an initializer that is called when a new instance of the `store` object is created and persisted. It often involves `log create` operations.
The `return` keyword is used explicitly here.

```coral
store task
    description
    priority ? 1
    complete ? false

    make  # Constructor-like method for persistent objects
        # $description, $priority, $complete refer to the instance properties
        return log create
            $description
            $priority
            $complete
```
**Instantiation of `store` objects:**
```coral
task1 is task 'Implement command-oriented persistence' # priority 1, complete false
task2 is task 'Write documentation', 2 # priority 2, complete false
```

### Polymorphic Methods
Methods in `store` objects can be designed to work both as instance methods (operating on a specific object instance) and class/static methods (operating on the object type or on an instance specified by ID). This is often achieved using the `property ? $parameter` pattern.

```coral
store task
    # ... other properties ...
    id # Assume 'id' is an automatically assigned property for stored objects

    complete
        id ? $id  # If instance.id is not set (or in a class context), use $id from parameter
                  # If instance.id is set (instance method context), it uses instance.id
        # 'id' here now refers to the resolved ID (either instance's or parameter's)
        return log update id # 'id' is used, not '$id' as it's now a local variable
            complete is true
```
**Usage:**
```coral
task1 is task 'Some task'
# ... task1 gets an id, e.g., 1 ...

task1.complete     # Instance method: uses task1's internal id.
                   # Here, 'id' inside 'complete' method resolves to task1.id.

task.complete 2    # Class/Static-like method: operates on task with id 2.
                   # Here, 'id' inside 'complete' method resolves to the parameter $id (which is 2).
```

---

## 6. Actor Model (`actor`)
Coral has built-in support for the actor model, defining concurrent entities that communicate via messages. Actors are often also `store` objects, meaning their state can be persisted.

### Actor Definition
Defined using `store actor` keywords.
```coral
store actor user
    name
    email
    password # Will be hashed in 'make'
    &blocklist    # Join table reference
    &messages     # Join table reference

    make
        password is hash.blake3 $password # Hash password on creation

    # ... other methods like send, authenticate ...
```

### Join Table References (`&`)
Properties prefixed with `&` (e.g., `&blocklist`) signify a relationship managed via a join table.
-   `&blocklist` in `user` implies a `user_blocklist` table, likely `user_blocklist(user_id, blocked_user_id)`.
-   `&messages` in `user` implies a `user_messages` table, likely `user_messages(user_id, message_id)`.

### Message Handlers (`@`)
Methods intended to handle incoming messages for an actor are annotated with `@`. A common one is `@receive`.
```coral
store actor user
    # ...
    @receive # This method is called when the actor receives a generic message
        # 'message' variable might be implicitly available or passed via $ conventions
        check_blocked log return # Example action
```
The complete example shows `@receive_task` for a custom message type.

```coral
store actor task_processor
    # ...
    @receive_task # Handles 'receive_task' messages
        task is task with $description, $priority # Creates a task from message parameters
        push task on pending_tasks
        process_next_task!
```

### Actor Communication
Actors communicate by sending messages. The `send` method is a common pattern, but any method can be invoked on another actor instance.
```coral
# Within a 'user' actor method:
send_message_to_another_user with recipient_id, message_content
    # Assuming 'user with $id' fetches an actor reference
    recipient_actor is user with recipient_id
    recipient_actor.receive message_content # Invokes @receive on recipient_actor
```
The example provided:
```coral
# Inside 'user' actor
send # Method name
    recipient is user with $id # $id refers to a parameter passed to 'send'
    recipient.receive $message # $message refers to a parameter passed to 'send'
```

---

## 7. Control Flow

### Conditional Statements
**Ternary-style Conditional Assignment:**
Uses `?` for the true-branch value and `!` for the false-branch value. Comparison operators like `gt` (greater than), `lt` (less than), `equals` are used.
```coral
status_text is
    system_status at load_average gt 0.9 ? 'High Load' ! 'Normal Load'

# General form:
# result is condition ? value_if_true ! value_if_false
```

**`unless` Conditionals:**
Executes code if a condition is false. Can be used as a prefix or postfix.
```coral
# Prefix
unless x equals 0
    process x

# Postfix
process x unless x equals 0
```
An `if` statement is implied but not explicitly shown in the basic examples. It would likely be `if x equals 0 ...`

### Loops
**`while` Loops:**
Executes a block of code as long as a condition is true.
```coral
iterator is 0
while iterator lt 3
    log 'iterator is {iterator}'
    iterator is iterator + 1 # Assuming increment syntax
```

**`until` Loops:**
Executes a block of code until a condition becomes true. Supports `from` (initial value) and `by` (step value) clauses.
```coral
# 'iterator' is implicitly declared and managed by the loop construct
until iterator from 0 by 2 is 8 # Loop while iterator < 8 (or <= 8, depends on exact semantic)
    log 'iterator is {iterator}'
# Output: iterator is 0, iterator is 2, iterator is 4, iterator is 6
# (Assuming 'is 8' means stop when iterator reaches 8)
```

### Iteration (`across`)
Used for iterating over collections, similar to `map` or `forEach` in other languages. The `$` symbol is often used as a placeholder for the current item.

**Basic Iteration (forEach style):**
```coral
# Calls check_health for each node in system_status.active_nodes
# $ is a placeholder for the current node
iterate system_status.active_nodes check_health $

# Alternative, more readable syntax
check_health across system_status.active_nodes
# ($ is implicit here, or check_health is expected to take one argument)
```

**Iteration with Result Collection (map style):**
The `into` keyword collects results.
```coral
# Calls check_health for each node and stores results in node_status
check_health across system_status.active_nodes into node_status
```

**Iteration with Additional Parameters:**
The `with` keyword can pass additional, fixed parameters to the function being called.
```coral
check_health across system_status.active_nodes with host 'localhost', timeout 5000 into node_status
# check_health would be called like: check_health(node, host 'localhost', timeout 5000)
```

---

## 8. Error Handling
Coral provides concise syntax for common error handling patterns.

### Error with Log and Return (`err log return`)
If an operation might fail, `err log return` can be appended. If an error occurs, it's logged, and the current function/method returns (possibly with an error value).
```coral
# If calculate_value fails, log the error and return from current function
calculate_value 10, 20 err log return

# If process_further fails with 'result', log and return
process_further result err log return
```

### Error with Default Value (`err default_value` or `as variable_name`)
If an operation can fail, provide a default value to use in case of error.
```coral
config is load 'coral.json' err {} # If load fails, config becomes an empty map/object

# Alternative syntax that might imply a default nil or error object if not specified
load 'coral.json' as config
# This is likely shorthand for:
# config is load 'coral.json' err some_default_error_or_nil_value
```

### Error on Instantiation/Operation (Pattern Matching Style)
This syntax seems to combine an operation (like fetching a user) with immediate error checking.
```coral
# Attempt to get a user with name 'root'
record is user with name 'root'
err return log err # If 'user with name 'root'' fails or returns an error state,
                   # return from the current function and log the error.
```
The `user!` syntax (see [Instantiation](#instantiation)) is a more forceful way of asserting success.

---

## 9. String Operations

### String Literals
-   Single-line strings: `'hello coral'`
-   Multi-line strings: Double quotes preserve formatting including newlines.
    ```coral
    description is "Multi-line strings
    can span multiple lines
    preserving formatting"
    ```

### String Interpolation
Variables can be embedded within strings (both single and double quoted, it seems) using curly braces `{}`.
```coral
greeting is 'Hello'
name is 'Coral User'
welcome_message is '{greeting}, {name}. Welcome to Coral.'
# welcome_message will be "Hello, Coral User. Welcome to Coral."

log 'iterator is {iterator}'
log 'task {id} (P{priority}) - {status}: {description}'
```

---

## 10. Data Conversion (`as`)
Objects (especially `store` objects) can define multiple representations of themselves using the `as` keyword followed by a type (e.g., `string`, `map`, `list`).

```coral
store message
    id # Assuming an ID field
    sender, recipient, subject, body
    timestamp ? now
    acknowledged ? no

    as string  # Define how a message object is converted to a string
        # Multi-line string definition for the representation
        'message:{id} from {sender} to {recipient}'
        'at {timestamp} (ack: {acknowledged})'
        
    as map     # Define how a message object is converted to a map/object literal
        'id' is id # 'id' on the right is the property of the message object
        'sender' is sender.id # Assuming sender is an object with an id
        'recipient' is recipient.id
        
    as list    # Define how a message object is converted to a list
        id, sender.id, recipient.id, 
        subject, body, timestamp
```
**Usage (Implied):**
```coral
my_message is message ...
message_string is my_message as string
message_map is my_message as map
```

---

## 11. Built-in Functions and Operations
Coral provides several built-in functions for common tasks. These are often used without explicit module imports for core functionalities.

-   `log 'message'`: Prints a message (e.g., to console or system log).
    ```coral
    log 'debug message: Process started'
    log 'Error: {error_details}'
    ```
-   `hash.blake3 'data'`: Computes a BLAKE3 hash of the input. Other hash algorithms might be available under `hash.*`.
    ```coral
    password_hash is hash.blake3 user_password
    ```
-   `load 'filepath'`: Loads data, typically from a file (e.g., JSON configuration).
    ```coral
    config is load 'config.json' err {}
    ```
-   `create <object_type_or_data>`: Used within `store` object `make` methods to persist new records.
    ```coral
    # Inside a 'store' object's 'make' method:
    return log create $description, $priority # $variables are instance properties
    
    # Or for creating other records:
    # create user_record name 'Test', email 'test@example.com'
    ```
-   `update <record_id> with <changes>`: Used to update persistent records.
    ```coral
    # Inside a 'store' object's 'complete' method:
    return log update id complete is true # 'id' is resolved, 'complete is true' are changes
    ```
-   `now`: Returns the current timestamp. Used for default values.
    ```coral
    timestamp ? now
    ```
-   `empty`: Represents an empty value, often for strings.
    ```coral
    subject ? empty
    ```

---

## 12. Special Operators and Syntax Keywords
Coral uses several keywords and symbols in unique ways to maintain readability and simplicity.

-   `is`:
    -   Assignment: `variable is value`
    -   Property definition in object literals: `host is 'localhost'`
    -   Part of `until` loop condition: `until iterator ... is 8`
-   `with`:
    -   Function/method parameter list: `fn greet with name`
    -   Passing extra arguments in `across` iteration: `check_health across ... with host 'localhost'`
    -   Fetching/creating an instance with specific properties: `user with name 'root'`, `task with $description`
-   `at` / `@`: Array/list element access: `primes at 0`, `primes@0`.
-   `?`:
    -   Default value assignment for properties/parameters: `priority ? 1`, `greeting 'Hello'`
    -   Ternary conditional (true part): `condition ? 'value_if_true'`
-   `!`:
    -   Ternary conditional (false part): `condition ? ... ! 'value_if_false'`
    -   Forced method call (when name conflicts with attribute): `d1.process!`
    -   Assert success on instantiation: `brandon is user! 'brandon', ...`
-   `&`: Join table reference in `store actor` definitions: `&blocklist`.
-   `@`:
    -   Annotation for actor message handlers: `@receive`, `@receive_task`.
    -   Alternative array access: `primes@0`.
-   `$`:
    -   Parameter/property reference: `$id`, `$description`.
    -   Positional parameter reference: `$0`, `$1`.
    -   Placeholder for current item in `iterate` loops: `iterate collection_name operation $`.
-   `then` / `and`: Method chaining: `object.method1 then .method2 and .method3`.
-   `across`: Keyword for iteration: `check_health across nodes`.
-   `into`: Collects results from `across` iteration: `... into results_collection`.
-   `unless`: Conditional execution if false: `unless condition ...`.
-   `err`: Introduces error handling logic: `operation err log return`, `operation err default_value`.
-   `gt`, `lt`, `equals`: Comparison operators: `x gt 10`, `y lt 5`, `z equals 0`.

---

## 13. Module System (`use`)
Coral has a module system to organize code and extend functionality. Modules can add new syntax, functions, and even influence compiler behavior.

```coral
use coral.neural
use coral.net.web

# After 'use coral.neural', new syntax or objects might be available:
# e.g., defining a neural network (hypothetical based on module name)
classifier is neural network
    type is 'classifier'
    layers is [10, 5, 1]

# After 'use coral.net.web':
# e.g., starting a web server (hypothetical)
server.listen 8080
```
The `use` statement makes the functionalities of the specified module available in the current scope.

---

## 14. Advanced Features
Coral's philosophy includes leveraging the compiler for complex tasks, simplifying the developer's job.

### Compile-time Evaluation
The compiler can automatically detect and evaluate expressions at compile-time if all their inputs are known. This is useful for defining constants or pre-calculating data.

```coral
PI is 3.1415926535 # A constant
CONSTANT_VALUE is 2 * PI # Evaluated at compile-time

# Assumes generate_primes is a function the compiler can run
lookup_table is generate_primes 1000 # Table generated during compilation
```

### Automatic Parallelization
The compiler is designed to automatically parallelize operations that are safe to run concurrently, particularly on large datasets.

```coral
# If expensive_calculation is a pure function and large_dataset is suitable,
# the compiler might distribute the work across multiple cores.
results is large_dataset.map expensive_calculation # '.map' implies an operation suitable for this
```
The `across` keyword might also be a candidate for automatic parallelization.

### Distributed Computing
Coral aims for seamless execution of code, whether locally or remotely, especially within its actor model.
```coral
store actor worker
    # ...
    @process_task with data
        # This method might be invoked on a worker actor
        # running on a different machine in a cluster.
        # The developer writes the logic, the Coral runtime handles distribution.
        result is perform_heavy_computation data
        return result
```
The location transparency of actor message passing facilitates this.

---

## 15. Style Guidelines
These guidelines help maintain readable and effective Coral code.

1.  **Use descriptive names**: Prefer `user_count` over `uc`.
2.  **Leverage inference**: Let the compiler infer types; explicit type declarations are generally not used.
3.  **Prefer readability**: Choose constructs that are closer to natural language, e.g., `unless x equals 0` is often preferred over `if x != 0` (assuming `!=` exists or `not (x equals 0)`).
4.  **Chain operations**: Use method chaining with `then` and `and` for fluent, step-by-step data transformations or sequences of actions: `data.filter active then .map process and .save`.
5.  **Use defaults wisely**: Define sensible default values for function parameters and object properties using `?` to make usage simpler: `priority ? 1`.
6.  **Trust the compiler**: Write code that clearly expresses your intent. Rely on the Coral compiler to handle optimizations, parallelization, and other complexities.

---

## 16. Complete Example (Annotated)

```coral
# Import necessary modules for store, actor, and potentially net functionalities
use coral.store, coral.actor, coral.net # '.net' might not be used here but good practice

# Define an actor that is also a persistent store object
store actor task_processor
    name                # Name of this processor instance
    &pending_tasks      # Join table for tasks waiting to be processed
    &completed_tasks    # Join table for tasks that have been processed
    
    # 'make' is called when a 'task_processor' instance is created
    make
        log 'Task processor {name} initialized' # Logs with string interpolation
    
    # Message handler for ':receive_task' messages
    # Parameters (e.g., description, priority) are passed via $ convention from the message
    @receive_task
        # Create a new 'task' object (assuming 'task' is another defined store object)
        # 'with' instantiates using named parameters from the message payload ($description, $priority)
        new_task is task with $description, $priority
        
        # Add the new task to the list of pending tasks for this actor
        push new_task on pending_tasks
        
        # Call another method of this actor to start processing
        # '!' forces the method call
        process_next_task! 
    
    # Method to process the next task from the pending queue
    process_next_task
        # 'unless' checks if the condition (pending_tasks.empty) is false
        unless pending_tasks.empty
            # 'pop' removes and returns the last task from 'pending_tasks'
            current_task is pending_tasks.pop 
            
            # 'process' is assumed to be a function or method that handles a task
            # and returns a result
            result is process current_task 
            
            # Add the processed task (or its result) to the 'completed_tasks' list
            push result on completed_tasks
            
            log 'Completed: {current_task.description}' # Log completion

# Instantiate a task_processor actor
# The string 'main' is passed to the 'name' property during 'make'
processor is task_processor 'main'

# Send a message to the 'processor' actor to trigger the '@receive_task' handler
# The message implicitly contains 'description' and 'priority' fields
processor.receive_task 'Build the future', 1 
# This is equivalent to: processor.send_message('receive_task', description 'Build the future', priority 1)
# (actual message sending mechanism might vary but dot call implies direct method call or message)
```