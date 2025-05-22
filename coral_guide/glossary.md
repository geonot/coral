# Glossary

This glossary provides definitions for key terms, keywords, and concepts used throughout the Coral Language Guide. Understanding these terms will help in comprehending Coral's features and design philosophy.

---

**Actor Model**
Coral's built-in concurrency model. Actors are independent computational entities that encapsulate state and behavior, communicating via asynchronous messages (abstracted as method calls). Each actor processes messages sequentially, ensuring safe concurrent operations without manual locks, simplifying complex concurrent programming.

**Asynchronous Call**
A call to a function or method (especially an actor method) that returns immediately, typically with a `Future` object, without waiting for the operation to complete. The operation itself is performed concurrently by the Coral runtime, allowing the calling code to remain responsive.

**`@persistent` (decorator)**
A decorator used with a class definition (e.g., `@persistent class MyData:`) to mark its instances as persistent objects. Coral's runtime automatically manages the state of these objects, transparently saving changes and loading them as needed. Persistent objects are often used as actors.

**`class` (keyword)**
The keyword used to define a class, which serves as a blueprint for creating objects. Example: `class MyClass: ...`. Classes encapsulate data (attributes) and behavior (methods).

**Closure**
A function that captures variables from its enclosing lexical scope. It "remembers" the environment in which it was created and can access those captured variables even when executed outside that original scope. Closures are a powerful feature for creating specialized functions and for patterns like callbacks.

**Constant**
A binding whose value is intended to remain unchanged throughout its scope. In Coral, constants are conventionally named in `ALL_UPPERCASE` and assigned using the `is` keyword (e.g., `MAX_SIZE is 100`). This convention aids code readability and signals intent.

**`def` (keyword)**
The keyword used to define a function or a method within a class. Example: `def my_function(param1, param2): ...`. All functions and methods defined with `def` adhere to Coral's tuple return convention.

**Error Handling**
Coral's primary mechanism for dealing with runtime errors, based on the universal tuple return convention `(result, (error_id, "description_string"))` for all functions and methods. This makes error conditions an explicit part of a function's contract.

**Error ID**
The numerical identifier within the error part of Coral's tuple return convention (i.e., the first element of the `error_details` tuple, `error_details.0`). An `error_id` of `0` signifies success, while any non-zero value indicates an error.

**First-Class Functions**
A characteristic of Coral where functions are treated like any other value: they can be assigned to variables, passed as arguments to other functions, and returned as results from other functions. This is fundamental to functional programming.

**Future**
An object representing the eventual result of an asynchronous operation, particularly actor method calls. In Coral, a `Future` typically resolves to a `(result, (error_id, "description_string"))` tuple, which is the standard return type of the asynchronous function or actor method that was called.

**Higher-Order Function (HOF)**
A function that either takes one or more functions as arguments, or returns a function as its result, or both. Examples include `map_list` and `filter_list`, which are used for processing collections in a functional style.

**Immutability**
A design principle where data, once created, cannot be changed. Coral encourages immutability through conventions (like `ALL_CAPS` constants) and by promoting a programming style where new data structures are created with modified values rather than altering existing ones in place. This simplifies state management and improves predictability.

**`is` (keyword)**
Coral's sole assignment keyword, used for binding values to variable names or constant names. Example: `my_variable is 10`, `MY_CONSTANT is "value"`. Its consistent use simplifies syntax.

**`it` (keyword)**
An implicit keyword available within an `iter` loop, representing the current item of the iteration. This allows for concise loop bodies.

**`iter` (keyword)**
Coral's primary keyword for looping and iteration over collections or ranges (e.g., `0..5`). Example: `iter my_list: ... print(it) ...`.

**Message Passing**
The communication mechanism between actors. In Coral, this is abstracted as asynchronous method calls on actor references. The Coral runtime translates these calls into messages, which are queued and processed by the recipient actor.

**Method**
A function that is defined within a class. Instance methods, the most common type, operate on instances of that class and are defined with `this` as their first parameter.

**Module**
A file containing Coral code (typically with a `.cr` extension), which serves as a unit for organizing and encapsulating code. Modules can import other modules to use their definitions.

**Object**
An instance of a class. Objects encapsulate state (attributes) and behavior (methods), forming the basis of Object-Oriented Programming.

**Persistent Object**
An object whose state is automatically managed and saved by Coral's runtime, allowing it to live beyond a single program execution. Classes are typically marked with `@persistent` to create persistent objects. This feature is central to Coral's approach to simplifying stateful application development.

**Persistent Object Model**
Coral's built-in system that transparently manages the lifecycle (creation, saving, loading, deletion) and data integrity of persistent objects. This often eliminates the need for external databases or ORMs for many common use cases, allowing developers to work directly with objects whose state is effortlessly persisted by the Coral system.

**Pure Function**
A function that adheres to two properties: 1) it is deterministic (always returns the same output for the same set of input arguments), and 2) it has no side effects (does not modify external state or perform I/O). Pure functions are easier to test and reason about.

**Side Effect**
An action where a function or expression modifies some state outside its local environment (e.g., changing a global variable, modifying a mutable object passed as an argument) or has an observable interaction with the outside world (e.g., printing to the console, writing to a file). Minimizing side effects is a key principle in functional programming.

**String Interpolation**
A process of embedding expressions (variables, constants, or other calculations) directly within a string literal. In Coral, interpolated strings are enclosed in single quotes (`'`) and use curly braces `{}` to delimit expressions, e.g., `'Hello, {user_name}!'`.

**String Literal**
A sequence of characters treated as a fixed value. In Coral, literal strings (which do not perform interpolation and treat all characters, including `{`, verbatim) are enclosed in double quotes (`"`), e.g., `"This is a {literal_brace} string."`.

**`this` (keyword)**
A keyword used within an instance method of a class to refer to the current object instance itself.

**Tuple Return Convention (`(result, error_details)`)**
The standard way all functions and methods in Coral return values. It's a 2-tuple where the first element is the actual `result` of the operation (or `null`/`true` as appropriate for the context), and the second element is the `error_details` tuple: `(error_id, "description_string")`. An `error_id` of `0` indicates success, while non-zero indicates an error. This convention ensures explicit and consistent error handling.

**Type Inference**
The Coral compiler's ability to automatically deduce the data type of a variable or constant from its assigned value, without requiring explicit type declarations from the developer. Coral features complete type inference, which contributes to its clean syntax and reduces boilerplate.

**Variable**
A named binding for a value that can be reassigned using the `is` keyword. Variable names are conventionally written in `snake_case` (e.g., `my_user_count`).

---

This glossary aims to be a helpful reference as you explore the Coral language. For more detailed explanations, please refer to the relevant chapters in this guide.
