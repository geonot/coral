# Persistence: The Built-in Object Model

Managing data persistence is often one of the most complex and boilerplate-heavy aspects of application development. Traditional approaches involve Object-Relational Mappers (ORMs), external databases, SQL, and the constant "impedance mismatch" between the object-oriented world of the application and the relational world of the database. Coral aims to radically simplify this with its **built-in, transparent persistent object model**.

Coral's persistence is not an afterthought or an add-on library; it's a fundamental feature of its runtime, seamlessly integrated with its object and actor models. The philosophy is largely **"it just works"**: the Coral compiler and runtime handle the complexities of data storage, retrieval, and consistency, allowing developers to focus on their application's logic rather than data plumbing.

## 1. What are Persistent Objects?

In Coral, objects of classes marked with the `@persistent` decorator have their state automatically managed by Coral's runtime. This means their data lives beyond a single program execution, effectively making them durable without manual intervention.

*   **Integration with Actors:** As detailed in the "Concurrency: The Actor Model" chapter, persistent objects are the foundation of actors in Coral. An actor's state is inherently persistent, managed by the same underlying system.
*   **General Purpose:** Any class can be marked `@persistent`, not just those intended to be actors. This allows a wide range of application data—from simple configuration objects to complex domain entities—to be managed effortlessly by Coral's persistence engine.
*   **Transparent Lifecycle:** The lifecycle (creation, saving, loading, deletion) and data integrity of these objects are handled transparently by the Coral runtime.

```coral
@persistent // Marks instances of BlogPost as persistent
class BlogPost:
    title is ""
    content is ""
    author_id is null      // Could be an ID linking to a User object (another persistent object)
    published_at is null   // Could be a DateTime object, also potentially persistent or serializable

    def init(this, title_val, content_val, auth_id_val):
        this.title is title_val
        this.content is content_val
        this.author_id is auth_id_val
        // 'init' implicitly returns (this, (0,"")) on success.
        // If initialization could fail (e.g., invalid essential data),
        // it should explicitly return (null, (error_id, "description")).

    def update_content(this, new_content_val):
        this.content is new_content_val
        // (now_val, time_err) is Time.now() // Assuming a Time module
        // if time_err.0 eq 0: this.published_at is now_val
        this.published_at is "current_timestamp_placeholder" // Placeholder for simplicity
        return (this, (0,"")) // Implicitly returns 'this' on successful modification

    def get_summary(this):
        summary_text is 'Title: {this.title}, Author ID: {this.author_id}'
        return (summary_text, (0, ""))
```
Once an instance of `BlogPost` is successfully created, it becomes a persistent entity. Its state is managed by Coral, and changes are automatically persisted.

## 2. How Persistence Works (Conceptual)

The "magic" of Coral's persistence lies in its transparent operation, where the runtime takes on the heavy lifting:

*   **Automatic Saving:** Changes made to persistent objects are automatically and transparently saved by the Coral runtime. There are typically **no explicit `save()` or `update()` calls** needed from application code. This saving mechanism likely occurs at well-defined "safe points," such as after a method call on a persistent object (especially an actor) completes successfully, ensuring that the state written to the persistent store is consistent.
*   **Automatic Loading (Activation):** When a persistent object is referenced (e.g., by its unique ID, or via a relationship from another loaded object), Coral automatically loads its state from the persistent store if it's not already in memory. This process is often called "activation" or "lazy loading." To the developer, it appears as if all persistent objects are always available, though the runtime efficiently manages memory by loading them only on demand.
*   **Object Identity:** Every persistent object in Coral has a unique, stable identity, managed by the system. This ID can be used to retrieve specific objects across different program sessions or even (conceptually) across different processes in a distributed environment. This ID might be accessible via a built-in method like `this.id()`.

## 3. Working with Persistent Objects

Interacting with persistent objects is designed to be as straightforward as working with regular, transient Coral objects.

### Creating Persistent Objects

Creating a persistent object uses the same syntax as any other Coral object. If the class is marked `@persistent`, the runtime automatically handles its persistence.

```coral
USER_ID_123 is "user_abc" // Example user ID

(post_obj, post_err) is BlogPost("My First Persistent Post", "Hello, Coral persistence!", USER_ID_123)

if post_err.0 eq 0:
    // 'post_obj' is now a live, persistent object. Its state is managed by Coral.
    // (id_val, _) is post_obj.id() // Assuming an id() method
    // print('Blog post created with ID: {id_val}')
    print('Blog post "{post_obj.title}" created and is now persistent.')
else:
    print('Error creating blog post: {post_err.1}')
```

### Retrieving Objects (Basic Querying by ID)

The most basic way to retrieve a persistent object is by its unique ID. Coral would provide a system-level mechanism for this, perhaps via a global `Coral` object or a specific `Persistence` module.

```coral
POST_ID is "some_unique_blog_post_id" // Example ID from a previous session or another source

// Hypothetical global 'Coral' object or 'Persistence' module for lookups
(retrieved_post_obj, retrieve_err) is Coral.find_by_id(BlogPost, POST_ID) // Pass class for type safety

if retrieve_err.0 eq 0:
    if retrieved_post_obj neq null:
        // 'retrieved_post_obj' is an instance of BlogPost, fully activated from the store.
        (summary, sum_err) is retrieved_post_obj.get_summary()
        if sum_err.0 eq 0: print('Retrieved: {summary}')
    else:
        // This means find_by_id succeeded but no object with POST_ID exists.
        print('Post with ID "{POST_ID}" not found.')
else:
    // This indicates an error in the retrieval process itself (e.g., store unavailable).
    print('Error retrieving post: {retrieve_err.1}')
```

