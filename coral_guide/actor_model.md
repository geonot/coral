# Concurrency: The Actor Model

Effective concurrency is crucial for modern applications, but traditional approaches using threads, locks, and shared memory are notoriously complex and error-prone, often leading to issues like race conditions and deadlocks. Coral addresses this challenge with a **built-in actor model**, designed for safe, scalable, and simplified concurrent programming.

This actor model is deeply intertwined with Coral's **persistent object model**. This powerful combination aims to dramatically simplify the development of robust concurrent and distributed applications by handling both state persistence and concurrent access seamlessly.

## 1. What is an Actor?

At its core, an actor is an independent computational entity that encapsulates both state and behavior. The actor model is based on a few key principles:

*   **Isolation:** Each actor has its own private state. This state cannot be directly accessed or modified by other actors. This isolation is fundamental to preventing data races.
*   **Message Passing (Abstracted):** Actors communicate by sending messages to each other. In Coral, this is often abstracted to look like asynchronous method calls on actor references. The system translates these calls into messages.
*   **Asynchronous Processing:** Actors process messages (method calls) one at a time from an internal mailbox. This serialized processing of incoming requests ensures that an actor's internal state is never accessed concurrently by multiple threads, eliminating the need for manual locking.

Think of actors as independent agents, each with their own responsibilities, their own private data, and a mailbox for receiving requests from other agents. They work autonomously and communicate without directly interfering with each other's internal workings.

## 2. Defining Actors in Coral

In Coral, actors are typically special kinds of persistent objects. You don't necessarily use a distinct `actor` keyword; instead, a class can be designated as an actor, often implicitly through its usage or via an annotation like `@persistent` which also implies actor-like concurrent behavior.

The "messages" an actor handles are its public methods. When a method is called on an actor reference from outside, Coral's runtime treats it as an asynchronous message being sent to that actor.

```coral
@persistent // Marks this class as a persistent object, managed by the actor system
class UserAccount {
    private owner_name: String;
    private balance: Float;
    private transaction_log: List<String>; // Assuming List is a persistent collection

    // Constructor
    fn init(owner: String, initial_deposit: Float) {
        this.owner_name = owner;
        this.balance = initial_deposit;
        this.transaction_log = []; // Initialize with an empty log
        this._log_event(f"Account created for {owner} with initial balance: ${initial_deposit}");
    }

    // Public method, acts as a message handler when called externally
    pub fn deposit(this, amount: Float) {
        if amount <= 0.0 {
            print(f"Error: Deposit amount must be positive for account {this.id()}.");
            return; // Or throw an error
        }
        this.balance += amount;
        this._log_event(f"Deposited ${amount}. New balance: ${this.balance}.");
    }

    // Another public method
    pub fn withdraw(this, amount: Float) -> Boolean {
        if amount <= 0.0 {
            print(f"Error: Withdrawal amount must be positive for account {this.id()}.");
            return false;
        }
        if this.balance >= amount {
            this.balance -= amount;
            this._log_event(f"Withdrew ${amount}. New balance: ${this.balance}.");
            return true;
        } else {
            this._log_event(f"Failed withdrawal of ${amount}. Insufficient funds.");
            return false;
        }
    }

    pub fn get_balance(this) -> Float {
        this._log_event("Balance requested.");
        return this.balance;
    }

    pub fn get_transaction_history(this) -> List<String> {
        return this.transaction_log.copy(); // Return a copy to maintain encapsulation
    }

    // Internal helper method, not directly callable as a message from outside
    private fn _log_event(this, event_details: String) {
        let timestamp = Time.now_string(); // Assume a Time utility
        this.transaction_log.add(f"[{timestamp} - ID: {this.id()}] {event_details}");
    }

    // Actors/persistent objects might have a system-provided unique ID
    // fn id(this) -> ActorId; // Conceptually provided by the system
}
```
In this example, `UserAccount` objects, when managed by Coral's persistence and concurrency layer, behave as actors. Their methods `deposit`, `withdraw`, `get_balance`, etc., become the "messages" they can process.

## 3. Creating and Interacting with Actors

### Spawning Actors

Creating an actor is similar to instantiating a regular object. If the class is marked for persistence (and thus actor behavior), the system handles its registration within the actor environment.

```coral
// Spawning a new UserAccount actor
let alice_account = UserAccount.new("Alice Wonderland", 1000.0);
let bob_account = UserAccount.new("Bob The Builder", 500.0);
```
Each call to `UserAccount.new(...)` creates a new independent actor with its own state and mailbox.

