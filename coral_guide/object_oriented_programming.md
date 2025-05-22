# Object-Oriented Programming in Coral

Coral seamlessly integrates Object-Oriented Programming (OOP) principles, allowing developers to structure their code around "objects" that combine data (attributes) and behavior (methods). This approach promotes modularity, reusability, and a clear organization of complex software. Coral's OOP aims to blend the intuitive ease of use found in Python with the benefits of strong typing and, where appropriate, the explicitness seen in languages like Rust.

## 1. Classes and Objects

### Defining Classes

Classes are blueprints for creating objects. You define a class using the `class` keyword.

```coral
class Greeter {
    // Attributes (fields/properties) and methods will be defined here
    greeting_prefix: String;

    fn init() {
        this.greeting_prefix = "Hello"; // Default prefix
    }

    fn greet(this, name: String) {
        print(f"{this.greeting_prefix}, {name}!");
    }
}
```

### Creating Objects (Instances)

An object is an instance of a class. You create objects by calling the class name as if it were a function. If the class has an `init` method (constructor), you can pass arguments to it.

```coral
// Create an instance of the Greeter class
let greeter_obj = Greeter();

// Call a method on the object
greeter_obj.greet("World"); // Output: Hello, World!
```

## 2. Attributes (Fields/Properties)

Attributes are variables that store data associated with an object. They are defined within the class, typically with type annotations for clarity and safety.

### Defining Attributes

```coral
class Circle {
    radius: Float;         // Attribute to store the radius
    color: String;         // Attribute to store the color

    // Constructor to initialize attributes
    fn init(r: Float, c: String) {
        this.radius = r;    // 'this' refers to the current instance
        this.color = c;
    }
}
```
In Coral, `this` is used to refer to the current instance of the class (similar to `self` in Python or `this` in Java/C++/JavaScript).

### Accessing Attributes

You access an object's attributes using dot notation (`.`).

```coral
let my_circle = Circle(5.0, "Red");

print(f"The circle's radius is: {my_circle.radius}"); // Output: 5.0
print(f"The circle's color is: {my_circle.color}");   // Output: Red

my_circle.radius = 6.0; // Attributes are generally mutable by default if declared with 'let'
print(f"The circle's new radius is: {my_circle.radius}"); // Output: 6.0
```

## 3. Methods

Methods are functions defined within a class that operate on the data (attributes) of an object.

### Defining Methods

Methods are defined like functions using the `fn` keyword inside a class. The first parameter of a method is typically `this` (or `self`), which refers to the object instance itself.

```coral
class Rectangle {
    width: Float;
    height: Float;

    fn init(w: Float, h: Float) {
        this.width = w;
        this.height = h;
    }

    // Method to calculate the area
    fn area(this) -> Float {
        return this.width * this.height;
    }

    // Method to scale the rectangle
    fn scale(this, factor: Float) {
        this.width *= factor;
        this.height *= factor;
    }

    // Method to check if the rectangle is a square
    fn is_square(this) -> Boolean {
        return this.width == this.height;
    }
}
```

### Calling Methods

You call methods on an object using dot notation, similar to accessing attributes.

```coral
let rect = Rectangle(10.0, 5.0);

print(f"Area: {rect.area()}");          // Output: Area: 50.0
print(f"Is square? {rect.is_square()}"); // Output: Is square? false

rect.scale(2.0);
print(f"New Area after scaling: {rect.area()}"); // Output: New Area after scaling: 200.0
```

## 4. Constructors (Initializers)

A constructor is a special method that gets called when an object is created. It's used to initialize the object's attributes. In Coral, the constructor is conventionally named `init`.

```coral
class Vehicle {
    number_of_wheels: Integer;
    model_name: String;

    // Constructor
    fn init(wheels: Integer, model: String) {
        this.number_of_wheels = wheels;
        this.model_name = model;
        print(f"{this.model_name} with {this.number_of_wheels} wheels created.");
    }
}

// When you create an object, the 'init' method is automatically called.
let car = Vehicle(4, "SedanX");     // Output: SedanX with 4 wheels created.
let bike = Vehicle(2, "Roadster"); // Output: Roadster with 2 wheels created.
```
The arguments passed during object creation are forwarded to the `init` method.

## 5. Encapsulation

Encapsulation is the bundling of data (attributes) and methods that operate on the data within a single unit (a class). It also involves restricting direct access to some of an object's components.

In Coral, members (attributes and methods) are **public by default**, meaning they can be accessed from outside the class.

For internal attributes or methods that are not intended to be part of the public API of the class, Coral encourages a convention similar to Python: prefixing the name with an underscore (`_`).

