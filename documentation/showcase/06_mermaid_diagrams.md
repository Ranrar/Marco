# Mermaid Diagrams

Marco renders Mermaid diagrams natively using `mermaid-rs-renderer` — pure Rust, no browser required.

Use a fenced code block with the `mermaid` info string:

````
```mermaid
graph TD
  A --> B
```
````

---

## Flowcharts

```mermaid
graph TD
    A[Start] --> B{Is it working?}
    B -- Yes --> C[Deploy]
    B -- No --> D[Debug]
    D --> E[Fix the bug]
    E --> B
    C --> F[Monitor]
```

Left-to-right flowchart:

```mermaid
graph LR
    User --> Login
    Login --> Dashboard
    Dashboard --> Editor
    Dashboard --> Settings
    Dashboard --> Logout
```

---

## Sequence Diagrams

```mermaid
sequenceDiagram
    participant U as User
    participant E as Editor
    participant P as Parser
    participant R as Renderer

    U->>E: Type Markdown
    E->>P: Parse document
    P-->>E: AST
    E->>R: Render AST
    R-->>E: HTML
    E-->>U: Live preview
```

---

## Class Diagrams

```mermaid
classDiagram
    class Document {
        +Vec~Node~ children
        +parse(input: String) Document
        +render(options: RenderOptions) String
    }
    class Node {
        +NodeKind kind
        +Option~Span~ span
        +Vec~Node~ children
    }
    class NodeKind {
        <<enumeration>>
        Heading
        Paragraph
        CodeBlock
        Emphasis
        Strong
    }
    Document --> Node : contains
    Node --> NodeKind : has
    Node --> Node : children
```

---

## State Diagrams

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Editing : Open file
    Editing --> Saving : Ctrl+S
    Saving --> Editing : Saved
    Editing --> Previewing : Toggle preview
    Previewing --> Editing : Edit
    Editing --> Idle : Close file
    Idle --> [*]
```

---

## Entity-Relationship Diagrams

```mermaid
erDiagram
    USER {
        int id PK
        string name
        string email
        datetime created_at
    }
    DOCUMENT {
        int id PK
        string title
        text content
        int user_id FK
        datetime updated_at
    }
    BOOKMARK {
        int id PK
        int document_id FK
        int line_number
        string label
    }
    USER ||--o{ DOCUMENT : owns
    DOCUMENT ||--o{ BOOKMARK : has
```

---

## Pie Charts

```mermaid
pie title Marco Parser Test Coverage
    "Grammar tests" : 42
    "Parser tests" : 28
    "Render tests" : 18
    "Integration tests" : 12
```

---

## Gantt Charts

```mermaid
gantt
    title Marco Development Roadmap
    dateFormat  YYYY-MM-DD
    section Parser
    CommonMark compliance    :done,    cm,   2024-01-01, 2024-06-01
    GFM extensions          :done,    gfm,  2024-04-01, 2024-08-01
    Marco extensions        :active,  mx,   2024-07-01, 2025-03-01
    section Editor
    Core editor UI          :done,    ui,   2024-03-01, 2024-09-01
    Toolbar and menus       :done,    tb,   2024-08-01, 2024-12-01
    LSP integration         :         lsp,  2025-01-01, 2025-06-01
    section Features
    Math rendering          :done,    math, 2024-10-01, 2025-01-01
    Mermaid diagrams        :done,    memd, 2025-01-01, 2025-03-01
    PDF export              :         pdf,  2025-04-01, 2025-09-01
```

---

## Git Graphs

```mermaid
gitGraph
    commit id: "Initial commit"
    commit id: "Add parser skeleton"
    branch feature/gfm-tables
    checkout feature/gfm-tables
    commit id: "Add table grammar"
    commit id: "Add table renderer"
    checkout main
    merge feature/gfm-tables id: "Merge GFM tables"
    branch feature/math
    checkout feature/math
    commit id: "Add KaTeX integration"
    checkout main
    merge feature/math id: "Merge math support"
    commit id: "Release v0.8.0"
```

---

## Diagrams in Context

Mermaid diagrams work inside blockquotes:

> The following diagram shows the request flow:
>
> ```mermaid
> graph LR
>     Client --> API --> Database
> ```

And in list items:

- **Frontend flow:**

  ```mermaid
  graph LR
      Input --> Validate --> Submit
  ```

- **Backend flow:**

  ```mermaid
  graph LR
      Request --> Auth --> Handler --> Response
  ```
