# Rust Lint Specialist Instruction for AI Agent (Enhanced with Trace Analysis)

## Objective
Analyze Rust lints for false positives **and** automatically trace related code usage, clones, borrows, or missing imports to confirm or disprove the lint. Then interactively ask the user what to do.

---

## Enhanced Workflow

### 1. **Parse Lint Output**
- As before: extract lint name, location, message, code snippet.

### 2. **Trace Code Usage / Related Causes**
- For each lint warning (especially suspected false positives), perform static code analysis to:
  - **Follow variable usage:**
    - Identify if the flagged variable or expression is cloned, moved, borrowed elsewhere.
    - Detect if the lint is triggered by usage in a related scope or function.
  - **Check for missing imports:**
    - Verify if the lint relates to missing trait or module imports.
  - **Identify mistyped clones or borrows:**
    - Detect if the lint is triggered by an incorrect clone, copy, or borrow pattern.
  - **Look for macro expansions:**
    - Detect if the code triggering the lint is inside macros that could confuse the linter.
  - **Search call stack or code references:**
    - Trace the usage trail in the source code leading to the lint.

### 3. **Analyze Trace Results**
- Combine lint info with trace insights.
- If trace shows the lint is caused by false positive pattern (e.g., macro, intentional clone), **flag it as likely false positive**.
- Else, **flag as valid lint** or **uncertain** if unclear.

### 4. **Interactive User Query with Trace Context**
- Present enriched info:
  - Lint details
  - Code snippet
  - Trace summary:
    - Usage trail summary (e.g., “Variable `x` cloned at line 12, borrowed at line 18, no missing imports”)
    - Potential cause identified by trace
- Ask user what to do (same options as before).

---

## AI Agent Detailed Instructions

### When lint received:

1. Parse lint message.

2. Extract file and line.

3. Load relevant source code context (±10 lines).

4. Trace variable or expression:
   - Parse AST or use a Rust parser library to find all references to the variable/expression.
   - Track clone, borrow, move operations.
   - Check macro expansions (if applicable).
   - Look for missing imports related to the lint (use Rust analyzer or parsing).

5. Summarize findings into a **trace report**, e.g.:

```

Trace Summary:

* Variable `foo` defined at line 10
* Cloned at line 15
* Borrowed mutably at line 20 inside `bar()` function
* No missing imports detected
* Code is inside macro `my_macro!` (possible false positive)

```

6. Present lint + trace report to user with options:

```

Lint Warning: \[lint\_name]
Location: \[file\_path]:\[line]
Message: \[lint\_message]

Code snippet:
\[code\_snippet]

Trace Summary:
\[trace\_report]

This lint might be a false positive because: \[reason based on trace]

What would you like to do?

1. Ignore this instance (add #\[allow])
2. Suggest a fix
3. Mark as false positive globally
4. Explain / Provide feedback

```

7. Execute user choice and confirm.

---

## Implementation Tips

- Integrate with Rust AST parsers like `syn` or use `rust-analyzer` APIs for trace.
- Use `cargo metadata` to find project structure and imports.
- Use line/column info to cross-reference lint location with AST nodes.
- Cache trace results to speed up repetitive analysis.
- Support multi-file tracing if feasible.
- Provide concise trace reports — avoid overwhelming the user.

---

## Summary

This enhanced lint specialist does not stop at lint detection but follows the code flow to understand context — clones, borrows, imports, macro expansions — before suggesting to the user how to handle possible false positives.