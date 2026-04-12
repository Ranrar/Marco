# Tables and Task Lists

GFM (GitHub Flavored Markdown) extends CommonMark with pipe tables and enhanced task list syntax.

---

## GFM Pipe Tables

### Alignment

Alignment is controlled by colons in the separator row.

| Left-aligned | Center-aligned | Right-aligned | Default |
|:-------------|:--------------:|--------------:|---------|
| Apple        |     Banana     |        Cherry | Date    |
| 100          |      200       |           300 | 400     |
| Long content |   Short        |     Also long | x       |

### Minimal Table

| A | B |
|---|---|
| 1 | 2 |
| 3 | 4 |

### Table with Inline Formatting

| Feature        | Status    | Notes                          |
|----------------|-----------|--------------------------------|
| Bold           | ✅ Works  | `**bold**`                     |
| Italic         | ✅ Works  | `*italic*`                     |
| Code span      | ✅ Works  | `` `code` ``                   |
| Links          | ✅ Works  | `[text](url)`                  |
| Strikethrough  | ✅ Works  | `~~text~~`                     |

### Wide Table

| Name        | Role          | Team        | Location   | Status   |
|-------------|---------------|-------------|------------|----------|
| Alice Smith | Lead Engineer | Platform    | Berlin     | Active   |
| Bob Jones   | Designer      | UX          | Amsterdam  | Active   |
| Carol White | DevOps        | Platform    | Remote     | Active   |
| Dave Brown  | Manager       | Engineering | London     | On leave |

---

## Headerless Tables (Marco Extension)

When the first line is the delimiter row (no header), Marco renders a body-only table:

|--------|--------|--------|
| First  | Second | Third  |
| Delta  | Epsilon| Zeta   |
| Eta    | Theta  | Iota   |

With alignment in the headerless delimiter:

|:---------|:-------:|--------:|
| Left     | Center  |  Right  |
| 1        |    2    |       3 |

---

## GFM Task Lists

### Basic Checkbox Lists

- [x] Set up the repository
- [x] Write the README
- [ ] Add CI/CD pipeline
- [ ] Publish first release
- [x] Add unit tests

### Nested Task Lists

- [x] Phase 1: Foundation
  - [x] Design data model
  - [x] Set up database
  - [x] Write migrations
- [ ] Phase 2: API
  - [x] Authentication endpoints
  - [ ] User management
  - [ ] File upload
- [ ] Phase 3: Frontend
  - [ ] Login page
  - [ ] Dashboard
  - [ ] Settings

### Task Lists in a Real Sprint

**Sprint 24 — March 4-18**

- [x] Fix parser regression on nested emphasis
- [x] Add Mermaid diagram rendering
- [ ] Implement PDF export
- [ ] Add bookmark sidebar
- [ ] Write user guide for tab blocks

---

## Inline Checkbox Markers (Marco Extension)

Task-style markers can also appear mid-paragraph:

Please review the PR [x] and merge when ready.

Outstanding items: add tests [ ], update docs [ ], notify team [x].

A checklist paragraph:

[x] DNS configured  
[x] SSL certificate active  
[ ] CDN edge cache cleared  
[ ] Monitoring alert set up  

---

## GFM Strikethrough

~~This text has been struck through.~~

Combined with other formatting: ~~**this was important**~~ (no longer relevant).

Longer example:

The original plan was to ~~deploy on Friday~~ deploy on Monday after the hotfix is merged.

---

## GFM Autolink Literals

Bare URLs and email addresses are automatically linked:

https://github.com/Ranrar/Marco

www.commonmark.org

user@example.com

Works mid-sentence: Visit https://github.com/Ranrar/Marco for the source code.

Multiple in a list:

- Homepage: https://github.com/Ranrar/Marco
- Issues: https://github.com/Ranrar/Marco/issues
- Contact: contact@example.com
