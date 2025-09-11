# Marco Grammar Performance Benchmark Report

Generated automatically from benchmark tests

## Summary

- **Total Tests**: 340
- **Passed**: 310 ✅
- **Failed**: 30 ❌
- **Total Time**: 134.77ms
- **Average Parse Time**: 303.64μs
- **Memory Estimate**: 57560 bytes
- **Slowest Test**: massive_nested_brackets (2548.42μs)
- **Fastest Test**: perf_complex_formatting (3.71μs)

## Performance Analysis

### Extra Large (1000+ chars)
- Tests: 10
- Average Time: 858.60μs
- Average Size: 1217 chars
- Throughput: 1.42 MB/s

### Large (201-1000 chars)
- Tests: 80
- Average Time: 713.36μs
- Average Size: 294 chars
- Throughput: 0.41 MB/s

### Medium (51-200 chars)
- Tests: 200
- Average Time: 180.32μs
- Average Size: 100 chars
- Throughput: 0.55 MB/s

### Small (0-50 chars)
- Tests: 50
- Average Time: 30.36μs
- Average Size: 33 chars
- Throughput: 1.09 MB/s

## Detailed Results

| Test Name | Rule | Time (μs) | Input Size | Status |
|-----------|------|-----------|------------|--------|
| perf_simple_parse | text | 22.98 | 30 | ✅ |
| perf_simple_parse | text | 19.95 | 30 | ✅ |
| perf_simple_parse | text | 19.76 | 30 | ✅ |
| perf_simple_parse | text | 19.74 | 30 | ✅ |
| perf_simple_parse | text | 19.76 | 30 | ✅ |
| perf_simple_parse | text | 20.67 | 30 | ✅ |
| perf_simple_parse | text | 19.99 | 30 | ✅ |
| perf_simple_parse | text | 19.91 | 30 | ✅ |
| perf_simple_parse | text | 19.88 | 30 | ✅ |
| perf_simple_parse | text | 19.93 | 30 | ✅ |
| perf_complex_formatting | emphasis | 3.75 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.75 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.72 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.72 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.71 | 65 | ✅ |
| perf_complex_formatting | emphasis | 4.08 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.77 | 65 | ✅ |
| perf_complex_formatting | emphasis | 4.07 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.89 | 65 | ✅ |
| perf_complex_formatting | emphasis | 3.80 | 65 | ✅ |
| perf_nested_structures | text | 16.90 | 52 | ✅ |
| perf_nested_structures | text | 17.09 | 52 | ✅ |
| perf_nested_structures | text | 16.95 | 52 | ✅ |
| perf_nested_structures | text | 16.87 | 52 | ✅ |
| perf_nested_structures | text | 16.80 | 52 | ✅ |
| perf_nested_structures | text | 16.84 | 52 | ✅ |
| perf_nested_structures | text | 16.94 | 52 | ✅ |
| perf_nested_structures | text | 16.89 | 52 | ✅ |
| perf_nested_structures | text | 16.91 | 52 | ✅ |
| perf_nested_structures | text | 16.98 | 52 | ✅ |
| perf_large_paragraph | text | 121.10 | 279 | ✅ |
| perf_large_paragraph | text | 121.26 | 279 | ✅ |
| perf_large_paragraph | text | 121.35 | 279 | ✅ |
| perf_large_paragraph | text | 122.49 | 279 | ✅ |
| perf_large_paragraph | text | 121.43 | 279 | ✅ |
| perf_large_paragraph | text | 128.01 | 279 | ✅ |
| perf_large_paragraph | text | 121.91 | 279 | ✅ |
| perf_large_paragraph | text | 121.39 | 279 | ✅ |
| perf_large_paragraph | text | 121.59 | 279 | ✅ |
| perf_large_paragraph | text | 121.55 | 279 | ✅ |
| perf_github_readme | document | 734.29 | 203 | ✅ |
| perf_github_readme | document | 733.44 | 203 | ✅ |
| perf_github_readme | document | 737.02 | 203 | ✅ |
| perf_github_readme | document | 726.83 | 203 | ✅ |
| perf_github_readme | document | 733.95 | 203 | ✅ |
| perf_github_readme | document | 743.32 | 203 | ✅ |
| perf_github_readme | document | 734.97 | 203 | ✅ |
| perf_github_readme | document | 726.19 | 203 | ✅ |
| perf_github_readme | document | 732.40 | 203 | ✅ |
| perf_github_readme | document | 735.03 | 203 | ✅ |
| perf_academic_paper | document | 1143.50 | 249 | ✅ |
| perf_academic_paper | document | 1137.09 | 249 | ✅ |
| perf_academic_paper | document | 1164.56 | 249 | ✅ |
| perf_academic_paper | document | 1144.64 | 249 | ✅ |
| perf_academic_paper | document | 1151.02 | 249 | ✅ |
| perf_academic_paper | document | 1151.94 | 249 | ✅ |
| perf_academic_paper | document | 1156.07 | 249 | ✅ |
| perf_academic_paper | document | 1140.47 | 249 | ✅ |
| perf_academic_paper | document | 1159.23 | 249 | ✅ |
| perf_academic_paper | document | 1149.96 | 249 | ✅ |
| perf_many_small_elements | text | 71.33 | 80 | ✅ |
| perf_many_small_elements | text | 71.57 | 80 | ✅ |
| perf_many_small_elements | text | 71.44 | 80 | ✅ |
| perf_many_small_elements | text | 71.48 | 80 | ✅ |
| perf_many_small_elements | text | 71.28 | 80 | ✅ |
| perf_many_small_elements | text | 72.66 | 80 | ✅ |
| perf_many_small_elements | text | 71.36 | 80 | ✅ |
| perf_many_small_elements | text | 79.68 | 80 | ✅ |
| perf_many_small_elements | text | 72.67 | 80 | ✅ |
| perf_many_small_elements | text | 71.30 | 80 | ✅ |
| perf_few_large_elements | text | 84.14 | 184 | ✅ |
| perf_few_large_elements | text | 84.02 | 184 | ✅ |
| perf_few_large_elements | text | 83.89 | 184 | ✅ |
| perf_few_large_elements | text | 83.80 | 184 | ✅ |
| perf_few_large_elements | text | 83.82 | 184 | ✅ |
| perf_few_large_elements | text | 83.97 | 184 | ✅ |
| perf_few_large_elements | text | 101.02 | 184 | ✅ |
| perf_few_large_elements | text | 85.05 | 184 | ✅ |
| perf_few_large_elements | text | 83.75 | 184 | ✅ |
| perf_few_large_elements | text | 83.97 | 184 | ✅ |
| perf_shallow_wide | text | 86.88 | 79 | ✅ |
| perf_shallow_wide | text | 87.02 | 79 | ✅ |
| perf_shallow_wide | text | 86.86 | 79 | ✅ |
| perf_shallow_wide | text | 86.73 | 79 | ✅ |
| perf_shallow_wide | text | 103.11 | 79 | ✅ |
| perf_shallow_wide | text | 88.95 | 79 | ✅ |
| perf_shallow_wide | text | 86.67 | 79 | ✅ |
| perf_shallow_wide | text | 86.64 | 79 | ✅ |
| perf_shallow_wide | text | 86.91 | 79 | ✅ |
| perf_shallow_wide | text | 86.97 | 79 | ✅ |
| perf_deep_narrow | text | 45.15 | 36 | ✅ |
| perf_deep_narrow | text | 45.22 | 36 | ✅ |
| perf_deep_narrow | text | 45.15 | 36 | ✅ |
| perf_deep_narrow | text | 45.24 | 36 | ✅ |
| perf_deep_narrow | text | 45.01 | 36 | ✅ |
| perf_deep_narrow | text | 45.17 | 36 | ✅ |
| perf_deep_narrow | text | 45.22 | 36 | ✅ |
| perf_deep_narrow | text | 45.22 | 36 | ✅ |
| perf_deep_narrow | text | 61.67 | 36 | ✅ |
| perf_deep_narrow | text | 47.40 | 36 | ✅ |
| huge_document_headings | text | 14.45 | 223 | ✅ |
| huge_document_headings | text | 14.63 | 223 | ✅ |
| huge_document_headings | text | 14.52 | 223 | ✅ |
| huge_document_headings | text | 14.57 | 223 | ✅ |
| huge_document_headings | text | 14.62 | 223 | ✅ |
| huge_document_headings | text | 14.74 | 223 | ✅ |
| huge_document_headings | text | 14.63 | 223 | ✅ |
| huge_document_headings | text | 14.58 | 223 | ✅ |
| huge_document_headings | text | 14.57 | 223 | ✅ |
| huge_document_headings | text | 14.62 | 223 | ✅ |
| many_bold_words | text | 223.58 | 210 | ✅ |
| many_bold_words | text | 210.13 | 210 | ✅ |
| many_bold_words | text | 207.22 | 210 | ✅ |
| many_bold_words | text | 207.44 | 210 | ✅ |
| many_bold_words | text | 207.31 | 210 | ✅ |
| many_bold_words | text | 224.21 | 210 | ✅ |
| many_bold_words | text | 209.20 | 210 | ✅ |
| many_bold_words | text | 207.18 | 210 | ✅ |
| many_bold_words | text | 207.10 | 210 | ✅ |
| many_bold_words | text | 222.80 | 210 | ✅ |
| many_links | text | 187.69 | 141 | ✅ |
| many_links | text | 203.57 | 141 | ✅ |
| many_links | text | 189.01 | 141 | ✅ |
| many_links | text | 188.07 | 141 | ✅ |
| many_links | text | 187.63 | 141 | ✅ |
| many_links | text | 187.74 | 141 | ✅ |
| many_links | text | 187.76 | 141 | ✅ |
| many_links | text | 201.30 | 141 | ✅ |
| many_links | text | 188.12 | 141 | ✅ |
| many_links | text | 188.66 | 141 | ✅ |
| many_footnotes_refs | footnote_ref | 20.04 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 20.17 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 20.68 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 20.17 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 20.26 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 20.59 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 20.76 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 20.17 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 20.06 | 156 | ❌ |
| many_footnotes_refs | footnote_ref | 20.20 | 156 | ❌ |
| huge_nested_list | list | 113.05 | 422 | ✅ |
| huge_nested_list | list | 112.88 | 422 | ✅ |
| huge_nested_list | list | 113.48 | 422 | ✅ |
| huge_nested_list | list | 113.82 | 422 | ✅ |
| huge_nested_list | list | 113.39 | 422 | ✅ |
| huge_nested_list | list | 113.12 | 422 | ✅ |
| huge_nested_list | list | 128.50 | 422 | ✅ |
| huge_nested_list | list | 113.50 | 422 | ✅ |
| huge_nested_list | list | 112.72 | 422 | ✅ |
| huge_nested_list | list | 112.95 | 422 | ✅ |
| exponential_nesting | text | 100.73 | 121 | ✅ |
| exponential_nesting | text | 100.88 | 121 | ✅ |
| exponential_nesting | text | 117.07 | 121 | ✅ |
| exponential_nesting | text | 103.18 | 121 | ✅ |
| exponential_nesting | text | 100.84 | 121 | ✅ |
| exponential_nesting | text | 100.79 | 121 | ✅ |
| exponential_nesting | text | 100.70 | 121 | ✅ |
| exponential_nesting | text | 100.90 | 121 | ✅ |
| exponential_nesting | text | 100.81 | 121 | ✅ |
| exponential_nesting | text | 100.90 | 121 | ✅ |
| parse_tree_explosion | text | 82.67 | 38 | ✅ |
| parse_tree_explosion | text | 67.30 | 38 | ✅ |
| parse_tree_explosion | text | 65.88 | 38 | ✅ |
| parse_tree_explosion | text | 65.67 | 38 | ✅ |
| parse_tree_explosion | text | 65.81 | 38 | ✅ |
| parse_tree_explosion | text | 66.08 | 38 | ✅ |
| parse_tree_explosion | text | 66.71 | 38 | ✅ |
| parse_tree_explosion | text | 65.83 | 38 | ✅ |
| parse_tree_explosion | text | 65.86 | 38 | ✅ |
| parse_tree_explosion | text | 65.92 | 38 | ✅ |
| large_table_data | table | 1668.12 | 370 | ✅ |
| large_table_data | table | 1603.33 | 370 | ✅ |
| large_table_data | table | 1622.96 | 370 | ✅ |
| large_table_data | table | 1621.69 | 370 | ✅ |
| large_table_data | table | 1603.55 | 370 | ✅ |
| large_table_data | table | 1614.17 | 370 | ✅ |
| large_table_data | table | 1592.36 | 370 | ✅ |
| large_table_data | table | 1615.79 | 370 | ✅ |
| large_table_data | table | 1614.50 | 370 | ✅ |
| large_table_data | table | 1593.77 | 370 | ✅ |
| backtrack_emphasis | text | 94.45 | 65 | ✅ |
| backtrack_emphasis | text | 92.44 | 65 | ✅ |
| backtrack_emphasis | text | 91.87 | 65 | ✅ |
| backtrack_emphasis | text | 91.97 | 65 | ✅ |
| backtrack_emphasis | text | 91.92 | 65 | ✅ |
| backtrack_emphasis | text | 91.96 | 65 | ✅ |
| backtrack_emphasis | text | 91.92 | 65 | ✅ |
| backtrack_emphasis | text | 91.94 | 65 | ✅ |
| backtrack_emphasis | text | 92.01 | 65 | ✅ |
| backtrack_emphasis | text | 92.30 | 65 | ✅ |
| backtrack_links | text | 249.70 | 74 | ✅ |
| backtrack_links | text | 262.87 | 74 | ✅ |
| backtrack_links | text | 249.76 | 74 | ✅ |
| backtrack_links | text | 249.03 | 74 | ✅ |
| backtrack_links | text | 259.12 | 74 | ✅ |
| backtrack_links | text | 254.66 | 74 | ✅ |
| backtrack_links | text | 249.57 | 74 | ✅ |
| backtrack_links | text | 249.06 | 74 | ✅ |
| backtrack_links | text | 266.92 | 74 | ✅ |
| backtrack_links | text | 250.44 | 74 | ✅ |
| backtrack_code | text | 12.34 | 51 | ✅ |
| backtrack_code | text | 12.46 | 51 | ✅ |
| backtrack_code | text | 12.34 | 51 | ✅ |
| backtrack_code | text | 12.42 | 51 | ✅ |
| backtrack_code | text | 12.43 | 51 | ✅ |
| backtrack_code | text | 12.35 | 51 | ✅ |
| backtrack_code | text | 12.31 | 51 | ✅ |
| backtrack_code | text | 12.38 | 51 | ✅ |
| backtrack_code | text | 12.37 | 51 | ✅ |
| backtrack_code | text | 12.33 | 51 | ✅ |
| large_table | table | 1267.31 | 169 | ✅ |
| large_table | table | 1271.87 | 169 | ✅ |
| large_table | table | 1279.29 | 169 | ✅ |
| large_table | table | 1258.06 | 169 | ✅ |
| large_table | table | 1250.61 | 169 | ✅ |
| large_table | table | 1253.35 | 169 | ✅ |
| large_table | table | 1274.77 | 169 | ✅ |
| large_table | table | 1267.01 | 169 | ✅ |
| large_table | table | 1258.74 | 169 | ✅ |
| large_table | table | 1263.40 | 169 | ✅ |
| many_footnotes | footnote_ref | 15.09 | 68 | ❌ |
| many_footnotes | footnote_ref | 9.72 | 68 | ❌ |
| many_footnotes | footnote_ref | 9.36 | 68 | ❌ |
| many_footnotes | footnote_ref | 9.13 | 68 | ❌ |
| many_footnotes | footnote_ref | 9.32 | 68 | ❌ |
| many_footnotes | footnote_ref | 8.88 | 68 | ❌ |
| many_footnotes | footnote_ref | 9.59 | 68 | ❌ |
| many_footnotes | footnote_ref | 8.98 | 68 | ❌ |
| many_footnotes | footnote_ref | 8.76 | 68 | ❌ |
| many_footnotes | footnote_ref | 9.36 | 68 | ❌ |
| recursive_refs | reference_link | 4.50 | 32 | ✅ |
| recursive_refs | reference_link | 4.56 | 32 | ✅ |
| recursive_refs | reference_link | 4.50 | 32 | ✅ |
| recursive_refs | reference_link | 4.42 | 32 | ✅ |
| recursive_refs | reference_link | 4.43 | 32 | ✅ |
| recursive_refs | reference_link | 4.50 | 32 | ✅ |
| recursive_refs | reference_link | 4.48 | 32 | ✅ |
| recursive_refs | reference_link | 4.41 | 32 | ✅ |
| recursive_refs | reference_link | 4.41 | 32 | ✅ |
| recursive_refs | reference_link | 4.42 | 32 | ✅ |
| deeply_nested_quotes | blockquote | 207.71 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 202.39 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 200.40 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 200.02 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 204.30 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 213.78 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 213.73 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 314.96 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 297.25 | 68 | ✅ |
| deeply_nested_quotes | blockquote | 315.49 | 68 | ✅ |
| deeply_nested_lists | list | 69.30 | 135 | ✅ |
| deeply_nested_lists | list | 68.93 | 135 | ✅ |
| deeply_nested_lists | list | 69.41 | 135 | ✅ |
| deeply_nested_lists | list | 69.05 | 135 | ✅ |
| deeply_nested_lists | list | 69.25 | 135 | ✅ |
| deeply_nested_lists | list | 69.12 | 135 | ✅ |
| deeply_nested_lists | list | 68.98 | 135 | ✅ |
| deeply_nested_lists | list | 89.14 | 135 | ✅ |
| deeply_nested_lists | list | 69.43 | 135 | ✅ |
| deeply_nested_lists | list | 69.46 | 135 | ✅ |
| deeply_nested_emphasis | emphasis | 9.38 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 9.35 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 9.49 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 9.71 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 9.29 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 9.60 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 9.76 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 9.47 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 9.43 | 52 | ✅ |
| deeply_nested_emphasis | emphasis | 10.08 | 52 | ✅ |
| extremely_long_line | text | 955.79 | 1217 | ✅ |
| extremely_long_line | text | 962.92 | 1217 | ✅ |
| extremely_long_line | text | 968.33 | 1217 | ✅ |
| extremely_long_line | text | 968.94 | 1217 | ✅ |
| extremely_long_line | text | 965.35 | 1217 | ✅ |
| extremely_long_line | text | 845.76 | 1217 | ✅ |
| extremely_long_line | text | 771.71 | 1217 | ✅ |
| extremely_long_line | text | 956.84 | 1217 | ✅ |
| extremely_long_line | text | 645.93 | 1217 | ✅ |
| extremely_long_line | text | 544.41 | 1217 | ✅ |
| many_emphasis_markers | text | 152.96 | 77 | ✅ |
| many_emphasis_markers | text | 153.36 | 77 | ✅ |
| many_emphasis_markers | text | 152.82 | 77 | ✅ |
| many_emphasis_markers | text | 152.87 | 77 | ✅ |
| many_emphasis_markers | text | 158.31 | 77 | ✅ |
| many_emphasis_markers | text | 154.55 | 77 | ✅ |
| many_emphasis_markers | text | 153.61 | 77 | ✅ |
| many_emphasis_markers | text | 153.22 | 77 | ✅ |
| many_emphasis_markers | text | 152.41 | 77 | ✅ |
| many_emphasis_markers | text | 152.91 | 77 | ✅ |
| alternating_chars | text | 160.19 | 79 | ✅ |
| alternating_chars | text | 160.19 | 79 | ✅ |
| alternating_chars | text | 160.07 | 79 | ✅ |
| alternating_chars | text | 165.44 | 79 | ✅ |
| alternating_chars | text | 160.45 | 79 | ✅ |
| alternating_chars | text | 159.87 | 79 | ✅ |
| alternating_chars | text | 159.79 | 79 | ✅ |
| alternating_chars | text | 160.16 | 79 | ✅ |
| alternating_chars | text | 159.61 | 79 | ✅ |
| alternating_chars | text | 160.22 | 79 | ✅ |
| quadratic_blowup | text | 733.42 | 198 | ✅ |
| quadratic_blowup | text | 738.59 | 198 | ✅ |
| quadratic_blowup | text | 737.81 | 198 | ✅ |
| quadratic_blowup | text | 737.85 | 198 | ✅ |
| quadratic_blowup | text | 733.22 | 198 | ✅ |
| quadratic_blowup | text | 737.90 | 198 | ✅ |
| quadratic_blowup | text | 737.83 | 198 | ✅ |
| quadratic_blowup | text | 737.52 | 198 | ✅ |
| quadratic_blowup | text | 734.87 | 198 | ✅ |
| quadratic_blowup | text | 738.73 | 198 | ✅ |
| mixed_line_endings_complex | text | 12.11 | 32 | ✅ |
| mixed_line_endings_complex | text | 12.27 | 32 | ✅ |
| mixed_line_endings_complex | text | 12.36 | 32 | ✅ |
| mixed_line_endings_complex | text | 12.24 | 32 | ✅ |
| mixed_line_endings_complex | text | 12.24 | 32 | ✅ |
| mixed_line_endings_complex | text | 12.32 | 32 | ✅ |
| mixed_line_endings_complex | text | 12.31 | 32 | ✅ |
| mixed_line_endings_complex | text | 12.35 | 32 | ✅ |
| mixed_line_endings_complex | text | 12.29 | 32 | ✅ |
| mixed_line_endings_complex | text | 12.21 | 32 | ✅ |
| binary_like_data | text | 21.82 | 103 | ❌ |
| binary_like_data | text | 22.79 | 103 | ❌ |
| binary_like_data | text | 21.78 | 103 | ❌ |
| binary_like_data | text | 22.55 | 103 | ❌ |
| binary_like_data | text | 22.38 | 103 | ❌ |
| binary_like_data | text | 22.94 | 103 | ❌ |
| binary_like_data | text | 22.23 | 103 | ❌ |
| binary_like_data | text | 28.55 | 103 | ❌ |
| binary_like_data | text | 22.97 | 103 | ❌ |
| binary_like_data | text | 22.43 | 103 | ❌ |
| massive_nested_brackets | text | 1500.74 | 398 | ✅ |
| massive_nested_brackets | text | 1484.86 | 398 | ✅ |
| massive_nested_brackets | text | 1483.26 | 398 | ✅ |
| massive_nested_brackets | text | 1489.07 | 398 | ✅ |
| massive_nested_brackets | text | 1484.25 | 398 | ✅ |
| massive_nested_brackets | text | 1486.19 | 398 | ✅ |
| massive_nested_brackets | text | 1484.77 | 398 | ✅ |
| massive_nested_brackets | text | 1941.91 | 398 | ✅ |
| massive_nested_brackets | text | 2537.39 | 398 | ✅ |
| massive_nested_brackets | text | 2548.42 | 398 | ✅ |

---
*Report generated by Marco Grammar Test Suite*
