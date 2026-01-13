# Large Document - Performance Testing.

This is a large markdown document designed for comprehensive performance testing
of the Marco parser. It contains multiple sections with various Markdown elements.

## Table of Contents

1. Introduction
2. Syntax Examples
3. Code Samples
4. Lists and Nested Structures
5. Links and References
6. Extended Content
7. Conclusion

---

## 1. Introduction

Performance is crucial for a markdown editor. This document tests the parser's
ability to handle larger documents with complex structures efficiently.

### Why Performance Matters

When editing documents, users expect:

* **Instant feedback** - No lag when typing
* **Smooth scrolling** - No stuttering when navigating
* **Fast preview** - Real-time HTML rendering
* **Responsive UI** - Quick response to all actions

The Marco editor aims to provide **sub-millisecond** parsing for documents
of this size, ensuring a fluid editing experience.

---

## 2. Syntax Examples

### Emphasis and Strong

Regular text, *italic text*, **bold text**, ***bold italic text***.

You can also use underscores: _italic_ and __bold__ and ___bold italic___.

### Inline Code and Code Spans

Use `inline code` for short snippets. Multiple backticks work too: ``code with `backtick` ``.

### Links

[Simple link](https://example.com)
[Link with title](https://example.com "Example Title")
<https://autolink.example.com>
<user@example.com>

### Images

![Alt text](https://example.com/image.png)
![Alt text with title](https://example.com/image.png "Image Title")

---

## 3. Code Samples

### Rust Example

```rust
use std::collections::HashMap;

fn main() {
    let mut scores = HashMap::new();
    
    scores.insert(String::from("Blue"), 10);
    scores.insert(String::from("Yellow"), 50);
    
    for (key, value) in &scores {
        println!("{}: {}", key, value);
    }
}
```

### Python Example

```python
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)

# Generate first 10 numbers
for i in range(10):
    print(f"F({i}) = {fibonacci(i)}")
```

### JavaScript Example

```javascript
class Person {
    constructor(name, age) {
        this.name = name;
        this.age = age;
    }
    
    greet() {
        console.log(`Hello, my name is ${this.name}`);
    }
}

const person = new Person("Alice", 30);
person.greet();
```

---

## 4. Lists and Nested Structures

### Unordered Lists

* First level item 1
* First level item 2
  * Second level item 2.1
  * Second level item 2.2
    * Third level item 2.2.1
    * Third level item 2.2.2
  * Second level item 2.3
* First level item 3

### Ordered Lists

1. First item
2. Second item
   1. Nested item 2.1
   2. Nested item 2.2
      1. Double nested 2.2.1
      2. Double nested 2.2.2
   3. Nested item 2.3
3. Third item

### Mixed Lists

1. Ordered item 1
   * Unordered nested 1.1
   * Unordered nested 1.2
2. Ordered item 2
   * Unordered nested 2.1
     1. Ordered double nested 2.1.1
     2. Ordered double nested 2.1.2

### Task Lists (if supported)

- [x] Completed task
- [ ] Incomplete task
- [x] Another completed task

---

## 5. Links and References

[Link reference definition]: https://example.com "Example"
[Another reference]: https://another.example.com

You can use [link references][Link reference definition] throughout
your document, and they'll all point to the [same destination][Link reference definition].

Here's [another reference link][Another reference] for good measure.

---

## 6. Extended Content

### Blockquotes

> This is a blockquote.
> It can span multiple lines.
>
> > And it can be nested!
> >
> > > Even multiple levels deep.
>
> Back to the first level.

> Another blockquote with **bold** and *italic* text,
> as well as `inline code`.

### Horizontal Rules

First section above the rule.

---

Second section between rules.

***

Third section after rules.

### HTML Blocks (if supported)

<div class="custom">
  <p>HTML blocks are preserved in the output.</p>
  <p>They can contain multiple elements.</p>
</div>

### Inline HTML

This paragraph contains <span class="highlight">inline HTML</span> mixed with
regular Markdown **bold** and *italic* text.

---

## 7. Conclusion

### Performance Metrics

This document should be parsed in:
* Less than 1ms for small changes
* Less than 5ms for full re-parse
* Less than 2ms for HTML rendering

### Summary

The Marco editor's parser handles documents of this size efficiently, providing:

1. **Fast parsing** - Optimized nom-based parser
2. **Incremental updates** - Only reparse changed sections
3. **Efficient rendering** - Minimal DOM updates
4. **Memory efficiency** - Smart caching strategies

---

## Additional Content Sections

### Section A - Lorem Ipsum

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor
incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis
nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.

Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu
fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in
culpa qui officia deserunt mollit anim id est laborum.

### Section B - More Content

Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium
doloremque laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore
veritatis et quasi architecto beatae vitae dicta sunt explicabo.

Nemo enim ipsam voluptatem quia voluptas sit aspernatur aut odit aut fugit, sed
quia consequuntur magni dolores eos qui ratione voluptatem sequi nesciunt.

### Section C - Even More

At vero eos et accusamus et iusto odio dignissimos ducimus qui blanditiis
praesentium voluptatum deleniti atque corrupti quos dolores et quas molestias
excepturi sint occaecati cupiditate non provident.

Similique sunt in culpa qui officia deserunt mollitia animi, id est laborum et
dolorum fuga. Et harum quidem rerum facilis est et expedita distinctio.

### Section D - Final Section

Nam libero tempore, cum soluta nobis est eligendi optio cumque nihil impedit quo
minus id quod maxime placeat facere possimus, omnis voluptas assumenda est,
omnis dolor repellendus.

Temporibus autem quibusdam et aut officiis debitis aut rerum necessitatibus saepe
eveniet ut et voluptates repudiandae sint et molestiae non recusandae.

---

**End of Document** - This large document provides comprehensive testing data
for performance benchmarking of the Marco markdown parser.
