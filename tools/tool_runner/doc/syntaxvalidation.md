## Requirements checklist
- Create a Rust CLI tool to load and run logic-based validations across `ast.ron`, `syntax.ron`, and `markdown.pest`. — Planned
- Tests must be 100% logic-based with no hardcoded values. — Planned (property-based testing with proptest)
- Research and propose libraries and a design that deserializes RON, parses pest grammars, and validates cross-file invariants. — Planned
- Provide file layout, data contracts, error handling, CI, and next steps. — Planned

Assumptions
- The RON files describe serializable Rust data structures (AST and syntax). If they use keys/structures not known, the tool will use flexible serde types (enums/structs) or untyped Value-like structures and derive shape expectations via tests.
- `markdown.pest` is a pest grammar file; validation will ensure grammar rules referenced by RON are present and consistent.
- The workspace uses stable Rust and can add dependencies via `Cargo.toml`.
- No network calls or external services; all checks are local.

## High-level approach
1. Build a Rust binary crate tool_runner (existing) exposing a CLI:
   - Commands: `validate` (validate files), `dump` (print parsed structures), `test` (run validation harness).
   - Use `clap` for CLI parsing.
2. Load files:
   - `ast.ron` and `syntax.ron`: deserialize using `ron` + `serde` to concrete types when possible, otherwise to `ron::Value` or intermediate structs.
   - `markdown.pest`: parse grammar as text and feed into `pest_derive` or `pest_meta` to inspect rules.
3. Define a validation contract (the “contract” / small contract summary):
   - Inputs: three files (AST RON, Syntax RON, Pest grammar).
   - Outputs: success / structured error list.
   - Error modes: parse error, schema mismatch, missing rule, cross-reference mismatch.
   - Success: all invariants hold.
4. Validation invariants (examples to implement as logic rules — no constants):
   - Every identifier referenced in `ast.ron` that should be defined in `syntax.ron` must have a matching definition.
   - All rules referenced by syntax or AST that correspond to grammar rules must exist in `markdown.pest`.
   - Node types in AST must have valid fields per syntax definitions (e.g., an AST node referencing a list of child node kinds must refer to kinds that exist).
   - No unreachable/undefined grammar rules referenced from RON structures.
   - Round-trip invariants: serializing and deserializing the loaded data preserves shape (using serde) — used in tests.
5. Testing strategy (100% logic-based):
   - Use property-based tests (proptest) to generate random, well-formed RON-like structures encoded via Rust types or Value-like types.
   - Instead of hardcoded examples, express invariants as properties:
     - For any generated well-formed AST and syntax pair (generator ensures internal consistency constraints), validate() returns Ok.
     - For any generated AST that references a non-existent syntax element, validate() returns Err and the error mentions the missing symbol.
     - Grammar rule validation: generate grammar-like rule names and test cross-reference properties; use `pest_meta` to parse the generated grammar text and assert the expected rules exist/are absent.
   - Use shrinkers and targeted strategies to expose minimal failing cases.
6. Implementation notes:
   - Use serde + ron crate for RON (serde feature).
   - Use pest_meta or pest to parse and inspect pest grammars programmatically.
   - Use proptest for property testing. If needed, combine with quickcheck-style arbitrary impls.
   - Represent RON content using typed structs for strong checks; but include fallback "untyped" path using `ron::Value` or `serde_json::Value` (via ron->serde) for flexible tests and to support unknown shapes.
7. CI and reproducibility:
   - Add GitHub Actions workflow to run `cargo test` on Linux, Windows, Mac.
   - Pin dependency versions in `Cargo.toml`.
8. Deliverables:
   - CLI code in main.rs plus modules `loader.rs`, `validate.rs`, `types.rs`, `tests/property_tests.rs`.
   - Updated `Cargo.toml` with dependencies: clap, ron, serde, pest, pest_meta, proptest, anyhow/thiserror for errors, tracing/log optional.
   - README with run and test instructions.

## Detailed technical plan and file layout (Markdown)
### Project layout
- tool_runner/
  - Cargo.toml (add deps)
  - src/
    - main.rs             — CLI (clap) + command dispatch
    - loader.rs           — functions to read & deserialize RON files and load grammar
    - types.rs            — Rust types mirroring expected RON structures, plus Value fallback
    - validate.rs         — cross-file validation logic and error types
    - errors.rs           — error enums (use thiserror)
    - tests/
      - prop_tests.rs     — property-based tests (proptest)
    - bin/ (optional)     — extra small runner
  - doc/
    - syntaxvalidation.md — existing doc can be integrated

### Key crates and versions (example, pick stable ranges)
- ron = "0.8"
- serde = { version = "1.0", features = ["derive"] }
- clap = { version = "4", features = ["derive"] }
- pest = "3"
- pest_meta = "3"
- proptest = "1.0"
- thiserror = "1.0"
- anyhow = "1.0"
- tracing = optional

### Core data contracts (contract summary)
- Inputs:
  - ast.ron -> ASTRoot (list/map of nodes with identifiers, node kinds, fields)
  - syntax.ron -> SyntaxRoot (definitions of node kinds, allowed fields, references to grammar rules)
  - markdown.pest -> text grammar accessible to pest_meta