```coral
class BankAccount {
    account_holder: String;
    _balance: Float; // Internal attribute by convention

    fn init(holder: String, initial_deposit: Float) {
        this.account_holder = holder;
        this._balance = initial_deposit;
    }

    fn deposit(this, amount: Float) {
        if amount > 0 {
            this._balance += amount;
            this._log_transaction(f"Deposited: {amount}");
        }
    }

    fn withdraw(this, amount: Float) -> Boolean {
        if amount > 0 && amount <= this._balance {
            this._balance -= amount;
            this._log_transaction(f"Withdrew: {amount}");
            return true;
        }
        this._log_transaction(f"Failed withdrawal attempt: {amount}");
        return false;
    }

    fn get_balance(this) -> Float { // Public accessor for the balance
        return this._balance;
    }

    // Internal method by convention
    fn _log_transaction(this, message: String) {
        print(f"LOG: Account {this.account_holder} - {message}");
    }
}

let my_acc = BankAccount("Alice", 100.0);
my_acc.deposit(50.0);
print(f"Alice's balance: {my_acc.get_balance()}"); // Output: 150.0

// While '_balance' can technically be accessed directly due to being public by default,
// the '_' indicates it's for internal use and direct modification is discouraged.
// print(my_acc._balance); // Possible, but not recommended practice
```
While the underscore is a convention and doesn't provide strict privacy like `private` keywords in some languages, it aligns with Coral's goal of Python-like ease of use while signaling intent to other developers. Future versions of Coral might introduce stricter privacy controls like `pub` and `priv` keywords for developers who prefer more Rust-like explicitness in access control.

## 6. Inheritance

Inheritance allows a class (subclass or derived class) to inherit attributes and methods from another class (superclass or base class). This promotes code reuse and establishes an "is-a" relationship.

Coral uses the `extends` keyword for inheritance.

### Syntax for Subclassing

```coral
class Animal {
    name: String;
    age: Integer;

    fn init(name: String, age: Integer) {
        this.name = name;
        this.age = age;
    }

    fn speak(this) {
        print("The animal makes a sound.");
    }

    fn describe(this) {
        print(f"This is {this.name}, {this.age} years old.");
    }
}

// Dog is a subclass of Animal
class Dog extends Animal {
    breed: String;

    // Constructor for Dog
    fn init(name: String, age: Integer, breed: String) {
        super.init(name, age); // Call the superclass (Animal) constructor
        this.breed = breed;
    }

    // Overriding the speak method from Animal
    fn speak(this) {
        print(f"{this.name} says Woof!");
    }

    // Adding a new method specific to Dog
    fn fetch(this, item: String) {
        print(f"{this.name} is fetching the {item}.");
    }
}
```

### Overriding Methods

A subclass can provide its own specific implementation of a method that is already defined in its superclass. This is called method overriding. See the `speak` method in the `Dog` class above.

### Calling Superclass Methods

To call a method from the superclass (especially useful in overridden methods or constructors), Coral uses the `super` keyword.

`super.method_name(arguments)` calls the method from the parent class.
`super.init(arguments)` calls the constructor of the parent class.

```coral
let generic_animal = Animal("Creature", 5);
generic_animal.describe(); // Output: This is Creature, 5 years old.
generic_animal.speak();    // Output: The animal makes a sound.

let my_dog = Dog("Buddy", 3, "Golden Retriever");
my_dog.describe();         // Output: This is Buddy, 3 years old. (Inherited method)
my_dog.speak();            // Output: Buddy says Woof! (Overridden method)
my_dog.fetch("ball");      // Output: Buddy is fetching the ball. (Subclass-specific method)
```

## 7. Polymorphism

Polymorphism ("many forms") means that objects of different classes can be treated as objects of a common superclass. When a method is called on such an object, the version specific to the object's actual class is executed.

```coral
fn make_animal_speak(animal: Animal) {
    animal.speak(); // This will call the appropriate 'speak' method
}

class Cat extends Animal {
    fn init(name: String, age: Integer) {
        super.init(name, age);
    }

    fn speak(this) { // Override for Cat
        print(f"{this.name} says Meow!");
    }
}

let animals = [
    Dog("Rex", 4, "German Shepherd"),
    Cat("Whiskers", 2),
    Animal("General", 1)
]; // Assuming a list/array syntax for Coral

for animal_obj in animals:
    make_animal_speak(animal_obj);
    // For Dog: Rex says Woof!
    // For Cat: Whiskers says Meow!
    // For Animal: General makes a sound.

    // You can also call methods directly
    // animal_obj.speak();
```
In this example, `make_animal_speak` takes an `Animal` type. However, you can pass `Dog` or `Cat` objects to it because they are subclasses of `Animal`. The correct `speak` method is called for each object.

## 8. Special Methods (e.g., String Representation)

Coral, like Python, may provide special methods that you can define in your classes to integrate with built-in operations or to provide standard behaviors. One common example is a method for string representation.

Let's assume Coral uses `to_string()` for this purpose.

```coral
class Point {
    x: Integer;
    y: Integer;

    fn init(x: Integer, y: Integer) {
        this.x = x;
        this.y = y;
    }

    // Special method for string representation
    fn to_string(this) -> String {
        return f"Point(x={this.x}, y={this.y})";
    }
}

let p1 = Point(10, 20);
let p2 = Point(5, -3);

// If 'print' is designed to automatically call 'to_string()' if available:
print(p1); // Output: Point(x=10, y=20)
print(p2); // Output: Point(x=5, y=-3)

// Or, you might need to call it explicitly:
// print(p1.to_string());
```
Other special methods could exist for comparison (`equals()`, `compare_to()`), hashing, etc., making objects behave more intuitively within the language's ecosystem.

This concludes our introduction to Object-Oriented Programming in Coral. By leveraging classes, inheritance, and polymorphism, you can build robust and well-organized applications.
