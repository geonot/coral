# Concurrency: The Actor Model

Effective concurrency is crucial for modern applications, but traditional approaches using threads, locks, and shared memory are notoriously complex and error-prone. Coral addresses this challenge with a **built-in actor model**, designed for safe, scalable, and simplified concurrent programming. This model is deeply intertwined with Coral's **transparent persistent object model**, where the Coral runtime and compiler handle much of the underlying complexity, allowing developers to focus on high-level application logic.

## 1. What is an Actor?

At its core, an actor is an independent computational entity that encapsulates both state and behavior. Key principles:

*   **Isolation:** Each actor has its own private state, inaccessible directly by others. This is fundamental to preventing data races.
*   **Message Passing (Abstracted):** Actors communicate via asynchronous method calls on actor references. The Coral system translates these calls into messages, managing their dispatch and queuing.
*   **Asynchronous Processing:** Actors process messages (method calls) from their internal mailbox one at a time. This serialized processing ensures that an actor's internal state is never accessed concurrently by multiple threads, eliminating the need for manual locking and simplifying concurrent state management.

Think of actors as independent agents, each with their own responsibilities and private data, communicating cleanly without direct interference.

## 2. Defining Actors in Coral

In Coral, actors are typically persistent objects. A class can be designated as an actor using the `@persistent` decorator. Its public methods become the "messages" it handles. All methods adhere to Coral's standard `(result, (error_id, "description"))` tuple return convention.

```coral
@persistent // Marks this class as a persistent object, managed by the actor system
class UserAccount:
    _owner_name is "" // Internal state, conventionally private
    _balance is 0.0
    _transaction_log is [] // Represents a list of transaction strings

    // Constructor
    def init(this, owner_name_val, initial_deposit_val):
        this._owner_name is owner_name_val
        if initial_deposit_val lt 0.0:
            // Actor init methods also return (this_or_null, error_tuple)
            return (null, (1, "Initial deposit cannot be negative"))
        this._balance is initial_deposit_val
        this._transaction_log is [] // Initialize with an empty log
        // Call to internal method, also returns a tuple (error handling omitted for brevity here)
        this._log_event('Account created for {owner_name_val} with initial balance: ${initial_deposit_val}')
        return (this, (0, "")) // Successful init returns the instance

    // Public method, acts as a message handler when called externally
    def deposit(this, amount_val):
        if amount_val lte 0.0:
            // print('Error: Deposit amount must be positive for account {this.id()}.') // Assuming this.id()
            return (this, (101, "Deposit amount must be positive")) // Return 'this' even on logical error
        this._balance is this._balance + amount_val
        this._log_event('Deposited ${amount_val}. New balance: ${this._balance}.')
        return (this, (0, "")) // Default success return for instance methods primarily causing side effects

    def withdraw(this, amount_val):
        if amount_val lte 0.0:
            return (false, (102, "Withdrawal amount must be positive")) // Explicit boolean result
        if this._balance gte amount_val:
            this._balance is this._balance - amount_val
            this._log_event('Withdrew ${amount_val}. New balance: ${this._balance}.')
            return (true, (0, "")) // Explicit boolean result
        else:
            this._log_event('Failed withdrawal of ${amount_val}. Insufficient funds.')
            return (false, (103, "Insufficient funds"))

    def get_balance(this):
        this._log_event("Balance requested.")
        return (this._balance, (0, "")) // Explicitly return the balance value

    def get_transaction_history(this):
        // Return a copy to maintain encapsulation.
        // For a simple list of strings, direct copy is fine.
        // More complex persistent collections might have specific cloning methods.
        return (this._transaction_log, (0, ""))

    // Internal helper method
    def _log_event(this, event_details_str):
        // timestamp_str is Time.now_string().0 // Assume a Time utility, .0 to get value
        // For simplicity, we'll just add the event details.
        // For high-frequency updates on large persistent lists, Coral's standard library
        // might offer optimized persistent collection types and methods.
        this._transaction_log is this._transaction_log + ['{event_details_str}'] // Immutable-style list update
        return (true, (0, "")) // Success for the logging action

    // Actors/persistent objects might have a system-provided unique ID, e.g.:
    // def id(this): return (system_provided_id_value, (0,""))
```
Instances of `UserAccount`, when managed by Coral's persistence and concurrency layer, behave as actors.

## 3. Creating and Interacting with Actors

### Spawning Actors

Creating an actor is like instantiating any other Coral class. If the class is marked `@persistent`, the Coral runtime automatically handles its registration within the actor environment and its persistence.

```coral
// Spawning new UserAccount actors
(alice_account, alice_err) is UserAccount("Alice Wonderland", 1000.0)
(bob_account, bob_err) is UserAccount("Bob The Builder", 500.0)

if alice_err.0 neq 0:
    print('Error creating Alice account: {alice_err.1}')
if bob_err.0 neq 0:
    print('Error creating Bob account: {bob_err.1}')
```
Each successful call to `UserAccount(...)` creates a new, independent actor.

### Asynchronous Method Calls (Message Passing)

Interacting with an actor involves calling its methods. These calls are inherently **asynchronous** and return a `Future`. This `Future` will eventually resolve to the standard Coral `(result, error_details)` tuple that the method itself returns. This design allows the caller to continue execution without waiting, promoting responsiveness.

