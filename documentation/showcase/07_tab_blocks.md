# Tab Blocks

Tab blocks let you present alternative content (code, instructions, configurations) in a switchable panel UI — no JavaScript required.

## Basic Syntax

Open with `:::tab`, add panels using `@tab Title`, close with `:::`.

:::tab
@tab Overview
Tab blocks render as a row of tabs with radio-button switching. Each `@tab` line starts a new panel. Panels can contain any Markdown content.

@tab Syntax
```
:::tab
@tab First Tab
Content for the first tab.

@tab Second Tab
Content for the second tab.
:::
```

@tab Notes
- Tabs are switched using pure CSS (no JavaScript).
- Missing closing `:::` prevents the block from rendering as tabs.
- Nested tab blocks are not supported; an inner `:::tab` renders as literal text.
:::

---

## Code Examples Across Languages

:::tab
@tab Rust
```rust
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

fn main() {
    println!("{}", greet("Marco"));
}
```

@tab Python
```python
def greet(name: str) -> str:
    return f"Hello, {name}!"

print(greet("Marco"))
```

@tab TypeScript
```typescript
function greet(name: string): string {
    return `Hello, ${name}!`;
}

console.log(greet("Marco"));
```

@tab Go
```go
package main

import "fmt"

func greet(name string) string {
    return fmt.Sprintf("Hello, %s!", name)
}

func main() {
    fmt.Println(greet("Marco"))
}
```
:::

---

## Installation Instructions by Platform

:::tab
@tab Linux (Debian/Ubuntu)
Download the latest `.deb` from the [latest release](https://github.com/Ranrar/Marco/releases/latest):

```bash
# Download and install
wget https://github.com/Ranrar/Marco/releases/latest/download/marco-suite_<version>_linux_amd64.deb
sudo dpkg -i marco-suite_<version>_linux_amd64.deb
sudo apt-get install -f   # fix any dependency gaps
```

@tab Windows
Download the portable `.zip` from the [latest release](https://github.com/Ranrar/Marco/releases/latest):

1. Download `marco-suite_<version>_windows_amd64.zip`
2. Extract to any folder, e.g. `C:\Tools\Marco`
3. Run `marco.exe` or `polo.exe`
4. Settings are stored next to the executable (portable mode)

@tab Build from Source
Requires Rust 1.92.0+ and the GTK4 development libraries.

```bash
git clone https://github.com/Ranrar/Marco.git
cd Marco
cargo build --release -p marco
cargo build --release -p polo
```
:::

---

## Configuration File Examples

:::tab
@tab settings.ron
```ron
(
    theme: Some("github"),
    language: Some("en"),
    editor_font: Some("JetBrains Mono"),
    font_size: Some(14),
    line_height: Some(1.6),
    log_to_file: Some(true),
)
```

@tab docker-compose.yml
```yaml
version: "3.9"
services:
  app:
    image: my-app:latest
    ports:
      - "8080:8080"
    environment:
      - DB_HOST=db
      - DB_PORT=5432
    depends_on:
      - db
  db:
    image: postgres:15
    volumes:
      - pgdata:/var/lib/postgresql/data

volumes:
  pgdata:
```

@tab .env
```bash
# Application
APP_ENV=production
APP_PORT=8080
SECRET_KEY=change-me-in-production

# Database
DB_HOST=localhost
DB_PORT=5432
DB_NAME=myapp
DB_USER=appuser
DB_PASS=secure-password
```
:::

---

## Rich Content Inside Panels

Panels can contain any Markdown elements:

:::tab
@tab Tables
| Feature       | Linux | Windows |
|---------------|-------|---------|
| GTK4 UI       | ✅    | ✅      |
| WebKit preview| ✅    | —       |
| WebView2      | —     | ✅      |
| .deb package  | ✅    | —       |
| Portable .zip | —     | ✅      |

@tab Lists
**Prerequisites:**

- [x] Rust 1.92.0 or later
- [x] GTK4 development libraries
- [ ] SourceView5 (optional, for editor syntax highlighting)
- [ ] WebKit6 (Linux preview engine)

@tab Math
The editor renders KaTeX math natively:

Inline: $E = mc^2$

Display:

$$
\int_0^1 x^2 \, dx = \frac{1}{3}
$$

:::

---

## Many Tabs

Up to 12 tabs are supported with the built-in CSS:

:::tab
@tab 1
Panel one.
@tab 2
Panel two.
@tab 3
Panel three.
@tab 4
Panel four.
@tab 5
Panel five.
@tab 6
Panel six.
:::
