#!/usr/bin/env python3#!/usr/bin/env python3

""""""

Convert TOML backup file to JSON format matching spec0312.json structure.Convert test_cases.toml to JSON format for the new testing system.

- Parse TOML sections and key-value pairsThis script parses the TOML file and creates a JSON file compatible with

- Convert to JSON format with example numbers and sectionsthe existing JSON test specifications.

- Remove any tests with "failure" in the name (expected failures)"""

- Generate proper HTML placeholders

"""import toml

import json

import reimport sys

import jsonfrom pathlib import Path

from typing import Dict, List, Tuple

def convert_toml_to_json(toml_file: str, output_file: str):

def parse_toml_file(file_path: str) -> List[Dict]:    """Convert TOML test cases to JSON format."""

    """Parse TOML backup file and convert to JSON structure."""    

    with open(file_path, 'r', encoding='utf-8') as f:    # Read the TOML file

        content = f.read()    try:

            with open(toml_file, 'r', encoding='utf-8') as f:

    results = []            data = toml.load(f)

    current_section = None    except Exception as e:

    example_number = 1        print(f"Error reading TOML file: {e}")

    start_line = 1        return False

        

    lines = content.split('\n')    # Create the JSON structure

    i = 0    json_tests = []

        example_counter = 1

    while i < len(lines):    

        line = lines[i].strip()    # Process each section in the TOML file

            for section_name, section_tests in data.items():

        # Skip empty lines and comments        if not isinstance(section_tests, dict):

        if not line or line.startswith('#'):            continue

            i += 1            

            continue        print(f"Processing section: {section_name}")

                    

        # Section header        # Convert section name to a readable format

        if line.startswith('[') and line.endswith(']'):        section_display = section_name.replace('_', ' ').title()

            current_section = line[1:-1]  # Remove brackets        

            # Convert section name to human-readable format        # Process each test in the section

            current_section = format_section_name(current_section)        for test_name, test_markdown in section_tests.items():

            i += 1            if not isinstance(test_markdown, str):

            continue                continue

                        

        # Test case line (key = "value")            # Create test description from test name

        if '=' in line and current_section:            description = test_name.replace('_', ' ').title()

            key, value = line.split('=', 1)            

            key = key.strip()            # Determine expected failure based on test name or content

            value = value.strip().strip('"')            expected_failure = (

                            'invalid' in test_name.lower() or 

            # Skip expected failures                'fail' in test_name.lower() or

            if 'failure' in key.lower():                'error' in test_name.lower() or

                i += 1                'should_fail' in test_name.lower()

                continue            )

                        

            # Create test case            # Try to determine the appropriate rule based on section and test content

            test_case = {            rule = determine_rule(section_name, test_name, test_markdown)

                "markdown": value,            

                "html": generate_html_placeholder(value, key),            # Create the test entry

                "example": example_number,            test_entry = {

                "start_line": start_line,                "example": example_counter,

                "end_line": start_line + value.count('\n') + 5,  # Approximate                "section": section_display,

                "section": current_section                "description": description,

            }                "markdown": test_markdown,

                            "html": "",  # Empty HTML for custom tests since we're testing parsing only

            results.append(test_case)                "rule": rule,

            example_number += 1                "expected_failure": expected_failure

            start_line += 10  # Approximate line spacing            }

                    

        i += 1            json_tests.append(test_entry)

                example_counter += 1

    return results    

    # Write the JSON file

def format_section_name(section: str) -> str:    try:

    """Convert snake_case section names to Title Case."""        with open(output_file, 'w', encoding='utf-8') as f:

    # Handle common patterns            json.dump(json_tests, f, indent=2, ensure_ascii=False)

    replacements = {        

        'text_and_words': 'Text And Words',        print(f"Successfully converted {len(json_tests)} tests to {output_file}")

        'headings_atx': 'Headings ATX',        return True

        'headings_setext': 'Headings Setext',        

        'bold_formatting': 'Bold Formatting',    except Exception as e:

        'italic_formatting': 'Italic Formatting',        print(f"Error writing JSON file: {e}")

        'bold_italic_combinations': 'Bold Italic Combinations',        return False

        'other_formatting': 'Other Formatting',

        'code_inline': 'Code Inline',def determine_rule(section_name: str, test_name: str, markdown: str) -> str:

        'math_inline': 'Math Inline',    """Determine the appropriate grammar rule based on section and content."""

        'code_blocks': 'Code Blocks',    

        'math_blocks': 'Math Blocks',    # Rule mapping based on section names and patterns

        'urls': 'URLs',    rule_mappings = {

        'inline_links': 'Inline Links',        'text_and_words': 'text',

        'link_title': 'Link Title',        'headings_atx': 'heading_atx',

        'inline_images': 'Inline Images',        'headings_setext': 'heading_setext', 

        'reference_links': 'Reference Links',        'bold_formatting': 'bold',

        'unordered_lists': 'Unordered Lists',        'italic_formatting': 'italic',

        'ordered_lists': 'Ordered Lists',        'bold_italic_combinations': 'emphasis',

        'task_lists': 'Task Lists',        'other_formatting': 'emphasis',

        'definition_lists': 'Definition Lists',        'code_inline': 'code_inline',

        'tables': 'Tables',        'math_inline': 'math_inline',

        'blockquotes': 'Blockquotes',        'code_blocks': 'code_block',

        'horizontal_rules': 'Horizontal Rules',        'math_blocks': 'math_block',

        'footnotes': 'Footnotes',        'urls': 'autolink',

        'html_elements': 'HTML Elements',        'inline_links': 'inline_link',

        'user_mentions': 'User Mentions',        'link_title': 'inline_link',

        'admonitions': 'Admonitions',        'inline_images': 'inline_image',

        'tab': 'Tabs',        'reference_links': 'reference_link',

        'page_and_doc': 'Page And Doc',        'unordered_lists': 'list',

        'bookmarks': 'Bookmarks',        'ordered_lists': 'list',

        'run_commands': 'Run Commands',        'task_lists': 'list',

        'diagrams': 'Diagrams',        'definition_lists': 'definition_list',

        'escaped_characters': 'Escaped Characters',        'tables': 'table',

        'edge_cases': 'Edge Cases'        'blockquotes': 'blockquote',

    }        'horizontal_rules': 'hr',

            'footnotes': 'footnote',

    return replacements.get(section, section.replace('_', ' ').title())        'admonitions': 'admonition_block',

        'tabs': 'tabs_block',

def generate_html_placeholder(markdown: str, test_name: str) -> str:        'user_mentions': 'user_mention',

    """Generate a simple HTML placeholder based on markdown content."""        'bookmarks': 'bookmark',

    # Simple conversion patterns        'page_tags': 'page_tag',

    if markdown.startswith('#'):        'toc': 'toc',

        level = len(markdown) - len(markdown.lstrip('#'))        'run_commands': 'run_command',

        content = markdown.lstrip('#').strip()        'html_blocks': 'html_block',

        return f"<h{level}>{content}</h{level}>\n"        'raw_html': 'html_inline',

    elif markdown.startswith('**') and markdown.endswith('**'):        'line_breaks': 'hard_line_break',

        content = markdown[2:-2]        'comments': 'comment',

        return f"<p><strong>{content}</strong></p>\n"        'directives': 'directive',

    elif markdown.startswith('*') and markdown.endswith('*') and not markdown.startswith('**'):        'citations': 'citation',

        content = markdown[1:-1]        'abbreviations': 'abbreviation'

        return f"<p><em>{content}</em></p>\n"    }

    elif markdown.startswith('`') and markdown.endswith('`'):    

        content = markdown[1:-1]    # Try to match section name first

        return f"<p><code>{content}</code></p>\n"    if section_name in rule_mappings:

    elif markdown.startswith('```'):        return rule_mappings[section_name]

        return f"<pre><code>{markdown[3:-3].strip()}</code></pre>\n"    

    elif markdown.startswith('[') and '](' in markdown:    # Try to match based on content patterns

        return f"<p><a href=\"#\">{markdown}</a></p>\n"    markdown_lower = markdown.lower()

    elif markdown.startswith('!['):    

        return f"<p><img src=\"#\" alt=\"{markdown}\" /></p>\n"    if markdown.startswith('#'):

    elif markdown.startswith('>'):        return 'heading_atx'

        content = markdown.lstrip('>').strip()    elif '**' in markdown or '__' in markdown:

        return f"<blockquote><p>{content}</p></blockquote>\n"        return 'bold'

    elif markdown.startswith('- ') or markdown.startswith('* ') or markdown.startswith('+ '):    elif '*' in markdown or '_' in markdown:

        content = markdown[2:]        return 'italic'

        return f"<ul><li>{content}</li></ul>\n"    elif '`' in markdown and not markdown.startswith('```'):

    elif re.match(r'^\d+\. ', markdown):        return 'code_inline'

        content = re.sub(r'^\d+\. ', '', markdown)    elif markdown.startswith('```'):

        return f"<ol><li>{content}</li></ol>\n"        return 'code_block'

    else:    elif markdown.startswith('$$') or (markdown.startswith('$') and markdown.endswith('$')):

        # Default to paragraph        return 'math_inline' if not markdown.startswith('$$') else 'math_block'

        return f"<p>{markdown}</p>\n"    elif markdown.startswith('http://') or markdown.startswith('https://'):

        return 'autolink'

def main():    elif '[' in markdown and '](' in markdown:

    """Main conversion function."""        return 'inline_link' if '![' not in markdown else 'inline_image'

    input_file = 'testcasestoml.backup'    elif markdown.startswith('- ') or markdown.startswith('+ ') or markdown.startswith('* '):

    output_file = 'spec_converted_from_toml.json'        return 'list'

        elif markdown.startswith('1.') or markdown.startswith('2.'):

    print(f"Converting {input_file} to JSON format...")        return 'list'

        elif markdown.startswith('> '):

    test_cases = parse_toml_file(input_file)        return 'blockquote'

        elif markdown.startswith('---') or markdown.startswith('***'):

    print(f"Converted {len(test_cases)} test cases")        return 'hr'

    print("Removed all tests with 'failure' in the name (expected failures)")    elif '|' in markdown and markdown.count('|') >= 2:

            return 'table'

    # Write JSON file    elif markdown.startswith(':::'):

    with open(output_file, 'w', encoding='utf-8') as f:        return 'admonition_block'

        json.dump(test_cases, f, indent=2, ensure_ascii=False)    elif '@' in markdown:

            return 'user_mention'

    print(f"Output written to {output_file}")    

    print(f"Total test cases: {len(test_cases)}")    # Default to document for comprehensive parsing

    return 'document'

if __name__ == '__main__':

    main()def main():
    """Main conversion function."""
    toml_file = "test_cases.toml"
    output_file = "spec_custom.json"
    
    if not Path(toml_file).exists():
        print(f"Error: {toml_file} not found!")
        return 1
    
    print(f"Converting {toml_file} to {output_file}...")
    
    if convert_toml_to_json(toml_file, output_file):
        print("Conversion completed successfully!")
        return 0
    else:
        print("Conversion failed!")
        return 1

if __name__ == "__main__":
    sys.exit(main())