#!/bin/bash

# Marco Markdown Showcase Test Runner
# This script runs basic parsing tests on all showcase documents

set -e

SHOWCASE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SHOWCASE_DIR/../.." && pwd)"

echo "🚀 Marco Markdown Showcase Test Runner"
echo "======================================="
echo

cd "$PROJECT_ROOT"

# Check if Marco binary exists
if ! cargo build --quiet; then
    echo "❌ Failed to build Marco. Please fix build errors first."
    exit 1
fi

echo "✅ Marco built successfully"
echo

# Test each showcase document
documents=(
    "01_basic_markdown.md"
    "02_marco_extensions.md" 
    "03_edge_cases.md"
    "04_real_world_example.md"
)

failures=0
total=0

for doc in "${documents[@]}"; do
    total=$((total + 1))
    doc_path="tests/markdown_showcase/$doc"
    
    echo "📄 Testing $doc..."
    
    # Try to parse the document
    if timeout 30s cargo run --quiet --bin debug_text_parsing -- "$doc_path" > /dev/null 2>&1; then
        echo "  ✅ Parsing successful"
    else
        echo "  ❌ Parsing failed or timed out"
        failures=$((failures + 1))
    fi
    
    # Check file size (documents should be reasonable size)
    size=$(wc -c < "$doc_path" 2>/dev/null || echo "0")
    if [ "$size" -gt 100000 ]; then
        echo "  ⚠️  Warning: Document is very large (${size} bytes)"
    elif [ "$size" -eq 0 ]; then
        echo "  ❌ Error: Document is empty or missing"
        failures=$((failures + 1))
    else
        echo "  📏 Size: ${size} bytes"
    fi
    
    echo
done

# Summary
echo "📊 Test Summary"
echo "==============="
echo "Total documents: $total"
echo "Successful: $((total - failures))"
echo "Failed: $failures"
echo

if [ $failures -eq 0 ]; then
    echo "🎉 All showcase documents passed basic parsing tests!"
    exit 0
else
    echo "💥 Some documents failed parsing tests."
    echo "Run individual tests with:"
    echo "  cargo run --bin debug_text_parsing -- tests/markdown_showcase/[document]"
    exit 1
fi