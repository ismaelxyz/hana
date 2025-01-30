# Haru Grammar

In programming, the ease of understanding and intuitiveness of a language are fundamental characteristics that determine its adoption and efficient use. A programming language is considered easy to understand and intuitive when it has a clear and concise syntax, complete and accessible documentation, and an active community that provides support and resources.

The syntax of a Haru is designed to be logical and predictable, allowing developers to write and read code efficiently. It should be notable for its clean syntax, which makes it easy to understand and reduces the possibility of errors.


## Lexical Structure

### Keywords
Haru has a set of reserved keywords that cannot be used as identifiers. Some of these keywords include:

- `fn`
- `end`
- `record`
- `use`
- `if`
- `else`
- `while`
- `for`
- `break`
- `return`
- `this`
- `nil`
- `true`
- `false`



### Identifiers
Identifiers in Haru are names used to identify variables, methods, recordes, modules, and other entities. They must start with a letter or an underscore, followed by letters, digits, or underscores.

```go
var_name := 1
fnName()
RecordName.value = 3
CONST_NAME := "const str"
```

### Comments
Haru has two types of comments. A comment on a line is denoted by the symbol `//`. Everything following `//` on that line is considered a comment and is ignored by the interpreter or compiler. In addition, multi-line comments are denoted by `/*` to open the comment and `*/` to close it. All text between `/*` and `*/` is considered a comment and is ignored by the interpreter or compiler.


```rust
// This is a comment

/*
    This is a multi-line comment
*/

/*
    /*
        This is a multi-line comment... again
    */
*/
```

## Syntax

### Variables
Variables in Haru require explicit declaration. They can be declared using the `let` keyword, or using the short declaration operator `:=` for local variables. 

```rust
let x = 10
name := "Hana"
```

### Methods
Methods in Haru are defined using the `fn` keyword, followed by the method name and an optional parameter list. The method body is enclosed between `fn` and `end`.

```ruby
fn greet(name)
  print("Hello, {{name}}!")
end
```

### Classes and Modules
Classes and modules are defined using the `record` and `module` keywords, respectively. The body of the record or module is enclosed between `record`/`module` and `end`.

```ruby
record Person
  def constructor(name, age)
    this.name = name
    this.age = age
  end

  def display
    print("Name: {{this.name}}, Age: {{this.age}}")
  end
end

module Greetings
  def greet()
    print("Hello!")
  end
end
```

### Control Structures
Haru provides several control structures for conditional execution and iteration.

#### Conditional Statements
Conditional statements include `if`, `else`.

```ruby
if x > 10
  print("x is greater than 10")
else if x == 10
  print("x is 10")
else
  print("x is less than 10")
end
```

#### Loops
Loops include `while`, `for`.

```ruby
while x < 10
  print(x)
  x += 1
end
```

### Haru Operators

Haru supports a variety of operators, including:

- **Arithmetic Operators**
  - `+` (addition)
  - `-` (subtraction)
  - `*` (multiplication)
  - `/` (division)
  - `%` (modulus)

- **Comparison Operators**
  - `==` (equal to)
  - `!=` (not equal to)
  - `>` (greater than)
  - `<` (less than)
  - `>=` (greater than or equal to)
  - `<=` (less than or equal to)

- **Logical Operators**
  - `&&` (logical AND)
  - `||` (logical OR)
  - `!` (logical NOT)

- **Bitwise Operators**
  - `&` (bitwise AND)
  - `|` (bitwise OR)
  - `^` (bitwise XOR)
  - `!` (bitwise NOT)
  - `<<` (left shift)
  - `>>` (right shift)

- **Assignment Operators**
  - `=` (assignment)
  - `+=` (addition assignment)
  - `-=` (subtraction assignment)
  - `*=` (multiplication assignment)
  - `/=` (division assignment)
  - `%=` (modulus assignment)
  - `&=` (bitwise AND assignment)
  - `|=` (bitwise OR assignment)
  - `^=` (bitwise XOR assignment)
  - `<<=` (left shift assignment)
  - `>>=` (right shift assignment)

- **Other Operators**
  - `?` (ternary operator )
  - `..` (range)
  - `..=` (inclusive range)
  - `::` (path separator)
  - `:` (function return type, then)