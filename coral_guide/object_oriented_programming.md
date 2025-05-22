# Object-Oriented Programming in Coral

Coral seamlessly integrates Object-Oriented Programming (OOP) principles, allowing developers to structure their code around "objects" that combine data (attributes) and behavior (methods). This approach promotes modularity, reusability, and a clear organization of complex software. Coral's OOP is designed to be intuitive, mirroring the ease of use found in languages like Python, while providing robustness and enabling powerful features through compiler support. The compiler handles much of the underlying complexity, allowing for a clean and productive OOP experience.

## 1. Classes and Objects

### Defining Classes

Classes are blueprints for creating objects. You define a class using the `class` keyword, followed by a colon and an indented block for its members.

```coral
class Greeter:
    // Class constant (shared by all instances)
    DEFAULT_PREFIX is "Hello"

    // Constructor method
    def init(this):
        // 'this' refers to the current instance being created.
        // Instance attributes are assigned using 'this.attribute_name is value'.
        this.greeting_prefix is Greeter.DEFAULT_PREFIX // Access class constant via class name
        // On successful initialization, 'init' implicitly returns (this, (0, "")).
        // If initialization can fail, it should explicitly return (null, (error_id, "description")).

    // Instance method
    def greet(this, name):
        message is '"{this.greeting_prefix}, {name}!"' // Interpolated string for the message
        print(message) // Built-in print for simplicity
        return (this, (0, "")) // Default successful return for methods that modify 'this' or perform actions
```

### Creating Objects (Instances)

An object is an instance of a class. You create objects by calling the class name as if it were a function. The `init` method is called automatically during this process. All object instantiations return a tuple `(instance, error_details)`.

```coral
// Create an instance of the Greeter class
(greeter_obj, error) is Greeter()

if error.0 eq 0:
    // 'greeter_obj' is now a valid instance of Greeter.
    // Call a method on the object:
    (call_result, method_error) is greeter_obj.greet("World")
    if method_error.0 neq 0:
        print('Error during greet: {method_error.1}')
else:
    // Object creation failed; 'greeter_obj' might be null or an uninitialized reference.
    print('Error creating Greeter: {error.1}')
```

## 2. Attributes (Fields/Properties)

Attributes are variables that store data associated with an object (instance attributes) or a class itself (class attributes).

### Defining Attributes

*   **Class Attributes (Constants/Defaults):** Defined directly within the class block using `ATTRIBUTE_NAME is value`. They are shared among all instances and accessed via the class name (e.g., `ClassName.ATTRIBUTE_NAME`).
*   **Instance Attributes:** Typically defined and initialized within the `init` method using `this.attribute_name is value`. Each object instance gets its own copy of these attributes.

```coral
class Circle:
    PI is 3.14159 // Class attribute (constant)

    def init(this, radius, color):
        if radius lte 0:
            // Signal creation failure if arguments are invalid
            return (null, (1, "Radius must be positive"))
        this.radius is radius
        this.color is color
        // Implicitly returns (this, (0, "")) for successful initialization

    def get_radius(this):
        return (this.radius, (0, ""))

    def set_radius(this, new_radius):
        if new_radius lte 0:
            // Return 'this' as result even on error, to allow chaining if desired, but signal error.
            return (this, (1, "Radius must be positive"))
        this.radius is new_radius
        return (this, (0, "")) // Method returns 'this' by convention on successful modification
```

### Accessing Attributes

Instance attributes are accessed using dot notation (`.`) on an object instance. Class attributes are accessed via the class name.

```coral
(my_circle, circle_err) is Circle(5.0, "Red")

if circle_err.0 eq 0:
    (radius_val, radius_err) is my_circle.get_radius()
    if radius_err.0 eq 0:
        print('The circle''s radius is: {radius_val}')

    // Direct attribute access is generally for attributes not prefixed with '_'
    print('Circle color is: {my_circle.color}')
    print('Value of PI from Circle class: {Circle.PI}')

    (set_op_result, set_err) is my_circle.set_radius(6.0)
    if set_err.0 eq 0:
        (new_radius_val, _) is my_circle.get_radius() // '_' to ignore error tuple if confident
        print('The circle''s new radius is: {new_radius_val}')
    else:
        print('Error setting radius: {set_err.1}')
else:
    print('Error creating circle: {circle_err.1}')
```

## 3. Methods

Methods are functions defined within a class that implement the behavior of objects.

### Defining Methods

Methods are defined using `def` inside a class. The first parameter is conventionally `this` for instance methods, referring to the object instance itself. All methods return a tuple `(result, error_details)`. Instance methods that primarily perform an action or modify `this` often implicitly or explicitly return `(this, (0, ""))` on success.