### Asynchronous Method Calls (Message Passing)

Interacting with an actor looks like calling methods on its reference. These calls are inherently **asynchronous**.

*   **Fire-and-Forget:** If a method doesn't return a value (or you don't immediately need its result), the call is "fire-and-forget." Your code continues executing while the actor processes the request at some point in the future.

    ```coral
    alice_account.deposit(200.0); // Asynchronously sends a "deposit" message
    bob_account.withdraw(50.0);   // Asynchronously sends a "withdraw" message
    print("Deposit and withdrawal requests sent."); // This line executes immediately
    ```

*   **Calls with Return Values (Futures/Promises):** If an actor method returns a value, the asynchronous call will immediately return a **Future** (or **Promise**). A Future is a placeholder for a value that will be available later.

    ```coral
    let future_balance_alice = alice_account.get_balance(); // Returns a Future<Float>
    let future_withdrawal_status = bob_account.withdraw(100.0); // Returns a Future<Boolean>

    print("Balance and withdrawal requests sent; results are pending in futures.");
    ```

    To get the actual value from a Future, you typically need to use an `await` keyword within an `async` function (if Coral supports async/await syntax) or use callback mechanisms.

    ```coral
    // Hypothetical usage with async/await
    async fn perform_transactions_and_check() {
        alice_account.deposit(150.0);
        bob_account.deposit(75.0);

        let alice_current_balance = await alice_account.get_balance();
        print(f"Alice's current balance: ${alice_current_balance}");

        let bobs_withdrawal_succeeded = await bob_account.withdraw(200.0);
        if bobs_withdrawal_succeeded {
            print("Bob's withdrawal of $200 was successful.");
        } else {
            print("Bob's withdrawal of $200 failed.");
        }

        let bobs_final_balance = await bob_account.get_balance();
        print(f"Bob's final balance: ${bobs_final_balance}");
    }

    // To run an async function:
    // Scheduler.run(perform_transactions_and_check()); // Conceptual scheduler
    ```

Under the hood, each method call on an actor reference is transformed into a message. This message is placed in the target actor's mailbox. The actor processes messages from its mailbox sequentially, ensuring that only one method executes at a time, thus guaranteeing serialized access to its internal state.

## 4. State Management

As highlighted, an actor's state (its attributes) is strictly private and protected from any direct external access. All modifications to an actor's state can *only* occur as a result of the actor itself processing a message (i.e., executing one of its own methods). This disciplined, serialized access to state is the cornerstone of the actor model's safety, eliminating race conditions without requiring manual locks.

## 5. Benefits of Coral's Actor Model

*   **Simplified Concurrency:** Developers work with familiar object-oriented method calls, largely freed from the complexities of manual thread management, locks, semaphores, or mutexes.
*   **Safety by Design:** The model inherently prevents data races on actor state because of state isolation and serialized message processing.
*   **Scalability:** Actors are lightweight and independent. An application can consist of millions of actors. The Coral runtime could potentially distribute actors across multiple CPU cores or even (conceptually) across different machines in a cluster, enhancing scalability.
*   **Fault Tolerance (Conceptual):** Actor systems often incorporate supervision hierarchies. If an actor encounters an error, its supervisor can decide how to handle it (e.g., restart the actor, escalate the error). Coral could provide default supervision strategies or allow developers to define custom ones, contributing to more resilient applications (this is a more advanced topic).

## 6. Actors and the Persistent Object Model

The true power of Coral's approach lies in the seamless integration of its actor model with its **persistent object model**. In essence:

*   **Persistent Objects *are* (or can be) Actors:** When you define a class as `@persistent`, its instances are not only managed for concurrency as actors but their state is also automatically persisted by the Coral runtime.
*   **Effortless Persistence:** There's no need to write separate code for database interaction (saving, loading, updating actor state). If `alice_account.deposit(100.0)` is called, the change to `alice_account.balance` is eventually made durable automatically. Even if the application restarts, Alice's account will reflect its last known state.
*   **Simplified Development:** This unified model means developers can focus on business logic and interactions, while Coral handles the underlying complexities of concurrent access and data persistence. You design your objects and their interactions, and Coral ensures they can run concurrently and their state endures.

By unifying concurrency and persistence through this actor-based approach, Coral aims to provide an exceptionally productive and robust platform for building sophisticated, stateful applications.
