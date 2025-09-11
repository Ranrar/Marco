# Line Break Test

## Normal Mode Test (when mode is "normal")
This is line one
This is line two

This is line three  
This is line four

This is line five\
This is line six

## Reversed Mode Test (when mode is "reversed")  
According to the settings, the mode is set to "reversed", so:

- Single newlines should create visible breaks (\<br\>)
- Double spaces + newlines should NOT create visible breaks (just spaces)

Let's test:
First line
Second line

First with spaces  
Second with spaces

First with backslash\
Second with backslash
