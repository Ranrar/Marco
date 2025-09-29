# Code Block Syntax Highlighting Test

This document tests the syntax highlighting feature for code blocks.

## Python Code

```python
def fibonacci(n):
    """Calculate the nth Fibonacci number."""
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)
# Test with different values
for i in range(10):
    print(f"F({i}) = {fibonacci(i)}")
```

## Rust Code

```rust
use std::collections::HashMap;
fn main() {
    let mut map = HashMap::new();
    map.insert("hello", "world");
    match map.get("hello") {
        Some(value) => println!("Found: {}", value),
        None => println!("Not found"),
    }
}
```

## JavaScript Code

```javascript
const users = [
    { name: "Alice", age: 30 },
    { name: "Bob", age: 25 },
    { name: "Charlie", age: 35 }
];
const adults = users
    .filter(user => user.age >= 18)
    .map(user => user.name);
console.log("Adults:", adults);
```

## HTML Code

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Sample Page</title>
</head>
<body>
    <h1>Welcome to Marco!</h1>
    <p>This is a <em>markdown</em> editor with <strong>syntax highlighting</strong>.</p>
</body>
</html>
```

## No Language Specified

```
This is a plain code block without language specification.
It should still be displayed in a monospace font
but without syntax highlighting colors.
```