*   **Calls with "Fire-and-Forget" Style Intent:**
    Even if a method like `deposit` returns `(this, (0,""))`, the call is still asynchronous. You receive a `Future`. If you don't immediately need to confirm completion or handle the result, you might not `await` the future right away.

    ```coral
    if alice_err.0 eq 0: // Assuming alice_account was created successfully
        future_deposit_outcome is alice_account.deposit(200.0)
        // future_deposit_outcome is a Future that will resolve to (UserAccount_instance, (Int, String))
        // The code continues executing. The deposit happens concurrently.
        // One might later await future_deposit_outcome if confirmation is needed.

    if bob_err.0 eq 0:
        future_withdraw_outcome is bob_account.withdraw(50.0)
        // future_withdraw_outcome is a Future that will resolve to (Boolean_value, (Int, String))

    print("Deposit and withdrawal requests sent; operations are pending completion by actors.")
    ```

*   **Calls with Anticipated Return Values (Futures):**
    When an actor method is designed to return a specific value (e.g., `get_balance`), the asynchronous call returns a `Future` that will resolve to `(the_value, (0,""))` on success, or an error tuple.

    ```coral
    if alice_err.0 eq 0:
        future_balance_alice is alice_account.get_balance()
        // future_balance_alice is a Future that will resolve to (Float_value, (Int, String))

    print("Balance request for Alice sent; result is pending in a future.")
    ```

    To get the actual `(result, error_details)` tuple from a `Future`, you typically use an `await` keyword within an `async def` function (Coral's specific syntax for defining and running `async` functions is conceptual here, but the `await` pattern is key).

    ```coral
    // Hypothetical usage with async/await
    async def perform_transactions_and_check():
        // Assume alice_account and bob_account are valid actor references from successful creation

        alice_account.deposit(150.0) // Fire-and-forget style for this example
        bob_account.deposit(75.0)

        // Call get_balance and await the future's resolution
        // The 'await' keyword "pauses" execution of this async function here until the future resolves.
        (resolved_alice_balance_tuple, future_sys_err_alice) is await alice_account.get_balance()

        if future_sys_err_alice.0 eq 0: // Check if the Future itself resolved without system error
            (balance_val, method_err_alice) is resolved_alice_balance_tuple // Unpack method's return
            if method_err_alice.0 eq 0:
                print('Alice''s current balance: ${balance_val}')
            else:
                print('Error from get_balance method: {method_err_alice.1}')
        else:
            // This indicates a system-level issue with the future or actor communication
            print('System error resolving future_balance_alice: {future_sys_err_alice.1}')


        (resolved_bob_withdrawal_tuple, future_sys_err_bob) is await bob_account.withdraw(200.0)
        if future_sys_err_bob.0 eq 0:
            (withdrawal_succeeded, method_err_bob) is resolved_bob_withdrawal_tuple
            if method_err_bob.0 eq 0:
                if withdrawal_succeeded:
                    print('Bob''s withdrawal of $200 was successful.')
                else:
                    print('Bob''s withdrawal of $200 failed (as per method logic).') // e.g. insufficient funds
            else:
                print('Error from withdraw method: {method_err_bob.1}')
        else:
            print('System error resolving future_bob_withdrawal: {future_sys_err_bob.1}')

        // (run_op_result, run_op_err) is Scheduler.run(perform_transactions_and_check()) // Conceptual scheduler
    ```
The Coral runtime ensures that each actor processes messages from its mailbox sequentially, guaranteeing serialized access to its internal state, thus simplifying concurrent programming logic.

## 4. State Management

An actor's state (its attributes) is strictly private. Modifications occur *only* when the actor processes a message (i.e., executes one of its own methods). This disciplined, serialized access to state is the cornerstone of the actor model's safety, eliminating race conditions without requiring manual locks from the developer.

## 5. Benefits of Coral's Actor Model

*   **Simplified Concurrency:** Developers work with familiar object-oriented method calls. The Coral runtime handles the complexities of message passing, queuing, and thread management.
*   **Safety by Design:** State isolation and serialized message processing inherently prevent data races on actor state.
*   **Scalability:** Actors are designed to be lightweight and independent. An application can potentially consist of millions of actors. The Coral runtime could distribute actors across multiple CPU cores or even (conceptually) across different machines in a cluster.
*   **Fault Tolerance (Conceptual):** Actor systems often incorporate supervision hierarchies. If an actor encounters an error, its supervisor can decide how to handle it (e.g., restart the actor, escalate the error). Coral could provide default supervision strategies or allow developers to define custom ones.

## 6. Actors and the Persistent Object Model

The true power of Coral's approach lies in the seamless integration of its actor model with its **transparent persistent object model**. In essence:

*   **Persistent Objects *are* (or can be) Actors:** When you define a class as `@persistent`, its instances are not only managed for concurrency as actors but their state is also automatically persisted by the Coral runtime.
*   **Effortless Persistence:** Developers are freed from writing separate code for database interaction (saving, loading, updating actor state). If `alice_account.deposit(100.0)` is called, the change to `alice_account._balance` is eventually made durable automatically by the Coral system. Even if the application restarts, Alice's account will reflect its last known state.
*   **Simplified Development:** This unified model means developers can focus on business logic and interactions. Coral handles the underlying complexities of both concurrent access and data persistence, leading to a more productive and joyful development experience.

By unifying concurrency and persistence through this actor-based approach, Coral aims to provide an exceptionally robust and developer-friendly platform for building sophisticated, stateful applications.
