# Marco Grammar Performance Benchmark Report

Generated automatically from benchmark tests

## Summary

- **Total Tests**: 340
- **Passed**: 310 ✅
- **Failed**: 30 ❌
- **Total Time**: 92.31ms
- **Average Parse Time**: 206.69μs
- **Memory Estimate**: 57560 bytes
- **Slowest Test**: large_table_data (1905.72μs)
- **Fastest Test**: perf_complex_formatting (3.50μs)

## Performance Analysis

### Small (0-50 chars)
- Tests: 50
- Average Time: 15.15μs
- Average Size: 33 chars
- Throughput: 2.18 MB/s

### Extra Large (1000+ chars)
- Tests: 10
- Average Time: 397.83μs
- Average Size: 1217 chars
- Throughput: 3.06 MB/s

### Large (201-1000 chars)
- Tests: 80
- Average Time: 463.80μs
- Average Size: 294 chars
- Throughput: 0.63 MB/s

### Medium (51-200 chars)
- Tests: 200
- Average Time: 142.18μs
- Average Size: 100 chars
- Throughput: 0.70 MB/s

## Detailed Results

| Test Name | Rule | Time (μs) | Input Size | Status |
|-----------|------|-----------|------------|--------|
| perf_simple_parse | text | 12.96 | 30 | ✅ |
| perf_simple_parse | text | 12.89 | 30 | ✅ |
| perf_simple_parse | text | 12.93 | 30 | ✅ |
| perf_simple_parse | text | 12.89 | 30 | ✅ |
| perf_simple_parse | text | 12.89 | 30 | ✅ |
| perf_simple_parse | text | 13.05 | 30 | ✅ |
| perf_simple_parse | text | 12.97 | 30 | ✅ |
| perf_simple_parse | text | 12.99 | 30 | ✅ |
| perf_simple_parse | text | 12.91 | 30 | ✅ |
| perf_simple_parse | text | 13.30 | 30 | ✅ |
| perf_complex_formatting | emphasis | 3.52 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.59 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.58 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.50 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.52 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.51 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.50 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.87 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.69 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.63 | 65 | ✅ |
| perf_nested_structures | text | 38.85 | 52 | ✅ |
| perf_nested_structures | text | 38.80 | 52 | ✅ |
| perf_nested_structures | text | 38.89 | 52 | ✅ |
| perf_nested_structures | text | 38.86 | 52 | ✅ |
| perf_nested_structures | text | 38.79 | 52 | ✅ |
| perf_nested_structures | text | 38.83 | 52 | ✅ |
| perf_nested_structures | text | 38.87 | 52 | ✅ |
| perf_nested_structures | text | 38.78 | 52 | ✅ |
| perf_nested_structures | text | 38.73 | 52 | ✅ |
| perf_nested_structures | text | 38.86 | 52 | ✅ |
| perf_large_paragraph | text | 90.02 | 279 | ✅ |
| perf_large_paragraph | text | 90.54 | 279 | ✅ |
| perf_large_paragraph | text | 90.12 | 279 | ✅ |
| perf_large_paragraph | text | 90.49 | 279 | ✅ |
| perf_large_paragraph | text | 90.48 | 279 | ✅ |
| perf_large_paragraph | text | 90.25 | 279 | ✅ |
| perf_large_paragraph | text | 90.33 | 279 | ✅ |
| perf_large_paragraph | text | 90.16 | 279 | ✅ |
| perf_large_paragraph | text | 90.19 | 279 | ✅ |
| perf_large_paragraph | text | 96.28 | 279 | ✅ |
| perf_github_readme | document | 410.92 | 203 | ✅ |
| perf_github_readme | document | 419.78 | 203 | ✅ |
| perf_github_readme | document | 453.37 | 203 | ✅ |
| perf_github_readme | document | 422.13 | 203 | ✅ |
| perf_github_readme | document | 419.17 | 203 | ✅ |
| perf_github_readme | document | 413.69 | 203 | ✅ |
| perf_github_readme | document | 418.22 | 203 | ✅ |
| perf_github_readme | document | 410.85 | 203 | ✅ |
| perf_github_readme | document | 423.75 | 203 | ✅ |
| perf_github_readme | document | 410.91 | 203 | ✅ |
| perf_academic_paper | document | 421.41 | 249 | ✅ |
| perf_academic_paper | document | 410.46 | 249 | ✅ |
| perf_academic_paper | document | 416.67 | 249 | ✅ |
| perf_academic_paper | document | 410.71 | 249 | ✅ |
| perf_academic_paper | document | 416.01 | 249 | ✅ |
| perf_academic_paper | document | 416.67 | 249 | ✅ |
| perf_academic_paper | document | 408.86 | 249 | ✅ |
| perf_academic_paper | document | 423.77 | 249 | ✅ |
| perf_academic_paper | document | 409.54 | 249 | ✅ |
| perf_academic_paper | document | 463.44 | 249 | ✅ |
| perf_many_small_elements | text | 59.24 | 80 | ✅ |
| perf_many_small_elements | text | 59.15 | 80 | ✅ |
| perf_many_small_elements | text | 59.22 | 80 | ✅ |
| perf_many_small_elements | text | 59.08 | 80 | ✅ |
| perf_many_small_elements | text | 59.24 | 80 | ✅ |
| perf_many_small_elements | text | 59.81 | 80 | ✅ |
| perf_many_small_elements | text | 59.15 | 80 | ✅ |
| perf_many_small_elements | text | 59.09 | 80 | ✅ |
| perf_many_small_elements | text | 59.07 | 80 | ✅ |
| perf_many_small_elements | text | 59.06 | 80 | ✅ |
| perf_few_large_elements | text | 84.32 | 184 | ✅ |
| perf_few_large_elements | text | 84.58 | 184 | ✅ |
| perf_few_large_elements | text | 84.25 | 184 | ✅ |
| perf_few_large_elements | text | 84.45 | 184 | ✅ |
| perf_few_large_elements | text | 84.58 | 184 | ✅ |
| perf_few_large_elements | text | 84.42 | 184 | ✅ |
| perf_few_large_elements | text | 84.42 | 184 | ✅ |
| perf_few_large_elements | text | 84.49 | 184 | ✅ |
| perf_few_large_elements | text | 84.45 | 184 | ✅ |
| perf_few_large_elements | text | 84.33 | 184 | ✅ |
| perf_shallow_wide | text | 68.68 | 79 | ✅ |
| perf_shallow_wide | text | 68.89 | 79 | ✅ |
| perf_shallow_wide | text | 68.75 | 79 | ✅ |
| perf_shallow_wide | text | 68.66 | 79 | ✅ |
| perf_shallow_wide | text | 69.04 | 79 | ✅ |
| perf_shallow_wide | text | 68.76 | 79 | ✅ |
| perf_shallow_wide | text | 68.82 | 79 | ✅ |
| perf_shallow_wide | text | 68.83 | 79 | ✅ |
| perf_shallow_wide | text | 68.86 | 79 | ✅ |
| perf_shallow_wide | text | 68.70 | 79 | ✅ |
| perf_deep_narrow | text | 26.09 | 36 | ✅ |
| perf_deep_narrow | text | 32.96 | 36 | ✅ |
| perf_deep_narrow | text | 26.56 | 36 | ✅ |
| perf_deep_narrow | text | 26.12 | 36 | ✅ |
| perf_deep_narrow | text | 26.09 | 36 | ✅ |
| perf_deep_narrow | text | 26.19 | 36 | ✅ |
| perf_deep_narrow | text | 26.15 | 36 | ✅ |
| perf_deep_narrow | text | 26.13 | 36 | ✅ |
| perf_deep_narrow | text | 26.01 | 36 | ✅ |
| perf_deep_narrow | text | 26.13 | 36 | ✅ |
| huge_document_headings | text | 154.74 | 223 | ✅ |
| huge_document_headings | text | 169.09 | 223 | ✅ |
| huge_document_headings | text | 156.01 | 223 | ✅ |
| huge_document_headings | text | 155.13 | 223 | ✅ |
| huge_document_headings | text | 155.05 | 223 | ✅ |
| huge_document_headings | text | 154.64 | 223 | ✅ |
| huge_document_headings | text | 154.86 | 223 | ✅ |
| huge_document_headings | text | 155.06 | 223 | ✅ |
| huge_document_headings | text | 159.03 | 223 | ✅ |
| huge_document_headings | text | 155.22 | 223 | ✅ |
| many_bold_words | text | 186.35 | 210 | ✅ |
| many_bold_words | text | 168.35 | 210 | ✅ |
| many_bold_words | text | 167.68 | 210 | ✅ |
| many_bold_words | text | 167.42 | 210 | ✅ |
| many_bold_words | text | 167.63 | 210 | ✅ |
| many_bold_words | text | 167.39 | 210 | ✅ |
| many_bold_words | text | 173.62 | 210 | ✅ |
| many_bold_words | text | 167.81 | 210 | ✅ |
| many_bold_words | text | 167.50 | 210 | ✅ |
| many_bold_words | text | 167.95 | 210 | ✅ |
| many_links | text | 87.36 | 141 | ✅ |
| many_links | text | 99.47 | 141 | ✅ |
| many_links | text | 87.96 | 141 | ✅ |
| many_links | text | 87.39 | 141 | ✅ |
| many_links | text | 87.23 | 141 | ✅ |
| many_links | text | 87.49 | 141 | ✅ |
| many_links | text | 87.45 | 141 | ✅ |
| many_links | text | 87.21 | 141 | ✅ |
| many_links | text | 87.30 | 141 | ✅ |
| many_links | text | 87.89 | 141 | ✅ |
| many_footnotes_refs | footnote_ref | 17.84 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 18.48 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 18.48 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 17.95 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 18.36 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 18.76 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 18.06 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 18.24 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 18.00 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 18.68 | 156 | ❌ |
| huge_nested_list | list | 113.42 | 422 | ✅ |
| huge_nested_list | list | 113.52 | 422 | ✅ |
| huge_nested_list | list | 113.67 | 422 | ✅ |
| huge_nested_list | list | 113.38 | 422 | ✅ |
| huge_nested_list | list | 113.19 | 422 | ✅ |
| huge_nested_list | list | 129.07 | 422 | ✅ |
| huge_nested_list | list | 114.28 | 422 | ✅ |
| huge_nested_list | list | 113.36 | 422 | ✅ |
| huge_nested_list | list | 113.07 | 422 | ✅ |
| huge_nested_list | list | 113.25 | 422 | ✅ |
| exponential_nesting | text | 77.33 | 121 | ✅ |
| exponential_nesting | text | 77.44 | 121 | ✅ |
| exponential_nesting | text | 90.21 | 121 | ✅ |
| exponential_nesting | text | 106.98 | 121 | ✅ |
| exponential_nesting | text | 78.95 | 121 | ✅ |
| exponential_nesting | text | 77.53 | 121 | ✅ |
| exponential_nesting | text | 77.33 | 121 | ✅ |
| exponential_nesting | text | 77.49 | 121 | ✅ |
| exponential_nesting | text | 77.36 | 121 | ✅ |
| exponential_nesting | text | 77.43 | 121 | ✅ |
| parse_tree_explosion | text | 24.43 | 38 | ✅ |
| parse_tree_explosion | text | 24.36 | 38 | ✅ |
| parse_tree_explosion | text | 24.31 | 38 | ✅ |
| parse_tree_explosion | text | 24.29 | 38 | ✅ |
| parse_tree_explosion | text | 24.31 | 38 | ✅ |
| parse_tree_explosion | text | 24.49 | 38 | ✅ |
| parse_tree_explosion | text | 24.36 | 38 | ✅ |
| parse_tree_explosion | text | 24.28 | 38 | ✅ |
| parse_tree_explosion | text | 24.04 | 38 | ✅ |
| parse_tree_explosion | text | 24.47 | 38 | ✅ |
| large_table_data | table | 1555.90 | 370 | ✅ |
| large_table_data | table | 1545.39 | 370 | ✅ |
| large_table_data | table | 1905.72 | 370 | ✅ |
| large_table_data | table | 1588.92 | 370 | ✅ |
| large_table_data | table | 1560.30 | 370 | ✅ |
| large_table_data | table | 1562.63 | 370 | ✅ |
| large_table_data | table | 1544.83 | 370 | ✅ |
| large_table_data | table | 1553.28 | 370 | ✅ |
| large_table_data | table | 1565.18 | 370 | ✅ |
| large_table_data | table | 1578.18 | 370 | ✅ |
| backtrack_emphasis | text | 82.05 | 65 | ✅ |
| backtrack_emphasis | text | 76.44 | 65 | ✅ |
| backtrack_emphasis | text | 75.52 | 65 | ✅ |
| backtrack_emphasis | text | 75.29 | 65 | ✅ |
| backtrack_emphasis | text | 75.31 | 65 | ✅ |
| backtrack_emphasis | text | 75.43 | 65 | ✅ |
| backtrack_emphasis | text | 75.45 | 65 | ✅ |
| backtrack_emphasis | text | 75.07 | 65 | ✅ |
| backtrack_emphasis | text | 75.57 | 65 | ✅ |
| backtrack_emphasis | text | 75.54 | 65 | ✅ |
| backtrack_links | text | 126.17 | 74 | ✅ |
| backtrack_links | text | 125.57 | 74 | ✅ |
| backtrack_links | text | 125.81 | 74 | ✅ |
| backtrack_links | text | 125.48 | 74 | ✅ |
| backtrack_links | text | 125.77 | 74 | ✅ |
| backtrack_links | text | 125.85 | 74 | ✅ |
| backtrack_links | text | 125.68 | 74 | ✅ |
| backtrack_links | text | 139.56 | 74 | ✅ |
| backtrack_links | text | 126.51 | 74 | ✅ |
| backtrack_links | text | 125.61 | 74 | ✅ |
| backtrack_code | text | 100.49 | 51 | ✅ |
| backtrack_code | text | 100.75 | 51 | ✅ |
| backtrack_code | text | 100.68 | 51 | ✅ |
| backtrack_code | text | 115.53 | 51 | ✅ |
| backtrack_code | text | 101.10 | 51 | ✅ |
| backtrack_code | text | 100.58 | 51 | ✅ |
| backtrack_code | text | 124.06 | 51 | ✅ |
| backtrack_code | text | 160.75 | 51 | ✅ |
| backtrack_code | text | 101.98 | 51 | ✅ |
| backtrack_code | text | 100.66 | 51 | ✅ |
| large_table | table | 1216.85 | 169 | ✅ |
| large_table | table | 1234.19 | 169 | ✅ |
| large_table | table | 1248.44 | 169 | ✅ |
| large_table | table | 1233.61 | 169 | ✅ |
| large_table | table | 1239.43 | 169 | ✅ |
| large_table | table | 1226.44 | 169 | ✅ |
| large_table | table | 1220.17 | 169 | ✅ |
| large_table | table | 1216.94 | 169 | ✅ |
| large_table | table | 1215.47 | 169 | ✅ |
| large_table | table | 1217.64 | 169 | ✅ |
| many_footnotes | footnote_ref | 8.74 | 68 | ❌ |
| many_footnotes | footnote_ref | 9.02 | 68 | ❌ |
| many_footnotes | footnote_ref | 8.58 | 68 | ❌ |
| many_footnotes | footnote_ref | 9.38 | 68 | ❌ |
| many_footnotes | footnote_ref | 8.63 | 68 | ❌ |
| many_footnotes | footnote_ref | 8.46 | 68 | ❌ |
| many_footnotes | footnote_ref | 8.43 | 68 | ❌ |
| many_footnotes | footnote_ref | 13.75 | 68 | ❌ |
| many_footnotes | footnote_ref | 8.65 | 68 | ❌ |
| many_footnotes | footnote_ref | 8.48 | 68 | ❌ |
| recursive_refs | reference_link | 4.31 | 32 | ✅ |
| recursive_refs | reference_link | 4.46 | 32 | ✅ |
| recursive_refs | reference_link | 4.37 | 32 | ✅ |
| recursive_refs | reference_link | 4.27 | 32 | ✅ |
| recursive_refs | reference_link | 4.33 | 32 | ✅ |
| recursive_refs | reference_link | 4.38 | 32 | ✅ |
| recursive_refs | reference_link | 4.30 | 32 | ✅ |
| recursive_refs | reference_link | 4.30 | 32 | ✅ |
| recursive_refs | reference_link | 4.28 | 32 | ✅ |
| recursive_refs | reference_link | 4.29 | 32 | ✅ |
| deeply_nested_quotes | blockquote | 158.09 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 158.37 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 176.32 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 158.95 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 158.02 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 193.73 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 158.93 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 158.24 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 174.04 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 158.51 | 68 | ✅ |
| deeply_nested_lists | list | 40.56 | 135 | ✅ |
| deeply_nested_lists | list | 40.78 | 135 | ✅ |
| deeply_nested_lists | list | 40.73 | 135 | ✅ |
| deeply_nested_lists | list | 40.75 | 135 | ✅ |
| deeply_nested_lists | list | 41.25 | 135 | ✅ |
| deeply_nested_lists | list | 41.25 | 135 | ✅ |
| deeply_nested_lists | list | 41.07 | 135 | ✅ |
| deeply_nested_lists | list | 41.95 | 135 | ✅ |
| deeply_nested_lists | list | 41.07 | 135 | ✅ |
| deeply_nested_lists | list | 41.09 | 135 | ✅ |
| deeply_nested_emphasis | emphasis | 5.34 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.46 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.41 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.67 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.41 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.52 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.40 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.39 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.39 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.70 | 52 | ✅ |
| extremely_long_line | text | 388.59 | 1217 | ✅ |
| extremely_long_line | text | 387.24 | 1217 | ✅ |
| extremely_long_line | text | 448.76 | 1217 | ✅ |
| extremely_long_line | text | 390.28 | 1217 | ✅ |
| extremely_long_line | text | 398.73 | 1217 | ✅ |
| extremely_long_line | text | 387.80 | 1217 | ✅ |
| extremely_long_line | text | 387.41 | 1217 | ✅ |
| extremely_long_line | text | 408.44 | 1217 | ✅ |
| extremely_long_line | text | 389.06 | 1217 | ✅ |
| extremely_long_line | text | 391.98 | 1217 | ✅ |
| many_emphasis_markers | text | 119.92 | 77 | ✅ |
| many_emphasis_markers | text | 119.84 | 77 | ✅ |
| many_emphasis_markers | text | 120.41 | 77 | ✅ |
| many_emphasis_markers | text | 121.09 | 77 | ✅ |
| many_emphasis_markers | text | 131.88 | 77 | ✅ |
| many_emphasis_markers | text | 120.96 | 77 | ✅ |
| many_emphasis_markers | text | 119.88 | 77 | ✅ |
| many_emphasis_markers | text | 119.76 | 77 | ✅ |
| many_emphasis_markers | text | 120.24 | 77 | ✅ |
| many_emphasis_markers | text | 120.40 | 77 | ✅ |
| alternating_chars | text | 129.22 | 79 | ✅ |
| alternating_chars | text | 129.33 | 79 | ✅ |
| alternating_chars | text | 129.25 | 79 | ✅ |
| alternating_chars | text | 128.35 | 79 | ✅ |
| alternating_chars | text | 128.54 | 79 | ✅ |
| alternating_chars | text | 128.63 | 79 | ✅ |
| alternating_chars | text | 128.75 | 79 | ✅ |
| alternating_chars | text | 138.00 | 79 | ✅ |
| alternating_chars | text | 128.46 | 79 | ✅ |
| alternating_chars | text | 129.22 | 79 | ✅ |
| quadratic_blowup | text | 360.06 | 198 | ✅ |
| quadratic_blowup | text | 373.15 | 198 | ✅ |
| quadratic_blowup | text | 360.45 | 198 | ✅ |
| quadratic_blowup | text | 420.03 | 198 | ✅ |
| quadratic_blowup | text | 363.40 | 198 | ✅ |
| quadratic_blowup | text | 359.72 | 198 | ✅ |
| quadratic_blowup | text | 369.40 | 198 | ✅ |
| quadratic_blowup | text | 360.99 | 198 | ✅ |
| quadratic_blowup | text | 361.92 | 198 | ✅ |
| quadratic_blowup | text | 365.51 | 198 | ✅ |
| mixed_line_endings_complex | text | 7.23 | 32 | ✅ |
| mixed_line_endings_complex | text | 7.25 | 32 | ✅ |
| mixed_line_endings_complex | text | 7.23 | 32 | ✅ |
| mixed_line_endings_complex | text | 7.20 | 32 | ✅ |
| mixed_line_endings_complex | text | 7.65 | 32 | ✅ |
| mixed_line_endings_complex | text | 7.23 | 32 | ✅ |
| mixed_line_endings_complex | text | 7.29 | 32 | ✅ |
| mixed_line_endings_complex | text | 7.21 | 32 | ✅ |
| mixed_line_endings_complex | text | 7.23 | 32 | ✅ |
| mixed_line_endings_complex | text | 7.20 | 32 | ✅ |
| binary_like_data | text | 16.89 | 103 | ❌ |
| binary_like_data | text | 17.67 | 103 | ❌ |
| binary_like_data | text | 16.99 | 103 | ❌ |
| binary_like_data | text | 16.92 | 103 | ❌ |
| binary_like_data | text | 17.19 | 103 | ❌ |
| binary_like_data | text | 16.94 | 103 | ❌ |
| binary_like_data | text | 16.91 | 103 | ❌ |
| binary_like_data | text | 17.98 | 103 | ❌ |
| binary_like_data | text | 16.83 | 103 | ❌ |
| binary_like_data | text | 17.82 | 103 | ❌ |
| massive_nested_brackets | text | 736.85 | 398 | ✅ |
| massive_nested_brackets | text | 736.64 | 398 | ✅ |
| massive_nested_brackets | text | 725.64 | 398 | ✅ |
| massive_nested_brackets | text | 736.97 | 398 | ✅ |
| massive_nested_brackets | text | 779.18 | 398 | ✅ |
| massive_nested_brackets | text | 764.29 | 398 | ✅ |
| massive_nested_brackets | text | 734.16 | 398 | ✅ |
| massive_nested_brackets | text | 728.84 | 398 | ✅ |
| massive_nested_brackets | text | 734.71 | 398 | ✅ |
| massive_nested_brackets | text | 736.34 | 398 | ✅ |

---
*Report generated by Marco Grammar Test Suite*