### Modifying Objects

Modifying a persistent object is done by calling its methods. Any state changes made to `this.attribute` within those methods are automatically marked for persistence by the runtime.

```coral
// Continuing from the previous example, assuming 'retrieved_post_obj' is a valid, loaded object.
if retrieved_post_obj neq null and retrieve_err.0 eq 0: // Ensure it was found and loaded
    print('Updating content for post: {retrieved_post_obj.title}')
    (updated_post_ref, update_err) is retrieved_post_obj.update_content("New, updated content here!")

    if update_err.0 eq 0:
        print('Post content updated. Change is automatically persisted by Coral.')
        // 'updated_post_ref' is 'retrieved_post_obj' itself.
    else:
        print('Error updating post content: {update_err.1}')
```

### Deleting Objects (Conceptual)

To remove a persistent object from the system, Coral would provide a clear mechanism.

```coral
// Continuing from the previous example.
// (delete_success_flag, delete_err) is Coral.delete(retrieved_post_obj)
// Or perhaps: (delete_success_flag, delete_err) is Coral.delete_by_id(BlogPost, POST_ID)

// if delete_err.0 eq 0:
//     if delete_success_flag is true:
//         print('Post titled "{retrieved_post_obj.title}" has been deleted from persistent store.')
//     else:
//         print('Post deletion failed (e.g., not found, permissions).') // Specifics depend on API
// else:
//     print('Error during deletion operation: {delete_err.1}')
```

## 4. Transactions and Consistency (High-Level)

Coral's persistence model aims to provide strong data integrity, likely ensuring ACID properties (Atomicity, Consistency, Isolation, Durability) or a well-defined subset thereof, especially for operations on single objects (actors).

*   **Atomicity:** Operations on a single persistent object are typically atomic. All changes within a method call are applied and persisted, or none are.
*   **Consistency:** The system ensures data remains in a valid state.
*   **Isolation:** Concurrent operations are isolated, managed primarily via the actor model.
*   **Durability:** Confirmed changes are durably stored.

For operations spanning multiple persistent objects requiring coordinated changes, Coral might offer an explicit transaction mechanism, though the goal is for many common multi-object interactions to be manageable with implicit safety guarantees provided by the runtime.

```coral
// Conceptual explicit transaction block:
// transaction: // This is purely illustrative of a potential syntax
//     (res1, err1) is account1.withdraw(100) // account1 is a persistent actor
//     if err1.0 eq 0:
//         (res2, err2) is account2.deposit(100) // account2 is another persistent actor
//         if err2.0 neq 0:
//             transaction.rollback() // Conceptual: Signal rollback for the entire transaction
//             print('Transaction failed on deposit: {err2.1}. Rolling back.')
//     else:
//         transaction.rollback()
//         print('Transaction failed on withdraw: {err1.1}. Rolling back.')
//
// If the block completes without explicit rollback, it commits automatically.
// The Coral runtime would manage the atomicity of this block.
```

## 5. Querying (Advanced - Conceptual Teaser)

Beyond simple ID-based lookups, a mature persistence system requires more powerful querying. Coral would likely provide a dedicated, type-safe query system, potentially inspired by its relational influences, where queries are themselves first-class constructs that the compiler can understand and optimize.

```coral
// Highly conceptual query example:
// (query_result, query_err) is Coral.query(BlogPost) // Start a query for BlogPost objects
//                               .filter(author_id eq USER_ID_123 and published_at neq null)
//                               .sort(published_at, "descending")
//                               .limit(10)
//                               .execute() // Returns (list_of_posts, error_tuple)

// if query_err.0 eq 0:
//     posts_list is query_result.0 // Extract the list of BlogPost objects
//     iter posts_list:
//         print('Found post by user {USER_ID_123}: {it.title}')
// else:
//     print('Error querying posts: {query_err.1}')
```
Such a system would aim for both expressiveness and performance, with the Coral compiler and runtime optimizing query execution against the persistent store.

## 6. Schema Evolution (Briefly)

As applications evolve, class definitions for persistent objects may change. Coral would need to provide robust mechanisms for schema evolution, such as:
*   Automatic migrations for simple changes.
*   Developer-defined transformation logic for complex changes.
*   Versioning of class definitions.
This ensures long-term maintainability of applications and their data.

## 7. Benefits of Coral's Persistent Object Model

*   **Drastically Reduced Boilerplate:** Eliminates ORMs, SQL for CRUD, and data conversion logic.
*   **Improved Developer Productivity and Joy:** Developers work directly and naturally with Coral objects. The system handles persistence transparently.
*   **Seamless Integration with Actor Model:** State management for actors is automatically persistent and concurrent-safe.
*   **Data is Just Objects:** Simplifies the mental model.
*   **Potential for High Performance:** A tightly integrated persistence engine, understood by the compiler, can be highly optimized for Coral's object and concurrency models.

By making persistence a core, transparent feature, Coral frees developers to focus on building compelling application logic, trusting the Coral system to manage the underlying data complexities efficiently and safely.
