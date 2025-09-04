# Marco Grammar Performance Benchmark Report

Generated automatically from benchmark tests

## Summary

- **Total Tests**: 340
- **Passed**: 320 ✅
- **Failed**: 20 ❌
- **Total Time**: 95.20ms
- **Average Parse Time**: 213.26μs
- **Memory Estimate**: 57560 bytes
- **Slowest Test**: large_table (1770.08μs)
- **Fastest Test**: perf_complex_formatting (3.53μs)

## Performance Analysis

### Small (0-50 chars)
- Tests: 50
- Average Time: 20.09μs
- Average Size: 33 chars
- Throughput: 1.64 MB/s

### Large (201-1000 chars)
- Tests: 80
- Average Time: 461.73μs
- Average Size: 294 chars
- Throughput: 0.64 MB/s

### Extra Large (1000+ chars)
- Tests: 10
- Average Time: 406.57μs
- Average Size: 1217 chars
- Throughput: 2.99 MB/s

### Medium (51-200 chars)
- Tests: 200
- Average Time: 152.50μs
- Average Size: 100 chars
- Throughput: 0.66 MB/s

## Detailed Results

| Test Name | Rule | Time (μs) | Input Size | Status |
|-----------|------|-----------|------------|--------|
| perf_simple_parse | text | 13.21 | 30 | ✅ |
| perf_simple_parse | text | 13.03 | 30 | ✅ |
| perf_simple_parse | text | 12.96 | 30 | ✅ |
| perf_simple_parse | text | 12.90 | 30 | ✅ |
| perf_simple_parse | text | 13.03 | 30 | ✅ |
| perf_simple_parse | text | 13.13 | 30 | ✅ |
| perf_simple_parse | text | 13.05 | 30 | ✅ |
| perf_simple_parse | text | 13.04 | 30 | ✅ |
| perf_simple_parse | text | 13.05 | 30 | ✅ |
| perf_simple_parse | text | 13.26 | 30 | ✅ |
| perf_complex_formatting | emphasis | 3.53 | 65 | ✅ |
| perf_complex_formatting | emphasis | 5.87 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.77 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.54 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.62 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.58 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.54 | 65 | ✅ |
| perf_complex_formatting | emphasis | 4.04 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.82 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.66 | 65 | ✅ |
| perf_nested_structures | text | 78.68 | 52 | ✅ |
| perf_nested_structures | text | 74.68 | 52 | ✅ |
| perf_nested_structures | text | 76.58 | 52 | ✅ |
| perf_nested_structures | text | 77.98 | 52 | ✅ |
| perf_nested_structures | text | 84.37 | 52 | ✅ |
| perf_nested_structures | text | 80.88 | 52 | ✅ |
| perf_nested_structures | text | 79.77 | 52 | ✅ |
| perf_nested_structures | text | 89.84 | 52 | ✅ |
| perf_nested_structures | text | 77.13 | 52 | ✅ |
| perf_nested_structures | text | 77.23 | 52 | ✅ |
| perf_large_paragraph | text | 179.59 | 279 | ✅ |
| perf_large_paragraph | text | 172.33 | 279 | ✅ |
| perf_large_paragraph | text | 163.04 | 279 | ✅ |
| perf_large_paragraph | text | 166.90 | 279 | ✅ |
| perf_large_paragraph | text | 165.08 | 279 | ✅ |
| perf_large_paragraph | text | 211.96 | 279 | ✅ |
| perf_large_paragraph | text | 167.11 | 279 | ✅ |
| perf_large_paragraph | text | 218.45 | 279 | ✅ |
| perf_large_paragraph | text | 168.68 | 279 | ✅ |
| perf_large_paragraph | text | 169.98 | 279 | ✅ |
| perf_github_readme | document | 746.40 | 203 | ✅ |
| perf_github_readme | document | 591.07 | 203 | ✅ |
| perf_github_readme | document | 490.37 | 203 | ✅ |
| perf_github_readme | document | 527.28 | 203 | ✅ |
| perf_github_readme | document | 532.84 | 203 | ✅ |
| perf_github_readme | document | 533.99 | 203 | ✅ |
| perf_github_readme | document | 486.84 | 203 | ✅ |
| perf_github_readme | document | 499.41 | 203 | ✅ |
| perf_github_readme | document | 496.63 | 203 | ✅ |
| perf_github_readme | document | 475.35 | 203 | ✅ |
| perf_academic_paper | document | 385.80 | 249 | ✅ |
| perf_academic_paper | document | 392.08 | 249 | ✅ |
| perf_academic_paper | document | 388.53 | 249 | ✅ |
| perf_academic_paper | document | 408.22 | 249 | ✅ |
| perf_academic_paper | document | 399.93 | 249 | ✅ |
| perf_academic_paper | document | 385.99 | 249 | ✅ |
| perf_academic_paper | document | 384.98 | 249 | ✅ |
| perf_academic_paper | document | 534.71 | 249 | ✅ |
| perf_academic_paper | document | 493.77 | 249 | ✅ |
| perf_academic_paper | document | 488.26 | 249 | ✅ |
| perf_many_small_elements | text | 74.06 | 80 | ✅ |
| perf_many_small_elements | text | 74.18 | 80 | ✅ |
| perf_many_small_elements | text | 73.97 | 80 | ✅ |
| perf_many_small_elements | text | 83.06 | 80 | ✅ |
| perf_many_small_elements | text | 74.21 | 80 | ✅ |
| perf_many_small_elements | text | 74.65 | 80 | ✅ |
| perf_many_small_elements | text | 74.25 | 80 | ✅ |
| perf_many_small_elements | text | 74.17 | 80 | ✅ |
| perf_many_small_elements | text | 74.00 | 80 | ✅ |
| perf_many_small_elements | text | 74.14 | 80 | ✅ |
| perf_few_large_elements | text | 102.06 | 184 | ✅ |
| perf_few_large_elements | text | 108.59 | 184 | ✅ |
| perf_few_large_elements | text | 102.27 | 184 | ✅ |
| perf_few_large_elements | text | 101.84 | 184 | ✅ |
| perf_few_large_elements | text | 101.99 | 184 | ✅ |
| perf_few_large_elements | text | 101.94 | 184 | ✅ |
| perf_few_large_elements | text | 102.02 | 184 | ✅ |
| perf_few_large_elements | text | 101.92 | 184 | ✅ |
| perf_few_large_elements | text | 101.83 | 184 | ✅ |
| perf_few_large_elements | text | 102.01 | 184 | ✅ |
| perf_shallow_wide | text | 86.37 | 79 | ✅ |
| perf_shallow_wide | text | 86.51 | 79 | ✅ |
| perf_shallow_wide | text | 86.61 | 79 | ✅ |
| perf_shallow_wide | text | 84.11 | 79 | ✅ |
| perf_shallow_wide | text | 69.42 | 79 | ✅ |
| perf_shallow_wide | text | 69.32 | 79 | ✅ |
| perf_shallow_wide | text | 69.31 | 79 | ✅ |
| perf_shallow_wide | text | 69.27 | 79 | ✅ |
| perf_shallow_wide | text | 69.25 | 79 | ✅ |
| perf_shallow_wide | text | 69.36 | 79 | ✅ |
| perf_deep_narrow | text | 26.12 | 36 | ✅ |
| perf_deep_narrow | text | 25.89 | 36 | ✅ |
| perf_deep_narrow | text | 25.89 | 36 | ✅ |
| perf_deep_narrow | text | 25.98 | 36 | ✅ |
| perf_deep_narrow | text | 25.92 | 36 | ✅ |
| perf_deep_narrow | text | 26.00 | 36 | ✅ |
| perf_deep_narrow | text | 25.95 | 36 | ✅ |
| perf_deep_narrow | text | 25.96 | 36 | ✅ |
| perf_deep_narrow | text | 25.92 | 36 | ✅ |
| perf_deep_narrow | text | 26.01 | 36 | ✅ |
| huge_document_headings | text | 151.64 | 223 | ✅ |
| huge_document_headings | text | 156.85 | 223 | ✅ |
| huge_document_headings | text | 152.06 | 223 | ✅ |
| huge_document_headings | text | 151.70 | 223 | ✅ |
| huge_document_headings | text | 152.44 | 223 | ✅ |
| huge_document_headings | text | 151.84 | 223 | ✅ |
| huge_document_headings | text | 151.69 | 223 | ✅ |
| huge_document_headings | text | 151.86 | 223 | ✅ |
| huge_document_headings | text | 155.70 | 223 | ✅ |
| huge_document_headings | text | 151.81 | 223 | ✅ |
| many_bold_words | text | 174.93 | 210 | ✅ |
| many_bold_words | text | 168.88 | 210 | ✅ |
| many_bold_words | text | 168.85 | 210 | ✅ |
| many_bold_words | text | 169.04 | 210 | ✅ |
| many_bold_words | text | 169.34 | 210 | ✅ |
| many_bold_words | text | 169.31 | 210 | ✅ |
| many_bold_words | text | 172.72 | 210 | ✅ |
| many_bold_words | text | 169.22 | 210 | ✅ |
| many_bold_words | text | 169.10 | 210 | ✅ |
| many_bold_words | text | 169.08 | 210 | ✅ |
| many_links | text | 87.30 | 141 | ✅ |
| many_links | text | 91.81 | 141 | ✅ |
| many_links | text | 87.44 | 141 | ✅ |
| many_links | text | 87.42 | 141 | ✅ |
| many_links | text | 87.32 | 141 | ✅ |
| many_links | text | 87.34 | 141 | ✅ |
| many_links | text | 87.27 | 141 | ✅ |
| many_links | text | 87.33 | 141 | ✅ |
| many_links | text | 87.31 | 141 | ✅ |
| many_links | text | 87.97 | 141 | ✅ |
| many_footnotes_refs | footnote_ref | 21.53 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 20.46 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 19.48 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 20.00 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 20.18 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 23.98 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 20.47 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 20.01 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 19.15 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 17.97 | 156 | ❌ |
| huge_nested_list | list | 110.02 | 422 | ✅ |
| huge_nested_list | list | 110.14 | 422 | ✅ |
| huge_nested_list | list | 110.26 | 422 | ✅ |
| huge_nested_list | list | 110.46 | 422 | ✅ |
| huge_nested_list | list | 116.92 | 422 | ✅ |
| huge_nested_list | list | 110.46 | 422 | ✅ |
| huge_nested_list | list | 118.89 | 422 | ✅ |
| huge_nested_list | list | 124.64 | 422 | ✅ |
| huge_nested_list | list | 110.72 | 422 | ✅ |
| huge_nested_list | list | 110.24 | 422 | ✅ |
| exponential_nesting | text | 77.48 | 121 | ✅ |
| exponential_nesting | text | 82.90 | 121 | ✅ |
| exponential_nesting | text | 77.60 | 121 | ✅ |
| exponential_nesting | text | 77.28 | 121 | ✅ |
| exponential_nesting | text | 77.23 | 121 | ✅ |
| exponential_nesting | text | 77.23 | 121 | ✅ |
| exponential_nesting | text | 77.14 | 121 | ✅ |
| exponential_nesting | text | 77.28 | 121 | ✅ |
| exponential_nesting | text | 77.28 | 121 | ✅ |
| exponential_nesting | text | 77.23 | 121 | ✅ |
| parse_tree_explosion | text | 57.20 | 38 | ✅ |
| parse_tree_explosion | text | 38.76 | 38 | ✅ |
| parse_tree_explosion | text | 32.39 | 38 | ✅ |
| parse_tree_explosion | text | 25.50 | 38 | ✅ |
| parse_tree_explosion | text | 24.73 | 38 | ✅ |
| parse_tree_explosion | text | 24.81 | 38 | ✅ |
| parse_tree_explosion | text | 30.54 | 38 | ✅ |
| parse_tree_explosion | text | 25.53 | 38 | ✅ |
| parse_tree_explosion | text | 24.81 | 38 | ✅ |
| parse_tree_explosion | text | 24.72 | 38 | ✅ |
| large_table_data | table | 1340.81 | 370 | ✅ |
| large_table_data | table | 1302.69 | 370 | ✅ |
| large_table_data | table | 1320.98 | 370 | ✅ |
| large_table_data | table | 1315.91 | 370 | ✅ |
| large_table_data | table | 1442.74 | 370 | ✅ |
| large_table_data | table | 1306.17 | 370 | ✅ |
| large_table_data | table | 1295.91 | 370 | ✅ |
| large_table_data | table | 1457.20 | 370 | ✅ |
| large_table_data | table | 1410.65 | 370 | ✅ |
| large_table_data | table | 1302.78 | 370 | ✅ |
| backtrack_emphasis | text | 125.90 | 65 | ✅ |
| backtrack_emphasis | text | 75.99 | 65 | ✅ |
| backtrack_emphasis | text | 75.38 | 65 | ✅ |
| backtrack_emphasis | text | 75.38 | 65 | ✅ |
| backtrack_emphasis | text | 75.31 | 65 | ✅ |
| backtrack_emphasis | text | 75.39 | 65 | ✅ |
| backtrack_emphasis | text | 75.27 | 65 | ✅ |
| backtrack_emphasis | text | 75.68 | 65 | ✅ |
| backtrack_emphasis | text | 75.52 | 65 | ✅ |
| backtrack_emphasis | text | 75.38 | 65 | ✅ |
| backtrack_links | text | 127.24 | 74 | ✅ |
| backtrack_links | text | 127.32 | 74 | ✅ |
| backtrack_links | text | 126.90 | 74 | ✅ |
| backtrack_links | text | 127.18 | 74 | ✅ |
| backtrack_links | text | 127.09 | 74 | ✅ |
| backtrack_links | text | 147.95 | 74 | ✅ |
| backtrack_links | text | 129.26 | 74 | ✅ |
| backtrack_links | text | 127.08 | 74 | ✅ |
| backtrack_links | text | 126.87 | 74 | ✅ |
| backtrack_links | text | 126.75 | 74 | ✅ |
| backtrack_code | text | 100.81 | 51 | ✅ |
| backtrack_code | text | 108.25 | 51 | ✅ |
| backtrack_code | text | 101.38 | 51 | ✅ |
| backtrack_code | text | 100.85 | 51 | ✅ |
| backtrack_code | text | 140.89 | 51 | ✅ |
| backtrack_code | text | 104.39 | 51 | ✅ |
| backtrack_code | text | 101.07 | 51 | ✅ |
| backtrack_code | text | 101.11 | 51 | ✅ |
| backtrack_code | text | 100.99 | 51 | ✅ |
| backtrack_code | text | 100.92 | 51 | ✅ |
| large_table | table | 1086.63 | 169 | ✅ |
| large_table | table | 1021.23 | 169 | ✅ |
| large_table | table | 1627.84 | 169 | ✅ |
| large_table | table | 1770.08 | 169 | ✅ |
| large_table | table | 1584.38 | 169 | ✅ |
| large_table | table | 1663.37 | 169 | ✅ |
| large_table | table | 1057.99 | 169 | ✅ |
| large_table | table | 1030.03 | 169 | ✅ |
| large_table | table | 1034.25 | 169 | ✅ |
| large_table | table | 1059.12 | 169 | ✅ |
| many_footnotes | footnote_ref | 8.83 | 68 | ❌ |
| many_footnotes | footnote_ref | 9.63 | 68 | ❌ |
| many_footnotes | footnote_ref | 9.42 | 68 | ❌ |
| many_footnotes | footnote_ref | 9.58 | 68 | ❌ |
| many_footnotes | footnote_ref | 9.26 | 68 | ❌ |
| many_footnotes | footnote_ref | 9.58 | 68 | ❌ |
| many_footnotes | footnote_ref | 9.65 | 68 | ❌ |
| many_footnotes | footnote_ref | 9.65 | 68 | ❌ |
| many_footnotes | footnote_ref | 9.67 | 68 | ❌ |
| many_footnotes | footnote_ref | 9.40 | 68 | ❌ |
| recursive_refs | reference_link | 4.17 | 32 | ✅ |
| recursive_refs | reference_link | 4.35 | 32 | ✅ |
| recursive_refs | reference_link | 4.26 | 32 | ✅ |
| recursive_refs | reference_link | 4.28 | 32 | ✅ |
| recursive_refs | reference_link | 4.19 | 32 | ✅ |
| recursive_refs | reference_link | 4.30 | 32 | ✅ |
| recursive_refs | reference_link | 4.26 | 32 | ✅ |
| recursive_refs | reference_link | 4.21 | 32 | ✅ |
| recursive_refs | reference_link | 4.19 | 32 | ✅ |
| recursive_refs | reference_link | 4.19 | 32 | ✅ |
| deeply_nested_quotes | blockquote | 153.76 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 152.78 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 152.30 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 152.08 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 152.30 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 168.19 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 166.22 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 156.34 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 165.94 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 152.13 | 68 | ✅ |
| deeply_nested_lists | list | 40.25 | 135 | ✅ |
| deeply_nested_lists | list | 40.90 | 135 | ✅ |
| deeply_nested_lists | list | 40.60 | 135 | ✅ |
| deeply_nested_lists | list | 40.38 | 135 | ✅ |
| deeply_nested_lists | list | 41.12 | 135 | ✅ |
| deeply_nested_lists | list | 45.54 | 135 | ✅ |
| deeply_nested_lists | list | 40.58 | 135 | ✅ |
| deeply_nested_lists | list | 41.18 | 135 | ✅ |
| deeply_nested_lists | list | 40.53 | 135 | ✅ |
| deeply_nested_lists | list | 40.58 | 135 | ✅ |
| deeply_nested_emphasis | emphasis | 5.39 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.47 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.44 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.46 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.48 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.50 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.48 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.45 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.49 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 5.38 | 52 | ✅ |
| extremely_long_line | text | 386.60 | 1217 | ✅ |
| extremely_long_line | text | 422.67 | 1217 | ✅ |
| extremely_long_line | text | 411.29 | 1217 | ✅ |
| extremely_long_line | text | 426.35 | 1217 | ✅ |
| extremely_long_line | text | 389.32 | 1217 | ✅ |
| extremely_long_line | text | 416.63 | 1217 | ✅ |
| extremely_long_line | text | 401.76 | 1217 | ✅ |
| extremely_long_line | text | 385.55 | 1217 | ✅ |
| extremely_long_line | text | 410.45 | 1217 | ✅ |
| extremely_long_line | text | 415.07 | 1217 | ✅ |
| many_emphasis_markers | text | 137.26 | 77 | ✅ |
| many_emphasis_markers | text | 121.56 | 77 | ✅ |
| many_emphasis_markers | text | 137.25 | 77 | ✅ |
| many_emphasis_markers | text | 122.49 | 77 | ✅ |
| many_emphasis_markers | text | 121.47 | 77 | ✅ |
| many_emphasis_markers | text | 121.38 | 77 | ✅ |
| many_emphasis_markers | text | 121.00 | 77 | ✅ |
| many_emphasis_markers | text | 121.32 | 77 | ✅ |
| many_emphasis_markers | text | 126.82 | 77 | ✅ |
| many_emphasis_markers | text | 121.77 | 77 | ✅ |
| alternating_chars | text | 131.08 | 79 | ✅ |
| alternating_chars | text | 131.30 | 79 | ✅ |
| alternating_chars | text | 131.29 | 79 | ✅ |
| alternating_chars | text | 143.20 | 79 | ✅ |
| alternating_chars | text | 131.60 | 79 | ✅ |
| alternating_chars | text | 131.04 | 79 | ✅ |
| alternating_chars | text | 131.33 | 79 | ✅ |
| alternating_chars | text | 131.19 | 79 | ✅ |
| alternating_chars | text | 131.12 | 79 | ✅ |
| alternating_chars | text | 157.29 | 79 | ✅ |
| quadratic_blowup | text | 371.56 | 198 | ✅ |
| quadratic_blowup | text | 371.56 | 198 | ✅ |
| quadratic_blowup | text | 376.56 | 198 | ✅ |
| quadratic_blowup | text | 396.59 | 198 | ✅ |
| quadratic_blowup | text | 385.31 | 198 | ✅ |
| quadratic_blowup | text | 479.56 | 198 | ✅ |
| quadratic_blowup | text | 374.92 | 198 | ✅ |
| quadratic_blowup | text | 400.20 | 198 | ✅ |
| quadratic_blowup | text | 371.56 | 198 | ✅ |
| quadratic_blowup | text | 384.90 | 198 | ✅ |
| mixed_line_endings_complex | text | 26.37 | 32 | ✅ |
| mixed_line_endings_complex | text | 26.35 | 32 | ✅ |
| mixed_line_endings_complex | text | 26.27 | 32 | ✅ |
| mixed_line_endings_complex | text | 26.29 | 32 | ✅ |
| mixed_line_endings_complex | text | 26.20 | 32 | ✅ |
| mixed_line_endings_complex | text | 26.23 | 32 | ✅ |
| mixed_line_endings_complex | text | 26.26 | 32 | ✅ |
| mixed_line_endings_complex | text | 26.29 | 32 | ✅ |
| mixed_line_endings_complex | text | 26.21 | 32 | ✅ |
| mixed_line_endings_complex | text | 26.31 | 32 | ✅ |
| binary_like_data | text | 48.40 | 103 | ✅ |
| binary_like_data | text | 48.32 | 103 | ✅ |
| binary_like_data | text | 48.33 | 103 | ✅ |
| binary_like_data | text | 48.38 | 103 | ✅ |
| binary_like_data | text | 48.39 | 103 | ✅ |
| binary_like_data | text | 57.87 | 103 | ✅ |
| binary_like_data | text | 72.50 | 103 | ✅ |
| binary_like_data | text | 50.02 | 103 | ✅ |
| binary_like_data | text | 53.61 | 103 | ✅ |
| binary_like_data | text | 48.48 | 103 | ✅ |
| massive_nested_brackets | text | 801.18 | 398 | ✅ |
| massive_nested_brackets | text | 739.12 | 398 | ✅ |
| massive_nested_brackets | text | 758.44 | 398 | ✅ |
| massive_nested_brackets | text | 749.82 | 398 | ✅ |
| massive_nested_brackets | text | 748.45 | 398 | ✅ |
| massive_nested_brackets | text | 741.32 | 398 | ✅ |
| massive_nested_brackets | text | 795.36 | 398 | ✅ |
| massive_nested_brackets | text | 771.17 | 398 | ✅ |
| massive_nested_brackets | text | 775.31 | 398 | ✅ |
| massive_nested_brackets | text | 775.99 | 398 | ✅ |

---
*Report generated by Marco Grammar Test Suite*