```coral
class Rectangle:
    def init(this, width, height):
        if width lte 0 or height lte 0:
            return (null, (1, "Dimensions must be positive"))
        this.width is width
        this.height is height
        // Implicitly returns (this, (0,""))

    def area(this):
        calculated_area is this.width * this.height
        return (calculated_area, (0, ""))

    def scale(this, factor):
        if factor lte 0:
            return (this, (2, "Factor must be positive"))
        this.width is this.width * factor
        this.height is this.height * factor
        return (this, (0, ""))

    def is_square(this):
        is_sq_val is this.width eq this.height
        return (is_sq_val, (0, ""))
```

### Calling Methods

Call methods on an object using dot notation. Always handle the returned tuple to check for errors and retrieve the result.

```coral
(rect, rect_err) is Rectangle(10.0, 5.0)

if rect_err.0 eq 0:
    (area_val, area_err) is rect.area()
    if area_err.0 eq 0: print('Area: {area_val}')

    (is_sq_val, sq_err) is rect.is_square()
    if sq_err.0 eq 0: print('Is square? {is_sq_val}')

    (scale_op_this, scale_err) is rect.scale(2.0) // 'scale_op_this' will be 'rect' itself
    if scale_err.0 eq 0:
        (new_area_val, _) is rect.area()
        print('New Area after scaling: {new_area_val}')
    else:
        print('Error scaling: {scale_err.1}')
else:
    print('Error creating rectangle: {rect_err.1}')
```

## 4. Constructors (Initializers)

The `init` method serves as the constructor. It's automatically called when an object is created. It should initialize the object's state and return `(this, (0, ""))` on success, or an error tuple `(null, (error_id, "description"))` if initialization fails.

```coral
class Vehicle:
    def init(this, number_of_wheels, model_name):
        this.number_of_wheels is number_of_wheels
        this.model_name is model_name
        print('Vehicle "{this.model_name}" with {this.number_of_wheels} wheels created.')
        // Implicitly returns (this, (0, ""))

(car, car_err) is Vehicle(4, "SedanX")
if car_err.0 neq 0: print('Car creation failed: {car_err.1}')
```

## 5. Encapsulation

Encapsulation is the bundling of data (attributes) and methods that operate on that data within a single unit (a class). It often involves restricting direct access to some of an object's components to protect internal state.

In Coral, members (attributes and methods) are **public by default**. To signal that an attribute or method is intended for internal use within the class and not as part of its public API, Coral follows the convention of prefixing its name with an underscore (`_`). While the language might not strictly enforce privacy for such members, this convention is a strong indicator to developers.

```coral
class BankAccount:
    def init(this, holder, initial_deposit):
        this.account_holder is holder
        if initial_deposit lt 0:
            return (null, (1, "Initial deposit cannot be negative"))
        this._balance is initial_deposit // Internal attribute by convention
        // return (this, (0,"")) implicit success

    def deposit(this, amount):
        if amount lte 0:
            return (this, (2, "Deposit amount must be positive")) // Return 'this' even on error
        this._balance is this._balance + amount
        (log_res, log_err) is this._log_transaction('Deposited: {amount}')
        return (this, (0, ""))

    def withdraw(this, amount):
        if amount lte 0:
            return (false, (3, "Withdrawal amount must be positive")) // Explicit boolean result
        if this._balance gte amount:
            this._balance is this._balance - amount
            this._log_transaction('Withdrew: {amount}')
            return (true, (0, ""))
        else:
            this._log_transaction('Failed withdrawal attempt: {amount}')
            return (false, (4, "Insufficient funds"))

    def get_balance(this): // Public accessor for the balance
        return (this._balance, (0, ""))

    def _log_transaction(this, message): // Internal method by convention
        print('LOG: Account {this.account_holder} - {message}')
        return (this, (0, "")) // Internal methods also follow the tuple return

(my_acc, acc_err) is BankAccount("Alice", 100.0)
if acc_err.0 eq 0:
    my_acc.deposit(50.0) // Error handling for deposit call omitted for brevity
    (balance, bal_err) is my_acc.get_balance()
    if bal_err.0 eq 0: print('Alice''s balance: {balance}')
else:
    print('Account creation error: {acc_err.1}')
```

## 6. Inheritance

Inheritance allows a new class (subclass or derived class) to acquire the properties and behaviors (attributes and methods) of an existing class (superclass or base class). This promotes code reuse and establishes an "is-a" type hierarchy. Coral uses the `extends` keyword for inheritance.

### Syntax for Subclassing

