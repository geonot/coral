# Getting Started with Coral

This guide will walk you through installing Coral and writing your very first Coral program. Coral is designed to be easy to learn, allowing you to become productive quickly.

## Installation

Getting Coral set up on your system is straightforward. Below are a few common methods.

### Binary Release

For most users, the quickest way to get started is by downloading a pre-compiled binary for your operating system from our official downloads page:

*   **Official Downloads:** [https://coral-lang.org/downloads](https://coral-lang.org/downloads) (Hypothetical URL)

Download the appropriate package for your system (e.g., Windows, macOS, Linux), extract it, and add the Coral executable to your system's PATH.

### Package Manager (corpkg)

If you prefer using a package manager, Coral has its own (fictional) package manager called `corpkg`. This tool simplifies the installation and management of Coral versions and related tools.

1.  **Install `corpkg`:** Follow the instructions on the Coral website ([https://coral-lang.org](https://coral-lang.org)) to install `corpkg`.
2.  **Install Coral:** Once `corpkg` is set up, you can install Coral by running:
    ```bash
    corpkg install coral
    ```

### Building from Source

For advanced users or those who want the latest cutting-edge version, you can also build Coral from source. Instructions for this are available in the main repository of the Coral language.

**Note:** Always refer to the official Coral website ([https://coral-lang.org](https://coral-lang.org)) for the most up-to-date and detailed installation instructions.

## Your First Coral Program: Hello, Coral!

Let's dive in and write a simple "Hello, Coral!" program. Coral's syntax is designed to be intuitive, readable, and expressive, drawing inspiration from Python's clarity while incorporating its own unique features for efficiency and robustness. You'll find that even complex operations can often be expressed cleanly, as the Coral compiler handles much of the underlying work.

1.  **Create a file:**
    Open your favorite text editor and create a new file. Save it as `main.cr`. The `.cr` extension is conventionally used for Coral source files.

2.  **Write the code:**
    Enter the following lines into `main.cr`:

    ```coral
    // main.cr - Your first Coral program
    // Coral uses colons and indentation to define code structure,
    // though this isn't visible in such a short script.
    // Semicolons are not used to terminate statements.

    print("Hello, Coral!") // This line prints a greeting to the console.
    ```

    In this simple program:
    *   `//` denotes a single-line comment. Coral also supports multi-line comments with `/* ... */`.
    *   `print()` is a built-in function that outputs text to your console. Its usage is straightforward, and it implicitly handles the standard `(result, error)` tuple for simple cases.
    *   `"Hello, Coral!"` is a **string literal**. In Coral, string literals (which don't embed variable values) are enclosed in double quotes.
    *   Coral does not use semicolons to end lines; the end of the line signifies the end of the statement.

3.  **Run the program:**
    Open your terminal or command prompt, navigate to the directory where you saved `main.cr`, and run it using the Coral interpreter:

    ```bash
    coral main.cr
    ```

    Alternatively, depending on the version and your setup, you might use:

    ```bash
    coral run main.cr
    ```

    You should see the following output:

    ```
    Hello, Coral!
    ```

Congratulations! You've successfully written and executed your first Coral program.

**Exploring a bit more (Conceptual):**

As you move forward, you'll discover more of Coral's clean and expressive syntax:

*   **Variables and Constants:** You'll use the `is` keyword for assignment. Constant names are typically in `ALL_CAPS`, signaling their immutable intent.
    ```coral
    // Conceptual example:
    // MY_GREETING is "Hello" // A constant
    // target_name is 'World'   // A variable, assigned using 'is'
    // print('{MY_GREETING}, {target_name}!') // Note: interpolated string uses single quotes
    ```
*   **String Interpolation:** Strings that need to embed variable values use single quotes (`'`) and curly braces `{}`.
    ```coral
    // Conceptual example:
    // PLANET is 'Mars'
    // print('Greetings from {PLANET}!') // Output: Greetings from Mars!
    ```
*   **Code Blocks:** For functions, conditional statements (`if`), and loops (`iter`), Coral uses a colon (`:`) followed by an indented block of code. This structure enhances readability.

    ```coral
    // Conceptual example of a function definition:
    // def greet_planet(planet_name): // No type annotations needed; types are inferred
    //     MESSAGE is 'Hello, {planet_name}!' // Assignment with 'is'
    //     print(MESSAGE)
    //     // Functions implicitly return (true, (0, "")) on success if no explicit return
    //
    // (call_result, call_error) is greet_planet('Venus') // All function calls return a tuple
    ```

These concepts will be covered in detail in the "Language Basics" chapter. For now, you've taken the first essential step into the world of Coral! In the next chapters, we'll explore the features and syntax that make Coral a productive and enjoyable language to use.
