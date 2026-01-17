# Tab Blocks (Marco_Extended) Showcase

This file focuses on `:::tab` + `@tab` behavior, including edge cases.

## Basic

:::tab
@tab First
Hello **world**.

- List item 1
- List item 2

@tab Second
A heading inside a panel:

## Panel Heading

A link: https://example.com
:::

## Opener line with a title (currently ignored by the parser)

:::tab My Title Here
@tab A
Panel A.

@tab B
Panel B.
:::

## Many tabs (UI limit)

The HTML renderer supports any number of tabs, but the current CSS rules
only toggle the first 12 panels.

:::tab
@tab 01
One.
@tab 02
Two.
@tab 03
Three.
@tab 04
Four.
@tab 05
Five.
@tab 06
Six.
@tab 07
Seven.
@tab 08
Eight.
@tab 09
Nine.
@tab 10
Ten.
@tab 11
Eleven.
@tab 12
Twelve.
@tab 13
Thirteen (may not toggle with current CSS).
:::

## Nested tabs are forbidden

The inner `:::tab` should be treated as literal content, not a nested tab UI.

:::tab
@tab Outer

:::tab
@tab Inner
Inner.
:::

Outer continues.
:::