- Outputs:
  - ValidationResult: Ok(()) or Err(Vec<ValidationError>)
- ValidationError shape:
  - kind: ParseError | MissingDefinition | FieldMismatch | GrammarMissingRule | CrossReferenceError
  - location: file + path/line/ron-key (optionally)
  - message: human-readable

### Example invariants (logic rules)
- name_resolution:
  - For all node.kind values in ASTRoot, exists a syntax definition with matching kind (case-insensitive or exact per config).
- field_conformance:
  - For each AST node, for each field, the field key is allowed by the syntax definition for that node kind.
- grammar_linking:
  - For any syntax definition that references a grammar rule (e.g., "inline_markdown_rule": "Paragraph"), verify that `markdown.pest` contains a rule named "Paragraph" (parsed by pest_meta).
- round_trip:
  - Deserialize then reserialize the RON content and re-deserialize; resulting structure must be equal (serde equality). Use this as a property.
- error reporting:
  - If a reference is missing, validation returns Err and error.kind == MissingDefinition.

### Property-based test design (no hardcoded values)
- Generators:
  - Generate sets of unique names for node kinds, field names, and grammar rule names.
  - Generate syntax definitions as maps keyed by node kind; each definition contains allowed field names and optionally grammar rule names chosen from the generated rule set.
  - Generate AST nodes that reference kinds chosen from the generated kind set (for valid cases) or outside that set (for invalid cases).
  - Generate grammar text by synthesizing pest grammar definitions for the generated rule names using simple pattern fragments (e.g., rule = { SOI ~ ANY* ~ EOI } style) — but ensure text parses with pest_meta by using valid RHS constructs. Use grammar templates instead of constants: build rules using concatenations and choices of sub-rule names from the generated names.
- Properties:
  - Valid pair: When AST only references existing kinds and syntax references existing grammar rules, validate() must return Ok.
  - Invalid-missing-kind: When AST references absent kind, validate() must return Err mentioning missing kind.
  - Invalid-grammar: When syntax references a grammar rule not in the grammar text, validate() must return Err pointing to missing grammar rule.
  - Round-trip property: Deserializing and serializing shouldn't change the structure.

### Edge cases to handle
- Empty files or empty collections.
- Recursive grammar rules and circular references in syntax; detection and clear errors.
- Large generated inputs — tests should cap size to keep runs fast.
- Unknown/unexpected RON structures — fallback to Value-based validation and produce helpful diagnostics.

### Implementation sketch
- loader.rs:
  - fn load_ron<T: for<'de> Deserialize<'de>>(path: impl AsRef<Path>) -> Result<T>
  - fn load_ron_value(path) -> Result<ron::Value>
  - fn load_pest_grammar(path) -> Result<pest_meta::ast::Grammar>
- types.rs:
  - Define minimal typed structs:
    - ASTRoot { nodes: Vec<Node> }
    - Node { id: String, kind: String, fields: HashMap<String, ron::Value> }
    - SyntaxRoot { kinds: HashMap<String, KindDef> }
    - KindDef { allowed_fields: Option<Vec<String>>, grammar_rule: Option<String> }
  - Provide From<ron::Value> fallback constructors.
- validate.rs:
  - fn validate(ast: &ASTRoot, syntax: &SyntaxRoot, grammar: &Grammar) -> Result<(), Vec<ValidationError>>
  - Implement invariants and collect errors instead of bail-on-first for better reports.
- tests/prop_tests.rs:
  - Use proptest strategies to generate the sets and structures described above.
  - For generation of `markdown.pest`, build grammar text programmatically ensuring it parses using pest_meta::parser::parse or pest_meta functions.

### CI and run instructions
- Run locally:
  - Build: `cargo build --manifest-path tool_runner/Cargo.toml`
  - Validate files: `cargo run --manifest-path Cargo.toml -- validate ast.ron syntax.ron markdown.pest`
  - Run tests: `cargo test --manifest-path Cargo.toml -p tool_runner`
- GitHub Actions:
  - Workflow that runs `cargo test` matrix across OSes and Rust stable.

### Next steps and optional improvements
- Add more typed schemas driven from real RON sample files in the repo.
- Provide auto-fix suggestions or CLI subcommands to produce minimal changes.
- Add structured JSON machine-readable output for integration with editors.
- Add benchmarks for large inputs.

## Research notes (libraries & rationale)
- ron + serde: natural fit for RON serialization/deserialization into Rust structs; well-maintained.
- pest + pest_meta: pest for grammars, and pest_meta can parse grammar text into an AST to programmatically inspect rule names.
- proptest: robust property-based testing; supports complex composite generators and shrinking.
- thiserror/anyhow: ergonomic error types and context.
- clap: battle-tested CLI builder (derive API keeps main.rs small).

## Minimal timeline and effort estimate
- Design + plan (this doc): 1 day (done)
- Implement loader + types + simple validator: 1-2 days
- Implement property-based tests + shrinkers: 1-2 days
- Add CI and docs: 0.5 day
- Polish & edge cases: 0.5-1 day

## Acceptance criteria
- CLI `validate` runs and returns non-zero exit on validation failures and 0 on success.
- Property tests exercise the invariants without using example constants.
- Descriptive error messages for missing definitions and grammar mismatches.
- All tests run in CI and pass.