```coral
class Animal:
    def init(this, name):
        this.name is name
        return (this, (0, ""))

    def speak(this):
        print('"{this.name}" makes a sound.')
        return (this, (0, ""))

    def describe(this):
        print('This is {this.name}.')
        return (this, (0, ""))

class Dog extends Animal: // Dog inherits from Animal
    BREED_UNKNOWN is "Unknown"

    def init(this, name, breed):
        // Call the superclass constructor. 'super.init' returns a tuple.
        (super_initialized_this, error) is super.init(name)
        if error.0 neq 0:
            return (null, error) // Propagate error from superclass init

        // 'this' is now the instance initialized by the superclass.
        // We can add subclass-specific attributes.
        this.breed is breed
        return (this, (0, "")) // Return 'this' (which is super_initialized_this)

    def speak(this): // Overriding the speak method from Animal
        print('"{this.name}" says Woof!')
        return (this, (0, ""))

    def fetch(this, item): // Adding a new method specific to Dog
        print('"{this.name}" is fetching the "{item}".')
        return (this, (0, ""))
```

### Calling Superclass Methods

Use the `super` keyword to call methods from the superclass, especially useful in overridden methods or subclass constructors. Calls like `super.method_name(...)` also return the standard `(result, error_details)` tuple.

```coral
(generic_animal, ga_err) is Animal("Creature")
if ga_err.0 eq 0: generic_animal.describe() // Error handling for describe call omitted

(my_dog, dog_err) is Dog("Buddy", "Golden Retriever")
if dog_err.0 eq 0:
    my_dog.describe()   // Inherited method call
    my_dog.speak()      // Overridden method call
    my_dog.fetch("ball") // Subclass-specific method call
    // Error handling for these method calls omitted for brevity
else:
    print('Dog creation error: {dog_err.1}')
```

## 7. Polymorphism

Polymorphism ("many forms") allows objects of different classes to be treated as objects of a common superclass through a uniform interface. When a method is called on such an object, the version specific to the object's actual class (the overridden version) is executed.

```coral
def make_animal_speak(animal_instance):
    // animal_instance can be an Animal, Dog, Cat, etc.
    (speak_res, speak_err) is animal_instance.speak() // Calls the appropriate 'speak'
    if speak_err.0 neq 0:
        print('Error making animal speak: {speak_err.1}')

class Cat extends Animal:
    def init(this, name):
        (super_this, error) is super.init(name)
        if error.0 neq 0: return (null, error)
        return (this, (0, "")) // 'this' is super_this

    def speak(this): // Override for Cat
        print('"{this.name}" says Meow!')
        return (this, (0, ""))

// Assuming successful creation for Dog, Cat, Animal instances (error handling omitted)
(dog_instance, _) is Dog("Rex", "German Shepherd")
(cat_instance, _) is Cat("Whiskers")
(animal_instance, _) is Animal("General")

animals_list is [dog_instance, cat_instance, animal_instance] // List of diverse Animal types

iter animals_list:
    make_animal_speak(it) // 'it' is the current animal instance
// Output demonstrates polymorphism:
// "Rex" says Woof!
// "Whiskers" says Meow!
// "General" makes a sound.
```
Coral's consistent method call and return system, combined with its class structure, naturally supports polymorphism. The compiler and runtime ensure the correct method is dispatched.

## 8. Special Methods (e.g., String Representation)

Coral may provide "special methods" (sometimes called "magic methods") that you can define in your classes to allow objects to integrate with built-in operations or to provide standard behaviors. A common example is a method for generating a string representation of an object.

Let's assume Coral uses `to_string()` for this purpose by convention.

```coral
class Point:
    def init(this, x, y):
        this.x is x
        this.y is y
        // return (this, (0, "")) implicit

    // Special method for string representation
    def to_string(this):
        representation is 'Point(x={this.x}, y={this.y})'
        return (representation, (0, "")) // Returns the string as the result

(p1, p1_err) is Point(10, 20)

if p1_err.0 eq 0:
    (p1_str_val, p1_str_err) is p1.to_string() // Call the special method
    if p1_str_err.0 eq 0:
        print(p1_str_val) // Output: Point(x=10, y=20)
    else:
        print('Error converting p1 to string: {p1_str_err.1}')
else:
    print('Error creating p1: {p1_err.1}')
```
If the `print()` function or string interpolation contexts are designed to automatically look for and use a `to_string()` method when an object is provided, this can make objects behave more intuitively.

This revised structure aligns OOP in Coral with its core syntax and error handling philosophy, aiming for both power and clarity. The Coral compiler ensures that these high-level OOP constructs are translated into efficient operations.
