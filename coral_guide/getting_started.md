# Getting Started with Coral

This guide will walk you through installing Coral and writing your very first Coral program.

## Installation

There are several ways to get Coral installed on your system. Below are a few common methods.

### Binary Release

For most users, the quickest way to get started is by downloading a pre-compiled binary for your operating system from our official downloads page:

*   **Official Downloads:** [https://coral-lang.org/downloads](https://coral-lang.org/downloads)

Download the appropriate package for your system (e.g., Windows, macOS, Linux), extract it, and add the Coral executable to your system's PATH.

### Package Manager (corpkg)

If you prefer using a package manager, Coral has its own (fictional) package manager called `corpkg`.

1.  **Install `corpkg`:** Follow the instructions on the Coral website to install `corpkg`.
2.  **Install Coral:** Once `corpkg` is set up, you can install Coral by running:
    ```bash
    corpkg install coral
    ```

### Building from Source

For advanced users or those who want the latest cutting-edge version, you can also build Coral from source. Instructions for this are available in the main repository of the Coral language.

**Note:** Always refer to the official Coral website ([https://coral-lang.org](https://coral-lang.org)) for the most up-to-date and detailed installation instructions.

## Your First Coral Program: Hello, Coral!

Let's dive in and write a simple "Hello, Coral!" program. Coral aims for a syntax that is intuitive and easy to read, drawing inspiration from the clarity of Python.

1.  **Create a file:**
    Open your favorite text editor and create a new file. Save it as `main.cr`. The `.cr` extension is conventionally used for Coral source files.

2.  **Write the code:**
    Enter the following lines into `main.cr`:

    ```coral
    // main.cr - Your first Coral program
    // This line prints a greeting to the console.
    print("Hello, Coral!");
    ```

    In this simple program:
    *   `//` denotes a single-line comment.
    *   `print()` is a built-in function that outputs text to your console.

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

Congratulations! You've successfully written and executed your first Coral program. In the next chapters, we'll explore the features and syntax of Coral in more detail.
