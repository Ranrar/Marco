# Marco Extensions Test

This document tests all the custom Marco-specific markdown extensions and features.

## Admonitions

### Admonitions with Custom Titles

:::note[Custom Note Title]
This note has a custom title instead of the default "Note".
:::

:::tip[Pro Tip]
This tip has a custom title.
:::

:::warning[Important Notice]
This warning has a custom title with **formatting**.
:::

:::danger[Critical Issue]
This danger block has a custom title.
:::

:::info[Additional Information]
This info block has a custom title.
:::

### Emoji Admonitions

:::[🔥] Fire Warning
This is a custom admonition with a fire emoji icon.
:::

:::[✨] Sparkle Note
This is a custom admonition with a sparkle emoji.
:::

:::[🚀] Rocket Tip
This is a custom admonition with a rocket emoji.
You can use any emoji as the icon.
:::

:::[⚠️] Alert
Custom warning with warning emoji.
:::

:::[💡] Idea
Custom tip with lightbulb emoji.
:::

### Nested Admonitions

:::note[Outer Note]
This is the outer admonition.

:::tip[Inner Tip]
This is a nested tip inside the note.
:::

Back to the outer note content.
:::

:::warning
Outer warning with different nesting.

:::danger
Inner danger block.

:::info
Even deeper info block.
:::

Back to danger level.
:::

Back to warning level.
:::


## Tables

### Tables without Headers

|--------|--------|--------|
| Data 1 | Data 2 | Data 3 |
| Data 4 | Data 5 | Data 6 |

## User Mentions

### Basic User Mentions

Mention users on various platforms:
@john[twitter]
@alice[github]
@bob[linkedin]
@charlie[discord]
@diana[slack]

### User Mentions with Display Names

Enhanced mentions with real names:
@john[twitter](John Doe)
@alice[github](Alice Smith)  
@bob[linkedin](Bob Johnson)
@charlie[discord](Charlie Brown)
@diana[slack](Diana Prince)

## Tab Blocks

### Basic Tab Block

:::tab
@tab JavaScript
```javascript
function hello() {
    console.log("Hello from JavaScript!");
}
```

@tab Python
```python
def hello():
    print("Hello from Python!")
```

@tab Rust
```rust
fn hello() {
    println!("Hello from Rust!");
}
```
:::

### Tab Block with Title

:::tab Code Examples
@tab Frontend
```javascript
// React component
function Welcome() {
    return <h1>Hello World!</h1>;
}
```

@tab Backend
```python
# Flask app
from flask import Flask
app = Flask(__name__)

@app.route('/')
def hello():
    return "Hello World!"
```

@tab Database
```sql
-- SQL query
SELECT * FROM users 
WHERE active = true
ORDER BY created_at DESC;
```
:::

### Mixed Content Tab Block

:::tab Documentation
Default content before any tabs.
This appears in the default tab.

@tab Installation
## Installation Steps

1. Download the software
2. Run the installer
3. Configure settings

@tab Configuration
### Config File

```json
{
    "theme": "dark",
    "language": "en",
    "auto_save": true
}
```

@tab Troubleshooting
**Common Issues:**

- Issue 1: Check permissions
- Issue 2: Restart service  
- Issue 3: Clear cache

More default content at the end.
:::

### Inline Tasks

Regular paragraph with [x] inline completed task here.
Another paragraph with [ ] incomplete inline task (user: pending).

## Definition Lists

Term 1
: Definition for term 1
: Alternative definition for term 1

Term 2
: Definition for term 2 with **formatting**
: Another definition with `code`

Complex Term with **Formatting**
: Definition with [link](https://example.com)
: Definition with math $x = y + z$

## YouTube Embeds

### Basic YouTube Links

[Video Title](https://youtu.be/dQw4w9WgXcQ)
[Another Video](https://www.youtube.com/watch?v=dQw4w9WgXcQ)
[HTTP YouTube](http://youtu.be/shortcode123)
[HTTP Full YouTube](http://www.youtube.com/watch?v=fullcode456)

## Footnotes

### Inline Footnotes

This text has an inline footnote^[This is an inline footnote with content].
Another sentence with footnote^[Another inline footnote here].


Other functions Marco can do.
Generate TOC
Automatic generate TOC from ID
Bookmark a lines in a document
Export to A4 or US Letter
Start with a predefined markdown template