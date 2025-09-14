#!/usr/bin/env python3
"""
Convert TOML backup file to JSON format matching spec0312.json structure.
"""

import re
import json
from typing import Dict, List

def parse_toml_file(file_path: str) -> List[Dict]:
    """Parse TOML backup file and convert to JSON structure."""
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    results = []
    current_section = None
    example_number = 1
    start_line = 1
    
    lines = content.split('\n')
    i = 0
    
    while i < len(lines):
        line = lines[i].strip()
        
        # Skip empty lines and comments
        if not line or line.startswith('#'):
            i += 1
            continue
            
        # Section header
        if line.startswith('[') and line.endswith(']'):
            current_section = line[1:-1]  # Remove brackets
            # Convert section name to human-readable format
            current_section = format_section_name(current_section)
            i += 1
            continue
        
        # Test case line (key = "value")
        if '=' in line and current_section:
            key, value = line.split('=', 1)
            key = key.strip()
            value = value.strip().strip('"')
            
            # Skip expected failures
            if 'failure' in key.lower():
                i += 1
                continue
            
            # Create test case
            test_case = {
                "markdown": value,
                "html": generate_html_placeholder(value, key),
                "example": example_number,
                "start_line": start_line,
                "end_line": start_line + value.count('\n') + 5,  # Approximate
                "section": current_section
            }
            
            results.append(test_case)
            example_number += 1
            start_line += 10  # Approximate line spacing
        
        i += 1
    
    return results

def format_section_name(section: str) -> str:
    """Convert snake_case section names to Title Case."""
    replacements = {
        'text_and_words': 'Text And Words',
        'headings_atx': 'ATX headings',
        'headings_setext': 'Setext headings', 
        'bold_formatting': 'Bold Formatting',
        'italic_formatting': 'Italic Formatting',
        'bold_italic_combinations': 'Bold Italic Combinations',
        'other_formatting': 'Other Formatting',
        'code_inline': 'Code spans',
        'math_inline': 'Math Inline',
        'code_blocks': 'Fenced code blocks',
        'math_blocks': 'Math Blocks',
        'urls': 'Autolinks',
        'inline_links': 'Links',
        'link_title': 'Link Title',
        'inline_images': 'Images',
        'reference_links': 'Link reference definitions',
        'unordered_lists': 'Lists',
        'ordered_lists': 'Lists',
        'task_lists': 'Task Lists',
        'definition_lists': 'Definition Lists',
        'tables': 'Tables',
        'blockquotes': 'Block quotes',
        'horizontal_rules': 'Horizontal rules',
        'footnotes': 'Footnotes',
        'html_elements': 'Raw HTML',
        'user_mentions': 'User Mentions',
        'admonitions': 'Admonitions',
        'tab': 'Tabs',
        'page_and_doc': 'Page And Doc',
        'bookmarks': 'Bookmarks',
        'run_commands': 'Run Commands',
        'diagrams': 'Diagrams',
        'escaped_characters': 'Backslash escapes',
        'edge_cases': 'Edge Cases'
    }
    
    return replacements.get(section, section.replace('_', ' ').title())

def generate_html_placeholder(markdown: str, test_name: str) -> str:
    """Generate a simple HTML placeholder based on markdown content."""
    # Escape HTML entities
    markdown_escaped = markdown.replace('&', '&amp;').replace('<', '&lt;').replace('>', '&gt;')
    
    # Simple conversion patterns
    if markdown.startswith('#'):
        level = len(markdown) - len(markdown.lstrip('#'))
        level = min(level, 6)  # Max h6
        content = markdown.lstrip('#').strip()
        if not content:
            content = ""
        return f"<h{level}>{content}</h{level}>"
    elif markdown.startswith('**') and markdown.endswith('**') and len(markdown) > 4:
        content = markdown[2:-2]
        return f"<p><strong>{content}</strong></p>"
    elif markdown.startswith('*') and markdown.endswith('*') and not markdown.startswith('**') and len(markdown) > 2:
        content = markdown[1:-1]
        return f"<p><em>{content}</em></p>"
    elif markdown.startswith('`') and markdown.endswith('`') and len(markdown) > 2:
        content = markdown[1:-1]
        return f"<p><code>{content}</code></p>"
    elif markdown.startswith('```') and markdown.endswith('```'):
        content = markdown[3:-3].strip()
        return f"<pre><code>{content}\\n</code></pre>\\n"
    elif '](http' in markdown and markdown.startswith('['):
        return f"<p>{markdown_escaped}</p>"
    elif markdown.startswith('!['):
        return f"<p>{markdown_escaped}</p>"
    elif markdown.startswith('>'):
        content = markdown.lstrip('>').strip()
        return f"<blockquote>\\n<p>{content}</p>\\n</blockquote>"
    elif markdown.startswith(('- ', '* ', '+ ')):
        content = markdown[2:]
        return f"<ul>\\n<li>{content}</li>\\n</ul>"
    elif re.match(r'^\d+\. ', markdown):
        content = re.sub(r'^\d+\. ', '', markdown)
        return f"<ol>\\n<li>{content}</li>\\n</ol>"
    elif markdown.startswith('$') and markdown.endswith('$') and len(markdown) > 2:
        return f"<p>{markdown_escaped}</p>"
    elif markdown.startswith('$$') and markdown.endswith('$$'):
        return f"<p>{markdown_escaped}</p>" 
    else:
        # Default to paragraph
        if not markdown.strip():
            return ""
        return f"<p>{markdown_escaped}</p>"

def main():
    """Main conversion function."""
    input_file = 'testcasestoml.backup'
    output_file = 'spec_converted_from_toml.json'
    
    print(f"Converting {input_file} to JSON format...")
    
    test_cases = parse_toml_file(input_file)
    
    print(f"Converted {len(test_cases)} test cases")
    print("Removed all tests with 'failure' in the name (expected failures)")
    
    # Write JSON file
    with open(output_file, 'w', encoding='utf-8') as f:
        json.dump(test_cases, f, indent=2, ensure_ascii=False)
    
    print(f"Output written to {output_file}")
    print(f"Total test cases: {len(test_cases)}")

if __name__ == '__main__':
    main()