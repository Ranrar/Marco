# Marco Grammar Performance Benchmark Report

Generated automatically from benchmark tests

## Summary

- **Total Tests**: 340
- **Passed**: 320 ✅
- **Failed**: 20 ❌
- **Total Time**: 84.36ms
- **Average Parse Time**: 188.49μs
- **Memory Estimate**: 57560 bytes
- **Slowest Test**: large_table_data (1329.40μs)
- **Fastest Test**: perf_complex_formatting (3.58μs)

## Performance Analysis

### Small (0-50 chars)
- Tests: 50
- Average Time: 19.47μs
- Average Size: 33 chars
- Throughput: 1.70 MB/s

### Medium (51-200 chars)
- Tests: 200
- Average Time: 130.67μs
- Average Size: 100 chars
- Throughput: 0.77 MB/s

### Extra Large (1000+ chars)
- Tests: 10
- Average Time: 387.64μs
- Average Size: 1217 chars
- Throughput: 3.14 MB/s

### Large (201-1000 chars)
- Tests: 80
- Average Time: 413.81μs
- Average Size: 294 chars
- Throughput: 0.71 MB/s

## Detailed Results

| Test Name | Rule | Time (μs) | Input Size | Status |
|-----------|------|-----------|------------|--------|
| perf_simple_parse | text | 13.22 | 30 | ✅ |
| perf_simple_parse | text | 13.07 | 30 | ✅ |
| perf_simple_parse | text | 13.06 | 30 | ✅ |
| perf_simple_parse | text | 13.04 | 30 | ✅ |
| perf_simple_parse | text | 13.13 | 30 | ✅ |
| perf_simple_parse | text | 19.64 | 30 | ✅ |
| perf_simple_parse | text | 13.35 | 30 | ✅ |
| perf_simple_parse | text | 13.04 | 30 | ✅ |
| perf_simple_parse | text | 13.19 | 30 | ✅ |
| perf_simple_parse | text | 13.36 | 30 | ✅ |
| perf_complex_formatting | emphasis | 3.58 | 65 | ✅ |
| perf_complex_formatting | emphasis | 5.71 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.80 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.61 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.63 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.60 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.59 | 65 | ✅ |
| perf_complex_formatting | emphasis | 4.01 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.74 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.71 | 65 | ✅ |
| perf_nested_structures | text | 38.65 | 52 | ✅ |
| perf_nested_structures | text | 38.68 | 52 | ✅ |
| perf_nested_structures | text | 38.81 | 52 | ✅ |
| perf_nested_structures | text | 38.65 | 52 | ✅ |
| perf_nested_structures | text | 38.65 | 52 | ✅ |
| perf_nested_structures | text | 38.74 | 52 | ✅ |
| perf_nested_structures | text | 38.68 | 52 | ✅ |
| perf_nested_structures | text | 38.64 | 52 | ✅ |
| perf_nested_structures | text | 38.61 | 52 | ✅ |
| perf_nested_structures | text | 38.65 | 52 | ✅ |
| perf_large_paragraph | text | 98.80 | 279 | ✅ |
| perf_large_paragraph | text | 89.76 | 279 | ✅ |
| perf_large_paragraph | text | 89.46 | 279 | ✅ |
| perf_large_paragraph | text | 90.10 | 279 | ✅ |
| perf_large_paragraph | text | 89.45 | 279 | ✅ |
| perf_large_paragraph | text | 89.39 | 279 | ✅ |
| perf_large_paragraph | text | 89.11 | 279 | ✅ |
| perf_large_paragraph | text | 89.08 | 279 | ✅ |
| perf_large_paragraph | text | 88.91 | 279 | ✅ |
| perf_large_paragraph | text | 89.09 | 279 | ✅ |
| perf_github_readme | document | 379.58 | 203 | ✅ |
| perf_github_readme | document | 380.10 | 203 | ✅ |
| perf_github_readme | document | 387.18 | 203 | ✅ |
| perf_github_readme | document | 379.38 | 203 | ✅ |
| perf_github_readme | document | 388.49 | 203 | ✅ |
| perf_github_readme | document | 379.30 | 203 | ✅ |
| perf_github_readme | document | 379.93 | 203 | ✅ |
| perf_github_readme | document | 387.75 | 203 | ✅ |
| perf_github_readme | document | 378.46 | 203 | ✅ |
| perf_github_readme | document | 478.92 | 203 | ✅ |
| perf_academic_paper | document | 381.24 | 249 | ✅ |
| perf_academic_paper | document | 388.69 | 249 | ✅ |
| perf_academic_paper | document | 382.77 | 249 | ✅ |
| perf_academic_paper | document | 381.11 | 249 | ✅ |
| perf_academic_paper | document | 400.35 | 249 | ✅ |
| perf_academic_paper | document | 382.04 | 249 | ✅ |
| perf_academic_paper | document | 382.57 | 249 | ✅ |
| perf_academic_paper | document | 394.44 | 249 | ✅ |
| perf_academic_paper | document | 382.44 | 249 | ✅ |
| perf_academic_paper | document | 392.76 | 249 | ✅ |
| perf_many_small_elements | text | 59.18 | 80 | ✅ |
| perf_many_small_elements | text | 59.01 | 80 | ✅ |
| perf_many_small_elements | text | 59.06 | 80 | ✅ |
| perf_many_small_elements | text | 58.95 | 80 | ✅ |
| perf_many_small_elements | text | 59.01 | 80 | ✅ |
| perf_many_small_elements | text | 59.63 | 80 | ✅ |
| perf_many_small_elements | text | 59.05 | 80 | ✅ |
| perf_many_small_elements | text | 58.93 | 80 | ✅ |
| perf_many_small_elements | text | 59.04 | 80 | ✅ |
| perf_many_small_elements | text | 58.99 | 80 | ✅ |
| perf_few_large_elements | text | 81.50 | 184 | ✅ |
| perf_few_large_elements | text | 81.83 | 184 | ✅ |
| perf_few_large_elements | text | 81.64 | 184 | ✅ |
| perf_few_large_elements | text | 81.58 | 184 | ✅ |
| perf_few_large_elements | text | 81.67 | 184 | ✅ |
| perf_few_large_elements | text | 81.59 | 184 | ✅ |
| perf_few_large_elements | text | 81.72 | 184 | ✅ |
| perf_few_large_elements | text | 81.61 | 184 | ✅ |
| perf_few_large_elements | text | 81.51 | 184 | ✅ |
| perf_few_large_elements | text | 81.55 | 184 | ✅ |
| perf_shallow_wide | text | 68.51 | 79 | ✅ |
| perf_shallow_wide | text | 68.53 | 79 | ✅ |
| perf_shallow_wide | text | 68.59 | 79 | ✅ |
| perf_shallow_wide | text | 68.77 | 79 | ✅ |
| perf_shallow_wide | text | 68.46 | 79 | ✅ |
| perf_shallow_wide | text | 68.60 | 79 | ✅ |
| perf_shallow_wide | text | 68.58 | 79 | ✅ |
| perf_shallow_wide | text | 68.52 | 79 | ✅ |
| perf_shallow_wide | text | 68.53 | 79 | ✅ |
| perf_shallow_wide | text | 68.55 | 79 | ✅ |
| perf_deep_narrow | text | 51.29 | 36 | ✅ |
| perf_deep_narrow | text | 26.41 | 36 | ✅ |
| perf_deep_narrow | text | 25.85 | 36 | ✅ |
| perf_deep_narrow | text | 25.79 | 36 | ✅ |
| perf_deep_narrow | text | 25.82 | 36 | ✅ |
| perf_deep_narrow | text | 25.90 | 36 | ✅ |
| perf_deep_narrow | text | 25.89 | 36 | ✅ |
| perf_deep_narrow | text | 25.87 | 36 | ✅ |
| perf_deep_narrow | text | 25.79 | 36 | ✅ |
| perf_deep_narrow | text | 25.93 | 36 | ✅ |
| huge_document_headings | text | 150.94 | 223 | ✅ |
| huge_document_headings | text | 157.47 | 223 | ✅ |
| huge_document_headings | text | 151.30 | 223 | ✅ |
| huge_document_headings | text | 151.19 | 223 | ✅ |
| huge_document_headings | text | 151.22 | 223 | ✅ |
| huge_document_headings | text | 151.15 | 223 | ✅ |
| huge_document_headings | text | 151.22 | 223 | ✅ |
| huge_document_headings | text | 151.30 | 223 | ✅ |
| huge_document_headings | text | 157.61 | 223 | ✅ |
| huge_document_headings | text | 151.27 | 223 | ✅ |
| many_bold_words | text | 168.35 | 210 | ✅ |
| many_bold_words | text | 173.61 | 210 | ✅ |
| many_bold_words | text | 168.32 | 210 | ✅ |
| many_bold_words | text | 167.59 | 210 | ✅ |
| many_bold_words | text | 167.83 | 210 | ✅ |
| many_bold_words | text | 246.83 | 210 | ✅ |
| many_bold_words | text | 176.87 | 210 | ✅ |
| many_bold_words | text | 168.78 | 210 | ✅ |
| many_bold_words | text | 168.24 | 210 | ✅ |
| many_bold_words | text | 167.82 | 210 | ✅ |
| many_links | text | 87.39 | 141 | ✅ |
| many_links | text | 93.12 | 141 | ✅ |
| many_links | text | 87.41 | 141 | ✅ |
| many_links | text | 87.31 | 141 | ✅ |
| many_links | text | 87.30 | 141 | ✅ |
| many_links | text | 87.29 | 141 | ✅ |
| many_links | text | 87.35 | 141 | ✅ |
| many_links | text | 87.39 | 141 | ✅ |
| many_links | text | 87.22 | 141 | ✅ |
| many_links | text | 87.97 | 141 | ✅ |
| many_footnotes_refs | footnote_ref | 18.97 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 19.97 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 17.81 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 20.03 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 18.30 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 20.14 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 19.27 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 19.64 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 18.55 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 24.54 | 156 | ❌ |
| huge_nested_list | list | 110.72 | 422 | ✅ |
| huge_nested_list | list | 110.75 | 422 | ✅ |
| huge_nested_list | list | 111.02 | 422 | ✅ |
| huge_nested_list | list | 110.97 | 422 | ✅ |
| huge_nested_list | list | 110.66 | 422 | ✅ |
| huge_nested_list | list | 117.17 | 422 | ✅ |
| huge_nested_list | list | 110.69 | 422 | ✅ |
| huge_nested_list | list | 110.73 | 422 | ✅ |
| huge_nested_list | list | 110.91 | 422 | ✅ |
| huge_nested_list | list | 110.90 | 422 | ✅ |
| exponential_nesting | text | 77.14 | 121 | ✅ |
| exponential_nesting | text | 77.37 | 121 | ✅ |
| exponential_nesting | text | 77.40 | 121 | ✅ |
| exponential_nesting | text | 83.84 | 121 | ✅ |
| exponential_nesting | text | 77.30 | 121 | ✅ |
| exponential_nesting | text | 77.01 | 121 | ✅ |
| exponential_nesting | text | 76.95 | 121 | ✅ |
| exponential_nesting | text | 76.92 | 121 | ✅ |
| exponential_nesting | text | 77.03 | 121 | ✅ |
| exponential_nesting | text | 76.96 | 121 | ✅ |
| parse_tree_explosion | text | 24.62 | 38 | ✅ |
| parse_tree_explosion | text | 24.66 | 38 | ✅ |
| parse_tree_explosion | text | 24.57 | 38 | ✅ |
| parse_tree_explosion | text | 24.54 | 38 | ✅ |
| parse_tree_explosion | text | 24.83 | 38 | ✅ |
| parse_tree_explosion | text | 24.74 | 38 | ✅ |
| parse_tree_explosion | text | 24.94 | 38 | ✅ |
| parse_tree_explosion | text | 24.67 | 38 | ✅ |
| parse_tree_explosion | text | 24.59 | 38 | ✅ |
| parse_tree_explosion | text | 24.63 | 38 | ✅ |
| large_table_data | table | 1254.71 | 370 | ✅ |
| large_table_data | table | 1251.21 | 370 | ✅ |
| large_table_data | table | 1244.81 | 370 | ✅ |
| large_table_data | table | 1261.73 | 370 | ✅ |
| large_table_data | table | 1256.04 | 370 | ✅ |
| large_table_data | table | 1329.40 | 370 | ✅ |
| large_table_data | table | 1250.49 | 370 | ✅ |
| large_table_data | table | 1248.41 | 370 | ✅ |
| large_table_data | table | 1256.77 | 370 | ✅ |
| large_table_data | table | 1244.98 | 370 | ✅ |
| backtrack_emphasis | text | 75.29 | 65 | ✅ |
| backtrack_emphasis | text | 75.03 | 65 | ✅ |
| backtrack_emphasis | text | 75.12 | 65 | ✅ |
| backtrack_emphasis | text | 75.34 | 65 | ✅ |
| backtrack_emphasis | text | 81.84 | 65 | ✅ |
| backtrack_emphasis | text | 75.85 | 65 | ✅ |
| backtrack_emphasis | text | 75.63 | 65 | ✅ |
| backtrack_emphasis | text | 75.49 | 65 | ✅ |
| backtrack_emphasis | text | 75.82 | 65 | ✅ |
| backtrack_emphasis | text | 75.11 | 65 | ✅ |
| backtrack_links | text | 126.16 | 74 | ✅ |
| backtrack_links | text | 132.24 | 74 | ✅ |
| backtrack_links | text | 126.50 | 74 | ✅ |
| backtrack_links | text | 126.06 | 74 | ✅ |
| backtrack_links | text | 126.18 | 74 | ✅ |
| backtrack_links | text | 125.86 | 74 | ✅ |
| backtrack_links | text | 126.18 | 74 | ✅ |
| backtrack_links | text | 125.90 | 74 | ✅ |
| backtrack_links | text | 125.96 | 74 | ✅ |
| backtrack_links | text | 131.16 | 74 | ✅ |
| backtrack_code | text | 99.96 | 51 | ✅ |
| backtrack_code | text | 99.72 | 51 | ✅ |
| backtrack_code | text | 100.07 | 51 | ✅ |
| backtrack_code | text | 99.83 | 51 | ✅ |
| backtrack_code | text | 99.89 | 51 | ✅ |
| backtrack_code | text | 99.89 | 51 | ✅ |
| backtrack_code | text | 103.97 | 51 | ✅ |
| backtrack_code | text | 100.45 | 51 | ✅ |
| backtrack_code | text | 99.62 | 51 | ✅ |
| backtrack_code | text | 99.68 | 51 | ✅ |
| large_table | table | 980.79 | 169 | ✅ |
| large_table | table | 981.47 | 169 | ✅ |
| large_table | table | 985.74 | 169 | ✅ |
| large_table | table | 983.02 | 169 | ✅ |
| large_table | table | 1062.36 | 169 | ✅ |
| large_table | table | 982.60 | 169 | ✅ |
| large_table | table | 981.94 | 169 | ✅ |
| large_table | table | 982.92 | 169 | ✅ |
| large_table | table | 986.92 | 169 | ✅ |
| large_table | table | 981.38 | 169 | ✅ |
| many_footnotes | footnote_ref | 9.14 | 68 | ❌ |
| many_footnotes | footnote_ref | 9.35 | 68 | ❌ |
| many_footnotes | footnote_ref | 8.63 | 68 | ❌ |
| many_footnotes | footnote_ref | 8.65 | 68 | ❌ |
| many_footnotes | footnote_ref | 9.10 | 68 | ❌ |
| many_footnotes | footnote_ref | 8.67 | 68 | ❌ |
| many_footnotes | footnote_ref | 8.68 | 68 | ❌ |
| many_footnotes | footnote_ref | 9.38 | 68 | ❌ |
| many_footnotes | footnote_ref | 8.80 | 68 | ❌ |
| many_footnotes | footnote_ref | 8.71 | 68 | ❌ |
| recursive_refs | reference_link | 4.16 | 32 | ✅ |
| recursive_refs | reference_link | 4.33 | 32 | ✅ |
| recursive_refs | reference_link | 4.25 | 32 | ✅ |
| recursive_refs | reference_link | 4.20 | 32 | ✅ |
| recursive_refs | reference_link | 4.20 | 32 | ✅ |
| recursive_refs | reference_link | 4.28 | 32 | ✅ |
| recursive_refs | reference_link | 4.23 | 32 | ✅ |
| recursive_refs | reference_link | 4.17 | 32 | ✅ |
| recursive_refs | reference_link | 4.19 | 32 | ✅ |
| recursive_refs | reference_link | 4.19 | 32 | ✅ |
| deeply_nested_quotes | blockquote | 160.29 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 151.16 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 149.94 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 150.22 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 150.59 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 149.85 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 157.91 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 150.66 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 150.09 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 149.92 | 68 | ✅ |
| deeply_nested_lists | list | 40.15 | 135 | ✅ |
| deeply_nested_lists | list | 40.92 | 135 | ✅ |
| deeply_nested_lists | list | 40.70 | 135 | ✅ |
| deeply_nested_lists | list | 40.54 | 135 | ✅ |
| deeply_nested_lists | list | 41.14 | 135 | ✅ |
| deeply_nested_lists | list | 40.67 | 135 | ✅ |
| deeply_nested_lists | list | 40.44 | 135 | ✅ |
| deeply_nested_lists | list | 41.49 | 135 | ✅ |
| deeply_nested_lists | list | 48.01 | 135 | ✅ |
| deeply_nested_lists | list | 40.52 | 135 | ✅ |
| deeply_nested_emphasis | emphasis | 5.39 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.46 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.42 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.46 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.49 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.55 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.43 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.46 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.43 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.44 | 52 | ✅ |
| extremely_long_line | text | 385.02 | 1217 | ✅ |
| extremely_long_line | text | 391.77 | 1217 | ✅ |
| extremely_long_line | text | 386.60 | 1217 | ✅ |
| extremely_long_line | text | 386.48 | 1217 | ✅ |
| extremely_long_line | text | 390.03 | 1217 | ✅ |
| extremely_long_line | text | 384.72 | 1217 | ✅ |
| extremely_long_line | text | 391.63 | 1217 | ✅ |
| extremely_long_line | text | 385.14 | 1217 | ✅ |
| extremely_long_line | text | 385.17 | 1217 | ✅ |
| extremely_long_line | text | 389.81 | 1217 | ✅ |
| many_emphasis_markers | text | 120.55 | 77 | ✅ |
| many_emphasis_markers | text | 120.51 | 77 | ✅ |
| many_emphasis_markers | text | 120.85 | 77 | ✅ |
| many_emphasis_markers | text | 125.58 | 77 | ✅ |
| many_emphasis_markers | text | 120.71 | 77 | ✅ |
| many_emphasis_markers | text | 119.98 | 77 | ✅ |
| many_emphasis_markers | text | 137.04 | 77 | ✅ |
| many_emphasis_markers | text | 129.52 | 77 | ✅ |
| many_emphasis_markers | text | 121.41 | 77 | ✅ |
| many_emphasis_markers | text | 120.32 | 77 | ✅ |
| alternating_chars | text | 129.61 | 79 | ✅ |
| alternating_chars | text | 130.33 | 79 | ✅ |
| alternating_chars | text | 129.90 | 79 | ✅ |
| alternating_chars | text | 129.88 | 79 | ✅ |
| alternating_chars | text | 129.43 | 79 | ✅ |
| alternating_chars | text | 134.19 | 79 | ✅ |
| alternating_chars | text | 130.01 | 79 | ✅ |
| alternating_chars | text | 129.77 | 79 | ✅ |
| alternating_chars | text | 129.46 | 79 | ✅ |
| alternating_chars | text | 130.23 | 79 | ✅ |
| quadratic_blowup | text | 373.84 | 198 | ✅ |
| quadratic_blowup | text | 369.32 | 198 | ✅ |
| quadratic_blowup | text | 369.22 | 198 | ✅ |
| quadratic_blowup | text | 373.74 | 198 | ✅ |
| quadratic_blowup | text | 368.63 | 198 | ✅ |
| quadratic_blowup | text | 369.90 | 198 | ✅ |
| quadratic_blowup | text | 372.90 | 198 | ✅ |
| quadratic_blowup | text | 368.94 | 198 | ✅ |
| quadratic_blowup | text | 373.51 | 198 | ✅ |
| quadratic_blowup | text | 368.45 | 198 | ✅ |
| mixed_line_endings_complex | text | 26.15 | 32 | ✅ |
| mixed_line_endings_complex | text | 26.28 | 32 | ✅ |
| mixed_line_endings_complex | text | 26.20 | 32 | ✅ |
| mixed_line_endings_complex | text | 26.11 | 32 | ✅ |
| mixed_line_endings_complex | text | 26.23 | 32 | ✅ |
| mixed_line_endings_complex | text | 26.24 | 32 | ✅ |
| mixed_line_endings_complex | text | 26.09 | 32 | ✅ |
| mixed_line_endings_complex | text | 26.21 | 32 | ✅ |
| mixed_line_endings_complex | text | 26.15 | 32 | ✅ |
| mixed_line_endings_complex | text | 26.20 | 32 | ✅ |
| binary_like_data | text | 54.05 | 103 | ✅ |
| binary_like_data | text | 48.60 | 103 | ✅ |
| binary_like_data | text | 48.44 | 103 | ✅ |
| binary_like_data | text | 48.25 | 103 | ✅ |
| binary_like_data | text | 48.29 | 103 | ✅ |
| binary_like_data | text | 48.29 | 103 | ✅ |
| binary_like_data | text | 48.36 | 103 | ✅ |
| binary_like_data | text | 48.30 | 103 | ✅ |
| binary_like_data | text | 48.27 | 103 | ✅ |
| binary_like_data | text | 48.20 | 103 | ✅ |
| massive_nested_brackets | text | 744.49 | 398 | ✅ |
| massive_nested_brackets | text | 737.98 | 398 | ✅ |
| massive_nested_brackets | text | 733.47 | 398 | ✅ |
| massive_nested_brackets | text | 741.61 | 398 | ✅ |
| massive_nested_brackets | text | 739.45 | 398 | ✅ |
| massive_nested_brackets | text | 739.28 | 398 | ✅ |
| massive_nested_brackets | text | 735.04 | 398 | ✅ |
| massive_nested_brackets | text | 751.65 | 398 | ✅ |
| massive_nested_brackets | text | 739.68 | 398 | ✅ |
| massive_nested_brackets | text | 739.39 | 398 | ✅ |

---
*Report generated by Marco Grammar Test Suite*
