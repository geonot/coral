
## Coral Syntax

A clear, whitespace-significant language designed for readability and productivity.

### Assignment

Assignment is flexible. The `is` keyword assigns the value on the right to the name on the left, or vice-versa.

```coral
message is 'hello coral'
ITERATIONS is 100
threshold is 0.95

// The reverse is also valid
3.1415926535 is PI
```

### Literals

Coral supports a rich set of literals. Strings can be single or double-quoted and support interpolation.

```coral
m is 'hello {name}'
n is "string literal"
x is 10
y is 1.0
z is true
a is no      // a special null-like value, distinct from 'empty'
b is empty   // represents an empty collection or value
c is 0x0F    // hex
d is b101    // binary
e is now     // a timestamp literal
```

### Data Structures

#### Lists

Lists are ordered collections, created with parentheses. They are 0-indexed.

```coral
primes is (2, 3, 5, 7, 11)
first_prime is primes(0)

primes.put(13) // Add an item
item is primes.pop
count is primes.size
```

#### Maps

Maps are key-value pairs. They can be defined inline with `key: value` syntax or using an indented block.

```coral
// Inline map
net_config is (host: 'localhost', port: 5000)

// Block map
net_config is
    host is 'localhost'
    port is 5000
```

### Operators

Coral prefers word-based operators for clarity.

*   **Comparison:** `equals`, `not equals`, `gt`, `gte`, `lt`, `lte`
*   **Logical:** `and`, `or`, `not`
*   **Arithmetic:** `+`, `-`, `*`, `/`, `%`

### Functions and Function Calls

Functions are defined with `fn`. Arguments can have default values. Calls can use positional or named arguments.

```coral
fn greet(name, greeting ? 'hello')
    '{greeting}, {name}.'

fn compute_total(price, quantity, tax_rate ? 0.07)
    sub_total is price * quantity
    sub_total + (sub_total * tax_rate)

// Positional arguments
greet('brandon')

// Named arguments
order_custom_tax is compute_total(price: 100, quantity: 5, tax_rate: 0.05)
```

### Conditionals

Conditional logic uses `if`/`else` and `unless`. Blocks are defined by indentation. No colons are needed.

```coral
// Ternary expression
status_text is system_status.load_average.gt(0.9) ? 'High Load' ! 'Normal Load'

// If/else block
if user.is_admin
    grant_access()
else
    log 'access denied'

// Unless block
unless x.equals(0)
    process x

// Postfix unless
process x unless x.equals(0)
```

### Loops

Coral provides several ways to loop, all defined by indentation.

```coral
// While loop
while iterator.lt(3)
    log 'iterator is {iterator}'

// Until loop
until iterator.equals(8)
    log 'iterator is {iterator}'

// Iterate over a collection
iterate(system_status.active_nodes)
     log check_health $ // '
 refers to the current item

// Apply a function across a collection
check_health.across(system_status.active_nodes)

// Pipe the results into a new variable
check_health.across(system_status.active_nodes).into(node_status)
```

### Objects

Objects are user-defined types with fields and methods.

```coral
object datapoint
    value
    processed ? no
    timestamp ? now

    make(val) // A constructor-like method
        value is val

    process
        processed is yes

d1 is datapoint.make(42)
d1.process()
```

### Persistent Objects (`store`)

The `store` keyword defines an object whose state is managed by a persistent backend.

```coral
store task
    description
    priority ? 1
    complete ? no

    is_done
        return complete

task1 is task.make('Do coding')
task_completed is task.is_done(task1)
```

### Actor Objects

Actors are objects that run concurrently and communicate via messages.

```coral
store actor user
    name
    email
    &blocklist // Relation to another store
    &message   // Relation to another store

    @receive_message(msg) // Message handler
        log 'Received: {msg}'
```

### Error Handling

Coral has built-in syntax for handling operations that can fail.

```coral
// If load fails, config becomes an empty map
config is load('coral.json') err (:)

// If the operation fails, log the error and return from the function
record is user.with('name', 'root') err return log err
```

### Modules

Code can be organized into modules with `mod` and imported with `use`.

```coral
// lib/http.co
mod http
    fn get(url)
        ...

// main.co
use http
get('https://somesite.com')
```
