# Diagnostics Popover Manual Test

Use this document to validate the diagnostics popover UX:
- severity sorting (Error → Warning → Info/Hint)
- tiny severity color chip per row
- row click jump to line:column
- filter toggle: **Show all** / **Critical only**

---

## Expected diagnostics payload

### Error (critical)

1. Empty image URL + missing alt:

![]()

### Warnings

2. Potentially unsafe link protocol:

[Unsafe JS link](javascript:alert('x'))

3. Missing footnote definition:

Reference to undefined footnote [^missing-def].

4. Duplicate heading id:

## Duplicate ID Heading {#dup-id}

## Duplicate ID Heading Again {#dup-id}

5. Link and image targets that do not exist (`MD206` / `MD404`, checked
   against the filesystem — only in a saved document):

[Link to a file that is not there](files/no-such-document.md)

![Image that is not there](files/no-such-image.png)

The existing target beside them must **not** be reported:

[Linked document](files/linked_doc.md)

### Info / Hint

6. Insecure link protocol info:

[HTTP link](http://example.com)

7. Code block without language hint:

```
let x = 42;
println!("no language fence");
```

8. Unused footnote definition hint:

[^unused-def]: This definition is intentionally unused.

---

## Interaction checks

1. Open diagnostics popover from footer trigger.
2. Confirm list order is **Error first**, then Warning, then Info/Hint.
3. Confirm each row has a tiny severity chip (`E/W/I/H`).
4. Click each row and verify cursor jumps to the reported `line:column`.
5. Switch to **Critical only** and confirm only errors remain.
6. Switch back to **Show all** and confirm all items return.

---

## Optional stress check

Add multiple extra links/images and verify popover remains usable and sorted.
