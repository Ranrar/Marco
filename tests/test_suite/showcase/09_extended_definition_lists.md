# Extended definition lists showcase

Implemented syntax (Markdown Guide extended syntax):

Term
: definition

## Canonical examples

Term 1
: Definition for term 1

Term 2
: Definition for term 2
: Alternative definition for term 2

Term 2b (blank line between items)
: Definition for term 2b

Term 2c
: Definition for term 2c

## Edge cases

### Formatting in terms and definitions

Complex Term with **Formatting**
: Definition with **bold**, *italic*, and `code`
: Definition with a [link](https://example.com)

### Indentation and multi-line definitions

Term 3
: First line  
  continuation line (indented)  
  another continuation

Term 3b (blank line inside a definition, only continues when next line is indented)
: First line of definition

  continuation after a blank line
  still same definition

Term 3c (definition marker line may have up to 3 leading spaces)
   : Definition marker with leading spaces

### Nested blocks inside a definition (design choice)

Term 4
: A list inside a definition
  - item 1
  - item 2

### Lookalikes / invalid forms

: Definition without a term

Term 5
:: Double colon (should not be a definition list)
