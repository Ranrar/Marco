# Table Auto-Align

This file is used to manually test the **Auto-Align Tables** feature (Settings → Editor → Auto-Align Tables).

When the toggle is **on**, pressing `Tab` or `Enter` inside any of the tables below will
reformat the entire table so all columns are evenly padded.

Try it: place the cursor inside a cell, type some text, then press `Tab` or `Enter`.

---

## Basic Two-Column Table

| Name  | Value |
|-------|-------|
| First | 1     |
| Second| 2     |
| Third | 3     |

---

## Table with Alignment

| Left-aligned | Center-aligned | Right-aligned |
|:-------------|:--------------:|--------------:|
| Apple        | Banana         | Cherry        |
| 100          | 200            | 300           |

---

## Wide Table — Many Columns

| ID | First Name | Last Name | Department  | Location | Status   |
|----|------------|-----------|-------------|----------|----------|
| 1  | Alice      | Smith     | Engineering | Berlin   | Active   |
| 2  | Bob        | Jones     | Design      | Paris    | Active   |
| 3  | Carol      | White     | Marketing   | London   | On leave |

---

## Narrow Table — Single Column

| Item    |
|---------|
| First   |
| Second  |
| Third   |

---

## Table with Long Cell Content

| Short | This column has a very long header that spans a lot of characters |
|-------|-------------------------------------------------------------------|
| A     | Short value                                                       |
| B     | A much longer value that extends the column width considerably    |
| C     | Medium length                                                     |

---

## Table with Inline Formatting

| Feature        | Supported | Example                      |
|----------------|-----------|------------------------------|
| Bold           | Yes       | **bold text**                |
| Italic         | Yes       | *italic text*                |
| Code span      | Yes       | `inline code`                |
| Strikethrough  | Yes       | ~~removed~~                  |
| Link           | Yes       | [example](https://example.com) |

---

## Empty Cells

| A   | B   | C   |
|-----|-----|-----|
| 1   |     | 3   |
|     | 2   |     |
| 4   | 5   |     |

---

## Resize Test

Add text to a cell below and press Tab — the table should widen to fit.

| Expand me | Expand me too |
|-----------|---------------|
| short     | short         |
| short     | short         |
