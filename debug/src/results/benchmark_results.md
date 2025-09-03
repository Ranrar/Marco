# Marco Grammar Performance Benchmark Report

Generated automatically from benchmark tests

## Summary

- **Total Tests**: 340
- **Passed**: 140 ✅
- **Failed**: 200 ❌
- **Total Time**: 33.26ms
- **Average Parse Time**: 72.98μs
- **Memory Estimate**: 57560 bytes
- **Slowest Test**: perf_academic_paper (489.83μs)
- **Fastest Test**: perf_complex_formatting (3.48μs)

## Performance Analysis

### Large (201-1000 chars)
- Tests: 80
- Average Time: 173.31μs
- Average Size: 294 chars
- Throughput: 1.70 MB/s

### Small (0-50 chars)
- Tests: 50
- Average Time: 14.46μs
- Average Size: 33 chars
- Throughput: 2.28 MB/s

### Medium (51-200 chars)
- Tests: 200
- Average Time: 34.55μs
- Average Size: 100 chars
- Throughput: 2.89 MB/s

### Extra Large (1000+ chars)
- Tests: 10
- Average Time: 331.64μs
- Average Size: 1217 chars
- Throughput: 3.67 MB/s

## Detailed Results

| Test Name | Rule | Time (μs) | Input Size | Status |
|-----------|------|-----------|------------|--------|
| perf_simple_parse | text | 11.75 | 30 | ✅ |
| perf_simple_parse | text | 11.60 | 30 | ✅ |
| perf_simple_parse | text | 11.61 | 30 | ✅ |
| perf_simple_parse | text | 11.55 | 30 | ✅ |
| perf_simple_parse | text | 11.62 | 30 | ✅ |
| perf_simple_parse | text | 11.94 | 30 | ✅ |
| perf_simple_parse | text | 11.65 | 30 | ✅ |
| perf_simple_parse | text | 11.62 | 30 | ✅ |
| perf_simple_parse | text | 11.57 | 30 | ✅ |
| perf_simple_parse | text | 11.63 | 30 | ✅ |
| perf_complex_formatting | emphasis | 5.94 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.57 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.51 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.48 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.54 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.49 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.51 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.84 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.66 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.59 | 65 | ✅ |
| perf_nested_structures | text | 7.01 | 52 | ❌ |
| perf_nested_structures | text | 7.18 | 52 | ❌ |
| perf_nested_structures | text | 7.11 | 52 | ❌ |
| perf_nested_structures | text | 7.06 | 52 | ❌ |
| perf_nested_structures | text | 7.34 | 52 | ❌ |
| perf_nested_structures | text | 7.14 | 52 | ❌ |
| perf_nested_structures | text | 7.04 | 52 | ❌ |
| perf_nested_structures | text | 6.92 | 52 | ❌ |
| perf_nested_structures | text | 7.18 | 52 | ❌ |
| perf_nested_structures | text | 6.92 | 52 | ❌ |
| perf_large_paragraph | text | 76.44 | 279 | ✅ |
| perf_large_paragraph | text | 76.39 | 279 | ✅ |
| perf_large_paragraph | text | 76.77 | 279 | ✅ |
| perf_large_paragraph | text | 76.90 | 279 | ✅ |
| perf_large_paragraph | text | 82.56 | 279 | ✅ |
| perf_large_paragraph | text | 76.67 | 279 | ✅ |
| perf_large_paragraph | text | 76.64 | 279 | ✅ |
| perf_large_paragraph | text | 76.92 | 279 | ✅ |
| perf_large_paragraph | text | 76.53 | 279 | ✅ |
| perf_large_paragraph | text | 76.67 | 279 | ✅ |
| perf_github_readme | document | 412.46 | 203 | ✅ |
| perf_github_readme | document | 399.17 | 203 | ✅ |
| perf_github_readme | document | 399.13 | 203 | ✅ |
| perf_github_readme | document | 405.99 | 203 | ✅ |
| perf_github_readme | document | 397.91 | 203 | ✅ |
| perf_github_readme | document | 405.16 | 203 | ✅ |
| perf_github_readme | document | 397.78 | 203 | ✅ |
| perf_github_readme | document | 396.61 | 203 | ✅ |
| perf_github_readme | document | 406.29 | 203 | ✅ |
| perf_github_readme | document | 401.61 | 203 | ✅ |
| perf_academic_paper | document | 362.89 | 249 | ✅ |
| perf_academic_paper | document | 353.62 | 249 | ✅ |
| perf_academic_paper | document | 352.44 | 249 | ✅ |
| perf_academic_paper | document | 359.95 | 249 | ✅ |
| perf_academic_paper | document | 355.69 | 249 | ✅ |
| perf_academic_paper | document | 362.80 | 249 | ✅ |
| perf_academic_paper | document | 353.55 | 249 | ✅ |
| perf_academic_paper | document | 381.56 | 249 | ✅ |
| perf_academic_paper | document | 489.83 | 249 | ✅ |
| perf_academic_paper | document | 397.27 | 249 | ✅ |
| perf_many_small_elements | text | 14.20 | 80 | ❌ |
| perf_many_small_elements | text | 14.26 | 80 | ❌ |
| perf_many_small_elements | text | 13.65 | 80 | ❌ |
| perf_many_small_elements | text | 14.35 | 80 | ❌ |
| perf_many_small_elements | text | 14.00 | 80 | ❌ |
| perf_many_small_elements | text | 14.68 | 80 | ❌ |
| perf_many_small_elements | text | 13.55 | 80 | ❌ |
| perf_many_small_elements | text | 13.70 | 80 | ❌ |
| perf_many_small_elements | text | 13.86 | 80 | ❌ |
| perf_many_small_elements | text | 13.72 | 80 | ❌ |
| perf_few_large_elements | text | 8.17 | 184 | ❌ |
| perf_few_large_elements | text | 7.67 | 184 | ❌ |
| perf_few_large_elements | text | 8.27 | 184 | ❌ |
| perf_few_large_elements | text | 8.29 | 184 | ❌ |
| perf_few_large_elements | text | 7.64 | 184 | ❌ |
| perf_few_large_elements | text | 7.62 | 184 | ❌ |
| perf_few_large_elements | text | 13.47 | 184 | ❌ |
| perf_few_large_elements | text | 7.92 | 184 | ❌ |
| perf_few_large_elements | text | 7.63 | 184 | ❌ |
| perf_few_large_elements | text | 8.27 | 184 | ❌ |
| perf_shallow_wide | text | 13.69 | 79 | ❌ |
| perf_shallow_wide | text | 14.07 | 79 | ❌ |
| perf_shallow_wide | text | 13.32 | 79 | ❌ |
| perf_shallow_wide | text | 14.16 | 79 | ❌ |
| perf_shallow_wide | text | 13.41 | 79 | ❌ |
| perf_shallow_wide | text | 14.04 | 79 | ❌ |
| perf_shallow_wide | text | 14.25 | 79 | ❌ |
| perf_shallow_wide | text | 14.15 | 79 | ❌ |
| perf_shallow_wide | text | 13.15 | 79 | ❌ |
| perf_shallow_wide | text | 13.95 | 79 | ❌ |
| perf_deep_narrow | text | 8.82 | 36 | ❌ |
| perf_deep_narrow | text | 8.54 | 36 | ❌ |
| perf_deep_narrow | text | 8.50 | 36 | ❌ |
| perf_deep_narrow | text | 8.78 | 36 | ❌ |
| perf_deep_narrow | text | 8.97 | 36 | ❌ |
| perf_deep_narrow | text | 8.77 | 36 | ❌ |
| perf_deep_narrow | text | 9.08 | 36 | ❌ |
| perf_deep_narrow | text | 8.70 | 36 | ❌ |
| perf_deep_narrow | text | 8.67 | 36 | ❌ |
| perf_deep_narrow | text | 8.96 | 36 | ❌ |
| huge_document_headings | text | 9.31 | 223 | ❌ |
| huge_document_headings | text | 8.61 | 223 | ❌ |
| huge_document_headings | text | 8.59 | 223 | ❌ |
| huge_document_headings | text | 8.52 | 223 | ❌ |
| huge_document_headings | text | 9.55 | 223 | ❌ |
| huge_document_headings | text | 8.58 | 223 | ❌ |
| huge_document_headings | text | 8.54 | 223 | ❌ |
| huge_document_headings | text | 8.64 | 223 | ❌ |
| huge_document_headings | text | 9.35 | 223 | ❌ |
| huge_document_headings | text | 9.38 | 223 | ❌ |
| many_bold_words | text | 28.67 | 210 | ❌ |
| many_bold_words | text | 29.93 | 210 | ❌ |
| many_bold_words | text | 29.05 | 210 | ❌ |
| many_bold_words | text | 28.41 | 210 | ❌ |
| many_bold_words | text | 29.30 | 210 | ❌ |
| many_bold_words | text | 27.98 | 210 | ❌ |
| many_bold_words | text | 29.56 | 210 | ❌ |
| many_bold_words | text | 29.96 | 210 | ❌ |
| many_bold_words | text | 29.81 | 210 | ❌ |
| many_bold_words | text | 29.07 | 210 | ❌ |
| many_links | text | 26.68 | 141 | ❌ |
| many_links | text | 23.00 | 141 | ❌ |
| many_links | text | 21.86 | 141 | ❌ |
| many_links | text | 20.56 | 141 | ❌ |
| many_links | text | 21.74 | 141 | ❌ |
| many_links | text | 22.18 | 141 | ❌ |
| many_links | text | 21.04 | 141 | ❌ |
| many_links | text | 21.78 | 141 | ❌ |
| many_links | text | 21.03 | 141 | ❌ |
| many_links | text | 21.25 | 141 | ❌ |
| many_footnotes_refs | footnote_ref | 19.38 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 19.24 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 19.13 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 20.01 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 19.80 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 18.79 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 19.74 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 20.03 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 19.78 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 19.12 | 156 | ❌ |
| huge_nested_list | list | 101.63 | 422 | ✅ |
| huge_nested_list | list | 108.35 | 422 | ✅ |
| huge_nested_list | list | 101.65 | 422 | ✅ |
| huge_nested_list | list | 101.29 | 422 | ✅ |
| huge_nested_list | list | 101.48 | 422 | ✅ |
| huge_nested_list | list | 101.39 | 422 | ✅ |
| huge_nested_list | list | 101.36 | 422 | ✅ |
| huge_nested_list | list | 101.27 | 422 | ✅ |
| huge_nested_list | list | 101.18 | 422 | ✅ |
| huge_nested_list | list | 109.85 | 422 | ✅ |
| exponential_nesting | text | 18.61 | 121 | ❌ |
| exponential_nesting | text | 19.28 | 121 | ❌ |
| exponential_nesting | text | 44.55 | 121 | ❌ |
| exponential_nesting | text | 19.65 | 121 | ❌ |
| exponential_nesting | text | 18.45 | 121 | ❌ |
| exponential_nesting | text | 18.10 | 121 | ❌ |
| exponential_nesting | text | 17.66 | 121 | ❌ |
| exponential_nesting | text | 18.46 | 121 | ❌ |
| exponential_nesting | text | 19.16 | 121 | ❌ |
| exponential_nesting | text | 18.57 | 121 | ❌ |
| parse_tree_explosion | text | 23.29 | 38 | ✅ |
| parse_tree_explosion | text | 23.24 | 38 | ✅ |
| parse_tree_explosion | text | 23.28 | 38 | ✅ |
| parse_tree_explosion | text | 23.24 | 38 | ✅ |
| parse_tree_explosion | text | 23.20 | 38 | ✅ |
| parse_tree_explosion | text | 23.32 | 38 | ✅ |
| parse_tree_explosion | text | 23.29 | 38 | ✅ |
| parse_tree_explosion | text | 23.22 | 38 | ✅ |
| parse_tree_explosion | text | 23.23 | 38 | ✅ |
| parse_tree_explosion | text | 23.22 | 38 | ✅ |
| large_table_data | table | 331.20 | 370 | ❌ |
| large_table_data | table | 365.13 | 370 | ❌ |
| large_table_data | table | 334.44 | 370 | ❌ |
| large_table_data | table | 331.91 | 370 | ❌ |
| large_table_data | table | 339.38 | 370 | ❌ |
| large_table_data | table | 330.80 | 370 | ❌ |
| large_table_data | table | 331.75 | 370 | ❌ |
| large_table_data | table | 339.88 | 370 | ❌ |
| large_table_data | table | 331.23 | 370 | ❌ |
| large_table_data | table | 330.21 | 370 | ❌ |
| backtrack_emphasis | text | 11.67 | 65 | ❌ |
| backtrack_emphasis | text | 18.25 | 65 | ❌ |
| backtrack_emphasis | text | 13.01 | 65 | ❌ |
| backtrack_emphasis | text | 12.14 | 65 | ❌ |
| backtrack_emphasis | text | 12.52 | 65 | ❌ |
| backtrack_emphasis | text | 12.29 | 65 | ❌ |
| backtrack_emphasis | text | 12.02 | 65 | ❌ |
| backtrack_emphasis | text | 12.17 | 65 | ❌ |
| backtrack_emphasis | text | 11.95 | 65 | ❌ |
| backtrack_emphasis | text | 11.75 | 65 | ❌ |
| backtrack_links | text | 13.60 | 74 | ❌ |
| backtrack_links | text | 14.19 | 74 | ❌ |
| backtrack_links | text | 13.14 | 74 | ❌ |
| backtrack_links | text | 12.70 | 74 | ❌ |
| backtrack_links | text | 13.69 | 74 | ❌ |
| backtrack_links | text | 13.71 | 74 | ❌ |
| backtrack_links | text | 13.13 | 74 | ❌ |
| backtrack_links | text | 13.77 | 74 | ❌ |
| backtrack_links | text | 13.70 | 74 | ❌ |
| backtrack_links | text | 13.65 | 74 | ❌ |
| backtrack_code | text | 4.42 | 51 | ❌ |
| backtrack_code | text | 4.50 | 51 | ❌ |
| backtrack_code | text | 4.33 | 51 | ❌ |
| backtrack_code | text | 4.32 | 51 | ❌ |
| backtrack_code | text | 4.31 | 51 | ❌ |
| backtrack_code | text | 4.32 | 51 | ❌ |
| backtrack_code | text | 4.30 | 51 | ❌ |
| backtrack_code | text | 4.32 | 51 | ❌ |
| backtrack_code | text | 4.64 | 51 | ❌ |
| backtrack_code | text | 4.35 | 51 | ❌ |
| large_table | table | 246.47 | 169 | ❌ |
| large_table | table | 247.83 | 169 | ❌ |
| large_table | table | 248.12 | 169 | ❌ |
| large_table | table | 256.19 | 169 | ❌ |
| large_table | table | 248.84 | 169 | ❌ |
| large_table | table | 247.48 | 169 | ❌ |
| large_table | table | 247.83 | 169 | ❌ |
| large_table | table | 299.54 | 169 | ❌ |
| large_table | table | 295.55 | 169 | ❌ |
| large_table | table | 248.24 | 169 | ❌ |
| many_footnotes | footnote_ref | 9.10 | 68 | ❌ |
| many_footnotes | footnote_ref | 8.94 | 68 | ❌ |
| many_footnotes | footnote_ref | 9.51 | 68 | ❌ |
| many_footnotes | footnote_ref | 9.22 | 68 | ❌ |
| many_footnotes | footnote_ref | 9.14 | 68 | ❌ |
| many_footnotes | footnote_ref | 9.53 | 68 | ❌ |
| many_footnotes | footnote_ref | 9.47 | 68 | ❌ |
| many_footnotes | footnote_ref | 8.63 | 68 | ❌ |
| many_footnotes | footnote_ref | 9.30 | 68 | ❌ |
| many_footnotes | footnote_ref | 9.19 | 68 | ❌ |
| recursive_refs | reference_link | 4.05 | 32 | ✅ |
| recursive_refs | reference_link | 4.12 | 32 | ✅ |
| recursive_refs | reference_link | 4.06 | 32 | ✅ |
| recursive_refs | reference_link | 4.04 | 32 | ✅ |
| recursive_refs | reference_link | 4.09 | 32 | ✅ |
| recursive_refs | reference_link | 4.12 | 32 | ✅ |
| recursive_refs | reference_link | 4.09 | 32 | ✅ |
| recursive_refs | reference_link | 4.04 | 32 | ✅ |
| recursive_refs | reference_link | 4.06 | 32 | ✅ |
| recursive_refs | reference_link | 10.50 | 32 | ✅ |
| deeply_nested_quotes | blockquote | 128.19 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 128.14 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 127.83 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 128.15 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 136.90 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 129.69 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 128.34 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 127.48 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 127.54 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 127.52 | 68 | ✅ |
| deeply_nested_lists | list | 36.11 | 135 | ✅ |
| deeply_nested_lists | list | 36.33 | 135 | ✅ |
| deeply_nested_lists | list | 38.54 | 135 | ✅ |
| deeply_nested_lists | list | 36.81 | 135 | ✅ |
| deeply_nested_lists | list | 42.26 | 135 | ✅ |
| deeply_nested_lists | list | 36.55 | 135 | ✅ |
| deeply_nested_lists | list | 36.41 | 135 | ✅ |
| deeply_nested_lists | list | 37.24 | 135 | ✅ |
| deeply_nested_lists | list | 36.59 | 135 | ✅ |
| deeply_nested_lists | list | 36.49 | 135 | ✅ |
| deeply_nested_emphasis | emphasis | 5.28 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.37 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.34 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.32 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.29 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.35 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.47 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.37 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.34 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.32 | 52 | ✅ |
| extremely_long_line | text | 330.72 | 1217 | ✅ |
| extremely_long_line | text | 331.41 | 1217 | ✅ |
| extremely_long_line | text | 333.91 | 1217 | ✅ |
| extremely_long_line | text | 330.96 | 1217 | ✅ |
| extremely_long_line | text | 330.05 | 1217 | ✅ |
| extremely_long_line | text | 334.36 | 1217 | ✅ |
| extremely_long_line | text | 330.21 | 1217 | ✅ |
| extremely_long_line | text | 330.67 | 1217 | ✅ |
| extremely_long_line | text | 333.75 | 1217 | ✅ |
| extremely_long_line | text | 330.38 | 1217 | ✅ |
| many_emphasis_markers | text | 13.32 | 77 | ❌ |
| many_emphasis_markers | text | 13.22 | 77 | ❌ |
| many_emphasis_markers | text | 13.14 | 77 | ❌ |
| many_emphasis_markers | text | 14.22 | 77 | ❌ |
| many_emphasis_markers | text | 13.49 | 77 | ❌ |
| many_emphasis_markers | text | 13.80 | 77 | ❌ |
| many_emphasis_markers | text | 13.87 | 77 | ❌ |
| many_emphasis_markers | text | 13.76 | 77 | ❌ |
| many_emphasis_markers | text | 14.08 | 77 | ❌ |
| many_emphasis_markers | text | 14.13 | 77 | ❌ |
| alternating_chars | text | 13.54 | 79 | ❌ |
| alternating_chars | text | 13.24 | 79 | ❌ |
| alternating_chars | text | 13.71 | 79 | ❌ |
| alternating_chars | text | 13.71 | 79 | ❌ |
| alternating_chars | text | 13.81 | 79 | ❌ |
| alternating_chars | text | 14.05 | 79 | ❌ |
| alternating_chars | text | 21.24 | 79 | ❌ |
| alternating_chars | text | 14.36 | 79 | ❌ |
| alternating_chars | text | 13.47 | 79 | ❌ |
| alternating_chars | text | 13.87 | 79 | ❌ |
| quadratic_blowup | text | 28.37 | 198 | ❌ |
| quadratic_blowup | text | 29.39 | 198 | ❌ |
| quadratic_blowup | text | 26.87 | 198 | ❌ |
| quadratic_blowup | text | 26.86 | 198 | ❌ |
| quadratic_blowup | text | 28.17 | 198 | ❌ |
| quadratic_blowup | text | 27.40 | 198 | ❌ |
| quadratic_blowup | text | 28.35 | 198 | ❌ |
| quadratic_blowup | text | 27.55 | 198 | ❌ |
| quadratic_blowup | text | 28.45 | 198 | ❌ |
| quadratic_blowup | text | 28.80 | 198 | ❌ |
| mixed_line_endings_complex | text | 23.89 | 32 | ✅ |
| mixed_line_endings_complex | text | 23.88 | 32 | ✅ |
| mixed_line_endings_complex | text | 23.91 | 32 | ✅ |
| mixed_line_endings_complex | text | 23.86 | 32 | ✅ |
| mixed_line_endings_complex | text | 23.82 | 32 | ✅ |
| mixed_line_endings_complex | text | 23.91 | 32 | ✅ |
| mixed_line_endings_complex | text | 23.91 | 32 | ✅ |
| mixed_line_endings_complex | text | 23.84 | 32 | ✅ |
| mixed_line_endings_complex | text | 23.85 | 32 | ✅ |
| mixed_line_endings_complex | text | 23.97 | 32 | ✅ |
| binary_like_data | text | 81.30 | 103 | ✅ |
| binary_like_data | text | 93.49 | 103 | ✅ |
| binary_like_data | text | 49.24 | 103 | ✅ |
| binary_like_data | text | 46.09 | 103 | ✅ |
| binary_like_data | text | 45.96 | 103 | ✅ |
| binary_like_data | text | 46.03 | 103 | ✅ |
| binary_like_data | text | 46.04 | 103 | ✅ |
| binary_like_data | text | 45.98 | 103 | ✅ |
| binary_like_data | text | 46.01 | 103 | ✅ |
| binary_like_data | text | 45.95 | 103 | ✅ |
| massive_nested_brackets | text | 51.84 | 398 | ❌ |
| massive_nested_brackets | text | 50.88 | 398 | ❌ |
| massive_nested_brackets | text | 53.68 | 398 | ❌ |
| massive_nested_brackets | text | 51.70 | 398 | ❌ |
| massive_nested_brackets | text | 53.03 | 398 | ❌ |
| massive_nested_brackets | text | 51.10 | 398 | ❌ |
| massive_nested_brackets | text | 55.23 | 398 | ❌ |
| massive_nested_brackets | text | 52.04 | 398 | ❌ |
| massive_nested_brackets | text | 52.23 | 398 | ❌ |
| massive_nested_brackets | text | 52.48 | 398 | ❌ |

---
*Report generated by Marco Grammar Test Suite*
