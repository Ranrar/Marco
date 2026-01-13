# CommonMark gaps showcase: entities + reference links

This file is a manual showcase used by the test suite assets.
It exercises two baseline CommonMark features:

- Entity references (named + numeric)
- Reference-style links (full / collapsed / shortcut), including forward definitions

## Entity references

Named + numeric entities should decode during parsing and then be rendered safely:

- Named: &copy; &amp;
- Numeric (decimal): &#169;
- Numeric (hex): &#xA9;

Expected visible output:

- `©` for the copyright symbol
- `&` for ampersand

## Reference-style links

These reference links should resolve even though the definition is below.

- Full: [Marco][marco]
- Collapsed: [Marco][]
- Shortcut: [marco]
- With inline formatting in link text: [**bold Marco**][marco]

Unresolved references should render literally:

- Missing: [missing][ref]

[marco]: https://github.com/Ranrar/Marco "Marco repository"
