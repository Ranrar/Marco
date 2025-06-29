# Marco Markdown Editor - Complete Feature Test

This document showcases **all** features available in Marco, the modern markdown editor built with Rust and GTK4.

## Table of Contents
- [Basic Markdown Syntax](#basic-markdown-syntax)
- [Extended Markdown Features](#extended-markdown-features)  
- [Advanced Markdown Hacks](#advanced-markdown-hacks)
- [Code Language Support](#code-language-support)
- [GitHub-Style Admonitions](#github-style-admonitions)
- [Interactive Elements](#interactive-elements)
- [Theme and UI Features](#theme-and-ui-features)
- [Keyboard Shortcuts](#keyboard-shortcuts)
- [Multi-Language Support](#multi-language-support)

---

## Basic Markdown Syntax

### Headers (Ctrl+1-6)
# Heading 1
## Heading 2  
### Heading 3
#### Heading 4
##### Heading 5
###### Heading 6

### Text Formatting
**Bold text** (Ctrl+B)  
*Italic text* (Ctrl+I)  
`Inline code` (Ctrl+`)  
~~Strikethrough~~ (Ctrl+U)

### Lists

#### Unordered List (Ctrl+Shift+8)
- First item
- Second item
  - Nested item
  - Another nested item
- Third item

#### Ordered List (Ctrl+Shift+7)
1. First numbered item
2. Second numbered item
   1. Nested numbered item
   2. Another nested item
3. Third numbered item

### Blockquotes (Ctrl+Shift+.)
> This is a blockquote
> 
> It can span multiple lines
> 
> > And can be nested

### Links and Images (Ctrl+K)
[Marco Repository](https://github.com/example/marco)  
![Sample Image](https://via.placeholder.com/300x200?text=Sample+Image)

### Horizontal Rule
---

## Extended Markdown Features

### Text Formatting Extensions
==Highlighted text==  
H~2~O (Subscript)  
E=mc^2^ (Superscript)

### Task Lists
- [x] Completed task
- [ ] Pending task
- [x] Another completed task
- [ ] Another pending task

### Footnotes
This text has a footnote[^1].

[^1]: This is the footnote content.

### Definition Lists
Term 1
: Definition for term 1

Term 2
: Definition for term 2
: Alternative definition for term 2

### Tables
| Feature | Status | Priority |
|---------|--------|----------|
| Live Preview | ✅ Complete | High |
| Syntax Highlighting | ✅ Complete | High |
| Multi-language UI | ✅ Complete | Medium |
| Export Features | ❌ Planned | Low |

## Advanced Markdown Hacks

### Text Styling Hacks
<u>Underlined text</u>

<center>Centered text</center>

<span style="color: red;">Red colored text</span>
<span style="color: blue;">Blue colored text</span>
<span style="color: green;">Green colored text</span>

### Indented Text
&nbsp;&nbsp;&nbsp;&nbsp;This text is indented using HTML entities.

### Comments (Hidden in Preview)
<!-- This is a hidden comment that won't appear in the preview -->

### Enhanced Images with Size
<img src="https://via.placeholder.com/150x100" width="150" height="100" alt="Sized image">

### Enhanced Images with Captions
<figure>
<img src="https://via.placeholder.com/200x150" alt="Image with caption">
<figcaption>This is an image caption</figcaption>
</figure>

### Links with Targets
<a href="https://github.com" target="_blank">External link (opens in new tab)</a>

### YouTube Video Embed
<iframe width="560" height="315" src="https://www.youtube.com/embed/dQw4w9WgXcQ" frameborder="0" allowfullscreen></iframe>

### HTML Entities and Special Symbols
&copy; &reg; &trade; &nbsp; &amp; &lt; &gt; &quot; &#39;

&larr; &rarr; &uarr; &darr; &harr; &crarr;

&#8364; &#8364; &#8482; &#174; &#169;

## Code Language Support

### Top 10 Programming Languages

#### JavaScript
```javascript
function greetUser(name) {
    return `Hello, ${name}! Welcome to Marco.`;
}

const user = "Developer";
console.log(greetUser(user));
```

#### Python
```python
def calculate_fibonacci(n):
    if n <= 1:
        return n
    return calculate_fibonacci(n-1) + calculate_fibonacci(n-2)

# Generate first 10 Fibonacci numbers
for i in range(10):
    print(f"F({i}) = {calculate_fibonacci(i)}")
```

#### Java
```java
public class HelloMarco {
    public static void main(String[] args) {
        System.out.println("Hello, Marco!");
        
        // Create a simple loop
        for (int i = 0; i < 5; i++) {
            System.out.println("Iteration: " + i);
        }
    }
}
```

#### TypeScript
```typescript
interface User {
    name: string;
    age: number;
    isActive: boolean;
}

class UserManager {
    private users: User[] = [];
    
    addUser(user: User): void {
        this.users.push(user);
    }
    
    getActiveUsers(): User[] {
        return this.users.filter(user => user.isActive);
    }
}
```

#### C#
```csharp
using System;
using System.Collections.Generic;
using System.Linq;

namespace MarcoDemo
{
    public class Program
    {
        public static void Main(string[] args)
        {
            var numbers = new List<int> { 1, 2, 3, 4, 5 };
            var evenNumbers = numbers.Where(n => n % 2 == 0);
            
            Console.WriteLine("Even numbers:");
            foreach (var num in evenNumbers)
            {
                Console.WriteLine(num);
            }
        }
    }
}
```

#### PHP
```php
<?php
class MarcoDemo {
    private $data = [];
    
    public function addData($key, $value) {
        $this->data[$key] = $value;
    }
    
    public function getData($key) {
        return isset($this->data[$key]) ? $this->data[$key] : null;
    }
}

$demo = new MarcoDemo();
$demo->addData("greeting", "Hello from Marco!");
echo $demo->getData("greeting");
?>
```

#### C++
```cpp
#include <iostream>
#include <vector>
#include <algorithm>

class NumberProcessor {
private:
    std::vector<int> numbers;
    
public:
    void addNumber(int num) {
        numbers.push_back(num);
    }
    
    void sortNumbers() {
        std::sort(numbers.begin(), numbers.end());
    }
    
    void printNumbers() {
        for (const auto& num : numbers) {
            std::cout << num << " ";
        }
        std::cout << std::endl;
    }
};

int main() {
    NumberProcessor processor;
    processor.addNumber(3);
    processor.addNumber(1);
    processor.addNumber(4);
    processor.sortNumbers();
    processor.printNumbers();
    return 0;
}
```

#### C
```c
#include <stdio.h>
#include <stdlib.h>

typedef struct {
    int x;
    int y;
} Point;

Point* createPoint(int x, int y) {
    Point* p = malloc(sizeof(Point));
    p->x = x;
    p->y = y;
    return p;
}

int main() {
    Point* p = createPoint(10, 20);
    printf("Point coordinates: (%d, %d)\n", p->x, p->y);
    free(p);
    return 0;
}
```

#### Go
```go
package main

import (
    "fmt"
    "time"
)

type User struct {
    Name     string
    Age      int
    JoinDate time.Time
}

func (u User) Greet() string {
    return fmt.Sprintf("Hello, I'm %s and I'm %d years old", u.Name, u.Age)
}

func main() {
    user := User{
        Name:     "Marco User",
        Age:      30,
        JoinDate: time.Now(),
    }
    
    fmt.Println(user.Greet())
    fmt.Printf("Joined on: %s\n", user.JoinDate.Format("2006-01-02"))
}
```

#### Rust
```rust
use std::collections::HashMap;

#[derive(Debug)]
struct Document {
    title: String,
    content: String,
    word_count: usize,
}

impl Document {
    fn new(title: &str, content: &str) -> Self {
        let word_count = content.split_whitespace().count();
        Document {
            title: title.to_string(),
            content: content.to_string(),
            word_count,
        }
    }
    
    fn summary(&self) -> String {
        format!("{}: {} words", self.title, self.word_count)
    }
}

fn main() {
    let doc = Document::new("Marco Test", "This is a test document for Marco editor");
    println!("{}", doc.summary());
    println!("{:#?}", doc);
}
```

### Markup and Data Languages

#### HTML
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Marco Test Page</title>
</head>
<body>
    <h1>Welcome to Marco</h1>
    <p>This is a test HTML document.</p>
</body>
</html>
```

#### CSS
```css
/* Marco Editor Styling */
.editor-container {
    display: flex;
    height: 100vh;
    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
}

.markdown-source {
    flex: 1;
    padding: 20px;
    background-color: #f8f9fa;
    border-right: 1px solid #e9ecef;
}

.markdown-preview {
    flex: 1;
    padding: 20px;
    background-color: white;
    overflow-y: auto;
}

@media (max-width: 768px) {
    .editor-container {
        flex-direction: column;
    }
}
```

#### JSON
```json
{
  "editor": {
    "name": "Marco",
    "version": "1.0.0",
    "features": [
      "live-preview",
      "syntax-highlighting",
      "multi-language",
      "themes"
    ],
    "languages": {
      "ui": ["en", "es", "fr", "de"],
      "programming": ["javascript", "python", "rust", "go", "java"]
    },
    "settings": {
      "theme": "system",
      "preview_mode": "html",
      "auto_save": false
    }
  }
}
```

#### XML
```xml
<?xml version="1.0" encoding="UTF-8"?>
<marco-config>
    <editor>
        <name>Marco Markdown Editor</name>
        <version>1.0.0</version>
    </editor>
    <features>
        <feature name="live-preview" enabled="true"/>
        <feature name="syntax-highlighting" enabled="true"/>
        <feature name="multi-language" enabled="true"/>
    </features>
    <languages>
        <ui>
            <language code="en">English</language>
            <language code="es">Español</language>
            <language code="fr">Français</language>
            <language code="de">Deutsch</language>
        </ui>
    </languages>
</marco-config>
```

#### SQL
```sql
-- Marco Database Schema
CREATE TABLE documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title VARCHAR(255) NOT NULL,
    content TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    word_count INTEGER DEFAULT 0
);

CREATE TABLE themes (
    id INTEGER PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    css_file VARCHAR(255),
    is_dark BOOLEAN DEFAULT FALSE
);

INSERT INTO themes (name, css_file, is_dark) VALUES
    ('Standard', 'standard.css', FALSE),
    ('GitHub', 'github.css', FALSE),
    ('Minimal', 'minimal.css', FALSE),
    ('Academic', 'academic.css', FALSE);

-- Query to get document statistics
SELECT 
    title,
    word_count,
    DATE(created_at) as created_date
FROM documents 
WHERE word_count > 100 
ORDER BY created_at DESC;
```

#### Bash
```bash
#!/bin/bash

# Marco build script
set -e

echo "Building Marco Markdown Editor..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Cargo not found. Please install Rust."
    exit 1
fi

# Build the project
echo "Compiling..."
cargo build --release

# Check if build was successful
if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo "Executable located at: target/release/marco"
else
    echo "❌ Build failed!"
    exit 1
fi

# Optional: Run tests
echo "Running tests..."
cargo test

echo "🎉 Marco is ready to use!"
```

#### YAML
```yaml
# Marco Configuration File
name: Marco Markdown Editor
version: 1.0.0
description: Modern markdown editor with live preview

features:
  - live_preview
  - syntax_highlighting
  - multi_language_ui
  - theme_support
  - keyboard_shortcuts

languages:
  ui_languages:
    - code: en
      name: English
      file: locales/en/main.yml
    - code: es
      name: Español
      file: locales/es/main.yml
    - code: fr
      name: Français
      file: locales/fr/main.yml
    - code: de
      name: Deutsch
      file: locales/de/main.yml

  programming_languages:
    - javascript
    - python
    - java
    - typescript
    - csharp
    - php
    - cpp
    - c
    - go
    - rust

themes:
  css_themes:
    - name: Standard
      file: css/standard.css
      description: Clean and modern
    - name: GitHub
      file: css/github.css
      description: GitHub-like styling
    - name: Minimal
      file: css/minimal.css
      description: Minimalist design
    - name: Academic
      file: css/academic.css
      description: Academic paper style

shortcuts:
  formatting:
    bold: "Ctrl+B"
    italic: "Ctrl+I"
    code: "Ctrl+`"
    link: "Ctrl+K"
  headings:
    h1: "Ctrl+1"
    h2: "Ctrl+2"
    h3: "Ctrl+3"
    h4: "Ctrl+4"
    h5: "Ctrl+5"
    h6: "Ctrl+6"
  lists:
    bullet: "Ctrl+Shift+8"
    numbered: "Ctrl+Shift+7"
    quote: "Ctrl+Shift+."
```

#### Markdown
```markdown
# Nested Markdown Example

This is a **markdown** document *inside* a markdown document!

## Features
- `Code highlighting`
- **Bold text**
- *Italic text*

### Code Example
\`\`\`rust
fn main() {
    println!("Hello from nested markdown!");
}
\`\`\`

> This is a blockquote in nested markdown
```

## GitHub-Style Admonitions

Marco supports GitHub-style admonitions with color-coded styling:

> [!NOTE]
> This is a note admonition. Use it to provide helpful information or context.

> [!TIP]
> This is a tip admonition. Use it to share useful advice or best practices.

> [!IMPORTANT]
> This is an important admonition. Use it to highlight critical information.

> [!WARNING]
> This is a warning admonition. Use it to alert users about potential issues.

> [!CAUTION]
> This is a caution admonition. Use it to warn about dangerous or destructive actions.

## Interactive Elements

### Tables with Dialog Support
You can create tables interactively using the **Format → Table...** menu:

| Feature | Keyboard Shortcut | Menu Location |
|---------|------------------|---------------|
| Bold | Ctrl+B | Insert → Bold |
| Italic | Ctrl+I | Insert → Italic |
| Code | Ctrl+` | Insert → Inline Code |
| Link | Ctrl+K | Insert → Link |
| Heading 1 | Ctrl+1 | Insert → Headings → Heading 1 |
| Heading 2 | Ctrl+2 | Insert → Headings → Heading 2 |
| Bullet List | Ctrl+Shift+8 | Insert → Unordered List |
| Numbered List | Ctrl+Shift+7 | Insert → Ordered List |
| Blockquote | Ctrl+Shift+. | Insert → Blockquote |

### Custom Task Lists
Create task lists with the **Format → Task List** menu:

- [x] Study Marco's features
- [x] Test basic markdown syntax
- [x] Try advanced features
- [ ] Test all keyboard shortcuts
- [ ] Try different themes
- [ ] Test multi-language switching

### Emoji Support 😀
Marco includes an emoji picker accessible via **Format → 😀 Emoji**:

📝 Document editing  
🎨 Theme customization  
🌍 Multi-language support  
⚡ Fast performance  
🔧 Extensible architecture  
💾 Auto-save (planned)  
📤 Export features (planned)  

### Definition Lists
Create definition lists using **Format → Definition List**:

Marco
: A modern markdown editor built with Rust and GTK4

Live Preview
: Real-time rendering of markdown as HTML in a split-pane interface

Syntax Highlighting
: Color-coded display of markdown syntax and code blocks

Multi-language
: Support for multiple interface languages (English, Spanish, French, German)

## Theme and UI Features

### Available CSS Themes
Marco includes 4 built-in CSS themes accessible via **View → CSS Style**:

1. **Standard** - Clean and modern default theme
2. **GitHub** - GitHub-like styling with familiar colors
3. **Minimal** - Minimalist design with reduced visual elements  
4. **Academic** - Academic paper style for formal documents

### UI Theme Options
Control the overall interface theme via **View → Theme**:

- **System** - Automatically follows OS dark/light preference
- **Light** - Forces light theme for editor and preview
- **Dark** - Forces dark theme for editor and preview

### View Modes
Switch between different preview modes via **View → Preview Mode**:

- **HTML Preview** - Rendered markdown with CSS styling
- **HTML Source** - Raw HTML source code for debugging

### Language Switching
Change the interface language via **View → Language**:

- **English** - Default language
- **Español** - Spanish interface
- **Français** - French interface  
- **Deutsch** - German interface

*Note: Language changes take effect immediately without restart!*

## Keyboard Shortcuts

### File Operations
- **Ctrl+N** - New document
- **Ctrl+O** - Open file
- **Ctrl+S** - Save
- **Ctrl+Shift+S** - Save As
- **Ctrl+Q** - Quit application

### Text Formatting
- **Ctrl+B** - Bold text
- **Ctrl+I** - Italic text
- **Ctrl+U** - Strikethrough
- **Ctrl+`** - Inline code
- **Ctrl+K** - Insert link

### Headings
- **Ctrl+1** - Heading 1
- **Ctrl+2** - Heading 2
- **Ctrl+3** - Heading 3
- **Ctrl+4** - Heading 4
- **Ctrl+5** - Heading 5
- **Ctrl+6** - Heading 6

### Lists and Quotes
- **Ctrl+Shift+8** - Bullet list
- **Ctrl+Shift+7** - Numbered list
- **Ctrl+Shift+.** - Blockquote

## Multi-Language Support

Marco provides complete interface translation in 4 languages. Here's how different UI elements appear:

### English (Default)
- File → New, Open, Save, Quit
- Insert → Bold, Italic, Link, Image
- Format → Strikethrough, Highlight, Table
- View → Theme, CSS Style, Language
- Help → About, Shortcuts

### Español  
- Archivo → Nuevo, Abrir, Guardar, Salir
- Insertar → Negrita, Cursiva, Enlace, Imagen
- Formato → Tachado, Resaltar, Tabla
- Ver → Tema, Estilo CSS, Idioma
- Ayuda → Acerca de, Atajos

### Français
- Fichier → Nouveau, Ouvrir, Enregistrer, Quitter
- Insérer → Gras, Italique, Lien, Image
- Format → Barré, Surligner, Tableau
- Affichage → Thème, Style CSS, Langue
- Aide → À propos, Raccourcis

### Deutsch
- Datei → Neu, Öffnen, Speichern, Beenden
- Einfügen → Fett, Kursiv, Link, Bild
- Format → Durchgestrichen, Hervorheben, Tabelle
- Ansicht → Thema, CSS-Stil, Sprache
- Hilfe → Über, Verknüpfungen

---

## Status and Statistics

This test document demonstrates:

✅ **Basic Markdown** - Headers, formatting, lists, links, images  
✅ **Extended Syntax** - Tables, task lists, footnotes, definition lists  
✅ **Advanced Features** - Admonitions, HTML elements, styling hacks  
✅ **Code Support** - 10 programming languages + 8 markup languages  
✅ **Interactive Elements** - Dialogs, emoji picker, table creator  
✅ **Themes** - 4 CSS themes + light/dark UI modes  
✅ **Multi-language** - 4 interface languages with instant switching  
✅ **Keyboard Shortcuts** - 20+ keyboard shortcuts for efficiency  

**Document Statistics:**
- **Words:** ~2,000
- **Characters:** ~15,000  
- **Lines:** ~500+
- **Code Blocks:** 18 different languages
- **Tables:** 3 tables
- **Links:** 10+ links
- **Images:** 5 images
- **Admonitions:** 5 types
- **Task Lists:** 10+ tasks

This comprehensive test file exercises every major feature in Marco and serves as both a demo and a testing document for development and QA purposes.

---

*Marco Markdown Editor - Built with ❤️ using Rust + GTK4*  
*Test document created: 2025-06-29*
