#!/usr/bin/env python3
"""
AST Syntax Checker and Display Hints Generator

A CLI tool for validating AST files against syntax definitions and generating
display hints based on the logical framework described in the documentation.
"""

import os
import sys
import re
from pathlib import Path
from typing import List, Dict, Any, Optional, Tuple


def clear_screen():
    """Clear the terminal screen."""
    os.system('clear' if os.name == 'posix' else 'cls')


class ASTSyntaxChecker:
    """Main class for AST validation and display hints generation."""
    
    def __init__(self, base_path: str):
        """
        Initialize the AST Syntax Checker.
        
        Args:
            base_path (str): The base path to the markdown_schema directory
        """
        self.base_path = Path(base_path)
        self.schema_folders = self._discover_schema_folders()
    
    def _discover_schema_folders(self) -> List[str]:
        """
        Discover all available schema folders in the markdown_schema directory.
        
        Returns:
            List[str]: List of folder names containing schema definitions
        """
        folders = []
        if self.base_path.exists() and self.base_path.is_dir():
            for item in self.base_path.iterdir():
                if item.is_dir():
                    # Check if it contains the required files
                    if (item / "syntax.ron").exists():
                        folders.append(item.name)
        return sorted(folders)
    
    def _count_markdown_functions(self, folder_name: str) -> Dict[str, int]:
        """
        Count markdown functions in syntax and AST files.
        
        Args:
            folder_name (str): Name of the schema folder
            
        Returns:
            Dict[str, int]: Count of functions in each file
        """
        folder_path = self.base_path / folder_name
        counts = {"syntax": 0, "ast": 0}
        
        # Count syntax functions
        syntax_file = folder_path / "syntax.ron"
        if syntax_file.exists():
            try:
                with open(syntax_file, 'r', encoding='utf-8') as f:
                    content = f.read()
                    # Count node definitions (lines with { that aren't comments)
                    lines = content.split('\n')
                    for line in lines:
                        line = line.strip()
                        if line and not line.startswith('#') and '{' in line:
                            counts["syntax"] += 1
            except Exception:
                pass
        
        # Count AST functions
        ast_file = folder_path / "ast.ron"
        if ast_file.exists():
            try:
                with open(ast_file, 'r', encoding='utf-8') as f:
                    content = f.read()
                    # Count node instances (lines with type: that aren't comments)
                    lines = content.split('\n')
                    for line in lines:
                        line = line.strip()
                        if line and not line.startswith('#') and 'type:' in line:
                            counts["ast"] += 1
            except Exception:
                pass
        
        return counts
    
    def display_menu(self) -> None:
        """Display the main CLI menu."""
        clear_screen()
        print("=" * 70)
        print("AST Syntax Checker & Display Hints Generator")
        print("=" * 70)
        print()
        print("Available schema folders:")
        print("-" * 40)
        
        if not self.schema_folders:
            print("No schema folders found!")
            print(f"   Expected path: {self.base_path}")
            return
        
        for i, folder in enumerate(self.schema_folders, 1):
            folder_path = self.base_path / folder
            has_ast = (folder_path / "ast.ron").exists()
            has_syntax = (folder_path / "syntax.ron").exists()
            has_display_hints = (folder_path / "display_hints.ron").exists()
            
            # Get function counts
            counts = self._count_markdown_functions(folder)
            
            status_icons = []
            if has_syntax:
                status_icons.append(f"syntax({counts['syntax']})")
            if has_ast:
                status_icons.append(f"ast({counts['ast']})")
            if has_display_hints:
                status_icons.append("hints")
            
            status = " | ".join(status_icons) if status_icons else "incomplete"
            print(f"  {i}. {folder:<15} [{status}]")
        
        print()
        print("Numbers in parentheses show markdown function counts")
        print("Select a folder to start working with it, or 'q' to quit.")
        print()
    
    def select_folder(self) -> Optional[str]:
        """
        Allow user to select a schema folder.
        
        Returns:
            Optional[str]: Selected folder name, or None if cancelled/quit
        """
        if not self.schema_folders:
            return None
        
        while True:
            try:
                choice = input(f"Select folder (1-{len(self.schema_folders)}) or 'q' to quit: ").strip()
                
                if choice.lower() == 'q':
                    return None
                
                folder_index = int(choice) - 1
                if 0 <= folder_index < len(self.schema_folders):
                    return self.schema_folders[folder_index]
                else:
                    clear_screen()
                    print(f"Please enter a number between 1 and {len(self.schema_folders)}")
                    self.display_menu()
                    print("Select a schema folder to work with:")
            
            except ValueError:
                clear_screen()
                print("Please enter a valid number or 'q' to quit")
                self.display_menu()
                print("Select a schema folder to work with:")
            except (KeyboardInterrupt, EOFError):
                clear_screen()
                print("\nGoodbye!")
                return None
    
    def load_ron_file(self, file_path: Path) -> Optional[Dict[str, Any]]:
        """
        Load and parse a RON file.
        
        Args:
            file_path (Path): Path to the RON file
            
        Returns:
            Optional[Dict[str, Any]]: Parsed data or None if error
        """
        try:
            if not file_path.exists():
                print(f"File not found: {file_path}")
                return None
            
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            return self._parse_ron_content(content)
        
        except Exception as e:
            print(f"Error loading {file_path}: {e}")
            return None
    
    def _parse_ron_content(self, content: str) -> Dict[str, Any]:
        """
        Parse RON content into a Python dictionary.
        This is a simplified parser for the specific RON format used in this project.
        
        Args:
            content (str): RON content to parse
            
        Returns:
            Dict[str, Any]: Parsed data
        """
        # Remove comments
        lines = []
        for line in content.split('\n'):
            # Remove comments (lines starting with #)
            if line.strip() and not line.strip().startswith('#'):
                lines.append(line)
        
        content = '\n'.join(lines).strip()
        
        if not content:
            return {}
        
        # For the specific RON format in this project, try to extract the main structure
        # Look for the main node definition
        try:
            # Handle the case where we have a single root node definition
            # like: RootNode { ... }
            if content.startswith(('RootNode', 'Document')):
                # Extract the main structure
                result = self._parse_ron_object(content)
                return result
            
            # Handle syntax.ron format with multiple node definitions
            elif '{' in content and 'node_type' in content:
                # This looks like syntax.ron format
                result = {}
                # Split by lines and parse each node definition
                current_node = ""
                for line in content.split('\n'):
                    line = line.strip()
                    if line and '{' in line:
                        if current_node:
                            # Parse the previous node
                            node_data = self._parse_syntax_node(current_node)
                            if node_data:
                                result.update(node_data)
                        current_node = line
                    elif line and current_node:
                        current_node += " " + line
                
                # Parse the last node
                if current_node:
                    node_data = self._parse_syntax_node(current_node)
                    if node_data:
                        result.update(node_data)
                
                return result
            
            else:
                # Fallback: return raw content wrapped
                return {"raw_content": content}
                
        except Exception as e:
            print(f"Warning: RON parsing failed: {e}")
            return {"error": f"Could not parse RON content: {e}", "raw_content": content}
    
    def _parse_ron_object(self, content: str) -> Dict[str, Any]:
        """Parse a single RON object definition with proper nested structure."""
        # This is a more comprehensive parser for the AST format
        # that handles the nested structure properly
        try:
            # Remove the outer node name (e.g., "RootNode") and get the content
            if '{' not in content:
                return {}
            
            brace_start = content.find('{')
            brace_end = content.rfind('}')
            
            if brace_start == -1 or brace_end == -1:
                return {}
            
            # Extract the content between braces
            inner_content = content[brace_start + 1:brace_end].strip()
            
            # Parse the content using a more robust approach
            return self._parse_ron_structure(inner_content)
            
        except Exception as e:
            print(f"Error parsing RON object: {e}")
            return {}
    
    def _parse_ron_structure(self, content: str) -> Dict[str, Any]:
        """Parse RON structure content into a Python dictionary."""
        result = {}
        
        # Split content into key-value pairs and nested structures
        i = 0
        while i < len(content):
            # Skip whitespace and commas
            while i < len(content) and content[i] in ' \t\n,':
                i += 1
                
            if i >= len(content):
                break
                
            # Find the key
            key_start = i
            while i < len(content) and content[i] not in ':':
                i += 1
                
            if i >= len(content):
                break
                
            key = content[key_start:i].strip()
            i += 1  # Skip the ':'
            
            # Skip whitespace after colon
            while i < len(content) and content[i] in ' \t\n':
                i += 1
                
            # Parse the value
            if i < len(content):
                if content[i] == '"':
                    # String value
                    i += 1  # Skip opening quote
                    value_start = i
                    while i < len(content) and content[i] != '"':
                        if content[i] == '\\':
                            i += 2  # Skip escaped character
                        else:
                            i += 1
                    value = content[value_start:i]
                    i += 1  # Skip closing quote
                    result[key] = value
                    
                elif content[i] == '[':
                    # Array value
                    array_content, new_i = self._parse_ron_array(content, i)
                    result[key] = array_content
                    i = new_i
                    
                elif content[i:i+4] == 'Some':
                    # Optional value
                    i += 4
                    if i < len(content) and content[i] == '(':
                        i += 1
                        value_start = i
                        paren_count = 1
                        while i < len(content) and paren_count > 0:
                            if content[i] == '(':
                                paren_count += 1
                            elif content[i] == ')':
                                paren_count -= 1
                            i += 1
                        value_content = content[value_start:i-1]
                        if value_content.isdigit():
                            result[key] = int(value_content)
                        elif value_content in ['true', 'false']:
                            result[key] = value_content == 'true'
                        else:
                            result[key] = value_content.strip('"')
                            
                elif content[i].isdigit() or content[i] == '-':
                    # Numeric value
                    value_start = i
                    while i < len(content) and (content[i].isdigit() or content[i] in '.-'):
                        i += 1
                    value = content[value_start:i]
                    result[key] = int(value) if value.isdigit() else float(value)
                    
                elif content[i:i+4] == 'true':
                    result[key] = True
                    i += 4
                    
                elif content[i:i+5] == 'false':
                    result[key] = False
                    i += 5
                    
                else:
                    # Try to find the end of this value (until comma or end)
                    value_start = i
                    brace_count = 0
                    while i < len(content):
                        if content[i] == '{':
                            brace_count += 1
                        elif content[i] == '}':
                            brace_count -= 1
                        elif content[i] == ',' and brace_count == 0:
                            break
                        i += 1
                    
                    value = content[value_start:i].strip()
                    if value:
                        result[key] = value
        
        return result
    
    def _parse_ron_array(self, content: str, start_index: int) -> tuple:
        """Parse a RON array and return the content and new index position."""
        i = start_index + 1  # Skip the opening '['
        array_content = []
        
        # Skip whitespace
        while i < len(content) and content[i] in ' \t\n':
            i += 1
            
        while i < len(content) and content[i] != ']':
            # Parse each array element
            if content[i] in ' \t\n,':
                i += 1
                continue
                
            # Check if this is a nested object
            if content[i].isupper():  # Likely a node type like "Heading", "TextNode", etc.
                # Find the node type and its content
                node_start = i
                while i < len(content) and content[i] not in ' \t\n{':
                    i += 1
                node_type = content[node_start:i]
                
                # Skip whitespace
                while i < len(content) and content[i] in ' \t\n':
                    i += 1
                    
                if i < len(content) and content[i] == '{':
                    # Parse the node content
                    brace_count = 1
                    node_content_start = i + 1
                    i += 1
                    
                    while i < len(content) and brace_count > 0:
                        if content[i] == '{':
                            brace_count += 1
                        elif content[i] == '}':
                            brace_count -= 1
                        i += 1
                    
                    node_content = content[node_content_start:i-1]
                    parsed_node = self._parse_ron_structure(node_content)
                    array_content.append(parsed_node)
            else:
                # Simple value
                value_start = i
                while i < len(content) and content[i] not in ',]':
                    i += 1
                value = content[value_start:i].strip().strip('"')
                if value:
                    array_content.append(value)
        
        if i < len(content) and content[i] == ']':
            i += 1  # Skip the closing ']'
            
        return array_content, i
    
    def _parse_syntax_node(self, node_line: str) -> Dict[str, Any]:
        """Parse a single syntax node definition."""
        try:
            # Example: Heading1 { node_type: "heading", depth: 1, markdown_syntax: "#" }
            if '{' not in node_line:
                return {}
            
            name_end = node_line.find('{')
            node_name = node_line[:name_end].strip()
            
            # Extract properties between braces
            content = node_line[name_end+1:]
            if '}' in content:
                content = content[:content.rfind('}')]
            
            # Parse properties
            properties = {}
            # Simple property parsing
            for prop in content.split(','):
                prop = prop.strip()
                if ':' in prop:
                    key, value = prop.split(':', 1)
                    key = key.strip()
                    value = value.strip().strip('"')
                    properties[key] = value
            
            return {node_name: properties}
            
        except Exception:
            return {}
    
    def save_ron_file(self, data: Dict[str, Any], file_path: Path) -> bool:
        """
        Save data to a RON file.
        
        Args:
            data (Dict[str, Any]): Data to save
            file_path (Path): Path where to save the file
            
        Returns:
            bool: True if successful, False otherwise
        """
        try:
            # Convert Python dict to RON-like format
            ron_content = self._format_as_ron(data)
            
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(ron_content)
            return True
        
        except Exception as e:
            print(f"Error saving {file_path}: {e}")
            return False
    
    def _format_as_ron(self, data: Dict[str, Any], indent: int = 0) -> str:
        """
        Format Python dictionary as RON-like content.
        
        Args:
            data (Dict[str, Any]): Data to format
            indent (int): Current indentation level
            
        Returns:
            str: RON-formatted string
        """
        lines = []
        indent_str = "    " * indent
        
        for key, value in data.items():
            if isinstance(value, dict):
                lines.append(f"{indent_str}{key} {{")
                for sub_key, sub_value in value.items():
                    if isinstance(sub_value, (str, int, float, bool)):
                        lines.append(f"{indent_str}    {sub_key}: {repr(sub_value) if isinstance(sub_value, str) else sub_value},")
                    elif isinstance(sub_value, list):
                        lines.append(f"{indent_str}    {sub_key}: [")
                        for item in sub_value:
                            if isinstance(item, dict):
                                lines.append(self._format_as_ron({f"item": item}, indent + 2))
                            else:
                                lines.append(f"{indent_str}        {repr(item) if isinstance(item, str) else item},")
                        lines.append(f"{indent_str}    ],")
                lines.append(f"{indent_str}}}")
            else:
                lines.append(f"{indent_str}{key}: {repr(value) if isinstance(value, str) else value}")
        
        return "\n".join(lines)
    
    def generate_display_hints(self, folder_name: str) -> bool:
        """
        Generate display hints from AST and syntax.
        First validates AST against syntax - only generates hints if validation passes.
        
        Args:
            folder_name (str): Name of the schema folder
            
        Returns:
            bool: True if successful, False otherwise
        """
        folder_path = self.base_path / folder_name
        
        # Load required files
        syntax_data = self.load_ron_file(folder_path / "syntax.ron")
        ast_data = self.load_ron_file(folder_path / "ast.ron")
        
        if not syntax_data or not ast_data:
            return False
        
        print(f"Generating display hints for '{folder_name}'...")
        
        # First, validate AST against syntax (silent validation)
        print("Validating AST against syntax first...")
        validator = ASTValidator(syntax_data, ast_data)
        validation_errors = validator.validate()
        
        if validation_errors:
            print(f"Validation failed with {len(validation_errors)} error(s):")
            for i, error in enumerate(validation_errors, 1):
                print(f"  {i}. {error}")
            print("Cannot generate display hints until validation errors are resolved.")
            return False
        
        print("AST validation passed! Proceeding with display hints generation...")
        
        generator = DisplayHintsGenerator(syntax_data, ast_data)
        display_hints = generator.generate()
        
        if display_hints:
            output_path = folder_path / "display_hints.ron"
            if self.save_ron_file(display_hints, output_path):
                print(f"Display hints saved to: {output_path}")
                return True
            else:
                return False
        else:
            print("Failed to generate display hints")
            return False
    
    def run(self) -> None:
        """Run the interactive CLI application."""
        try:
            while True:
                self.display_menu()
                
                if not self.schema_folders:
                    print("Press Enter to exit...")
                    input()
                    break
                
                # First, select a folder
                print("Select a schema folder to work with:")
                folder = self.select_folder()
                
                if not folder:
                    # User chose to quit
                    clear_screen()
                    print("Goodbye!")
                    break
                
                # Now show actions for the selected folder
                self.show_folder_actions(folder)
        
        except KeyboardInterrupt:
            clear_screen()
            print("\nGoodbye!")
    
    def show_folder_actions(self, folder: str) -> None:
        """
        Show actions for the selected folder after auto-validation.
        
        Args:
            folder (str): Selected folder name
        """
        while True:
            clear_screen()
            folder_path = self.base_path / folder
            
            # Get file status
            syntax_file = folder_path / "syntax.ron"
            ast_file = folder_path / "ast.ron"
            hints_file = folder_path / "display_hints.ron"
            
            has_syntax = syntax_file.exists()
            has_ast = ast_file.exists()
            has_hints = hints_file.exists()
            
            # Count functions/nodes
            counts = self._count_markdown_functions(folder)
            
            # Count display hints items if file exists
            hints_count = "Missing"
            if has_hints:
                try:
                    hints_data = self.load_ron_file(hints_file)
                    if hints_data and 'children' in hints_data:
                        hints_count = f"Found ({len(hints_data['children'])} items)"
                    else:
                        hints_count = "Found (unknown items)"
                except:
                    hints_count = "Found (parse error)"
            
            print("=" * 70)
            print(f"Working with folder: {folder}")
            print("=" * 70)
            print()
            
            # Auto-validate if we have required files
            validation_errors = []
            if has_syntax and has_ast:
                syntax_data = self.load_ron_file(syntax_file)
                ast_data = self.load_ron_file(ast_file)
                
                if syntax_data and ast_data:
                    validator = ASTValidator(syntax_data, ast_data)
                    validation_errors = validator.validate()
            
            # Show validation status
            if has_syntax and has_ast and not validation_errors:
                print("Validation status: Passed!")
            elif has_syntax and has_ast:
                print("Validation status: Failed!")
            else:
                print("Validation status: Cannot validate (missing files)")
            
            print("-" * 20)
            print(f"  Syntax file: {'Found' if has_syntax else 'Missing'} ({counts['syntax']} functions)")
            print(f"  AST file: {'Found' if has_ast else 'Missing'} ({counts['ast']} nodes)")
            print(f"  Display hints: {hints_count}")
            print(f"  Errors: {len(validation_errors)}")
            print()
            
            # Show appropriate actions based on validation status
            print("Available actions:")
            
            if has_syntax and has_ast and not validation_errors:
                # Validation passed - show generation options
                print("  1) Generate Display Hints")
                print("  2) Show detailed file information")
                print("  3) Go back and select another folder")
                print("  4) Quit")
                print()
                
                choice = input("Choose action (1-4): ").strip()
                
                if choice == '1':
                    clear_screen()
                    success = self.generate_display_hints(folder)
                    if success:
                        print("Display hints generated successfully!")
                    input("Press Enter to continue...")
                
                elif choice == '2':
                    self.show_detailed_info(folder)
                
                elif choice == '3':
                    break  # Go back to folder selection
                
                elif choice == '4':
                    print("Goodbye!")
                    sys.exit(0)
                
                else:
                    clear_screen()
                    print("Invalid choice. Please enter 1-4.")
                    input("Press Enter to continue...")
            
            elif has_syntax and has_ast:
                # Validation failed - show error options
                print("  1) Show errors")
                print("  2) Show detailed file information")
                print("  3) Go back and select another folder")
                print("  4) Quit")
                print()
                
                choice = input("Choose action (1-4): ").strip()
                
                if choice == '1':
                    clear_screen()
                    print("=" * 70)
                    print(f"Validation Errors for: {folder}")
                    print("=" * 70)
                    print(f"Found {len(validation_errors)} error(s):")
                    print()
                    
                    for i, error in enumerate(validation_errors, 1):
                        print(f"  {i}. {error}")
                    
                    print()
                    input("Press Enter to continue...")
                
                elif choice == '2':
                    self.show_detailed_info(folder)
                
                elif choice == '3':
                    break  # Go back to folder selection
                
                elif choice == '4':
                    print("Goodbye!")
                    sys.exit(0)
                
                else:
                    clear_screen()
                    print("Invalid choice. Please enter 1-4.")
                    input("Press Enter to continue...")
            
            else:
                # Missing files - limited options
                print("  1) Show detailed file information")
                print("  2) Go back and select another folder")
                print("  3) Quit")
                print()
                
                choice = input("Choose action (1-3): ").strip()
                
                if choice == '1':
                    self.show_detailed_info(folder)
                
                elif choice == '2':
                    break  # Go back to folder selection
                
                elif choice == '3':
                    print("Goodbye!")
                    sys.exit(0)
                
                else:
                    clear_screen()
                    print("Invalid choice. Please enter 1-3.")
                    input("Press Enter to continue...")
    
    def show_detailed_info(self, folder: str) -> None:
        """
        Show detailed information about files in the selected folder.
        
        Args:
            folder (str): Selected folder name
        """
        clear_screen()
        folder_path = self.base_path / folder
        
        print("=" * 70)
        print(f"Detailed Information for: {folder}")
        print("=" * 70)
        
        # Get function counts
        counts = self._count_markdown_functions(folder)
        
        # Analyze files
        files_info = []
        for file_name in ['syntax.ron', 'ast.ron', 'display_hints.ron', 'syntax_doc.md']:
            file_path = folder_path / file_name
            if file_path.exists():
                try:
                    stat = file_path.stat()
                    size = stat.st_size
                    
                    # Get line count
                    with open(file_path, 'r', encoding='utf-8') as f:
                        lines = len(f.readlines())
                    
                    files_info.append({
                        'name': file_name,
                        'size': size,
                        'lines': lines,
                        'exists': True
                    })
                except Exception:
                    files_info.append({
                        'name': file_name,
                        'size': 0,
                        'lines': 0,
                        'exists': True
                    })
            else:
                files_info.append({
                    'name': file_name,
                    'size': 0,
                    'lines': 0,
                    'exists': False
                })
        
        # Check completeness for later use
        has_syntax = any(f['name'] == 'syntax.ron' and f['exists'] for f in files_info)
        has_ast = any(f['name'] == 'ast.ron' and f['exists'] for f in files_info)
        
        # Perform validation to get error count and failed functions
        validation_errors = []
        failed_functions = set()
        
        if has_syntax and has_ast:
            syntax_file = folder_path / "syntax.ron"
            ast_file = folder_path / "ast.ron"
            syntax_data = self.load_ron_file(syntax_file)
            ast_data = self.load_ron_file(ast_file)
            
            if syntax_data and ast_data:
                validator = ASTValidator(syntax_data, ast_data)
                validation_errors = validator.validate()
                
                # Extract failed function names from errors
                for error in validation_errors:
                    if "Syntax defines" in error and "but AST has no example" in error:
                        # Extract function name from error message
                        if "(" in error:
                            func_name = error.split("Syntax defines ")[1].split(" (")[0]
                            failed_functions.add(func_name)
        
        # Show AST structure with friendly names and depths
        if has_ast:
            print()
            print("AST Structure Analysis:")
            print("-" * 50)
            self._show_ast_structure(folder, failed_functions)
        
        # Show syntax definitions with friendly names
        if has_syntax:
            print()
            print("Syntax Definitions Analysis:")
            print("-" * 50)
            self._show_syntax_structure(folder, failed_functions)
        
        # Summary section at the bottom
        print()
        print("Summary:")
        print(f"  • Syntax definitions: {counts['syntax']}")
        print(f"  • AST nodes: {counts['ast']}")
        
        # Show validation status with error count
        if has_syntax and has_ast:
            if validation_errors:
                print(f"  • Status: Failed ({len(validation_errors)} errors)")
            else:
                print("  • Status: Passed")
        else:
            print("  • Status: Cannot validate (missing files)")
        
        # Add structure overview under summary
        if has_ast:
            structure_overview = self._get_structure_overview(folder)
            if structure_overview:
                print()
                print("Structure Overview:")
                print(f"  • Total nodes: {structure_overview['total_nodes']}")
                print(f"  • Maximum depth: {structure_overview['max_depth']}")
                print(f"  • Node types: {structure_overview['node_types']}")
        
        print()
        input("Press Enter to continue...")
    
    def _show_ast_structure(self, folder: str, failed_functions: Optional[set] = None) -> None:
        """
        Show AST structure with friendly names and depth analysis.
        
        Args:
            folder (str): Selected folder name
            failed_functions (set): Set of failed function names to mark in red
        """
        if failed_functions is None:
            failed_functions = set()
            
        folder_path = self.base_path / folder
        ast_file = folder_path / "ast.ron"
        
        if not ast_file.exists():
            print("AST file not found")
            return
        
        try:
            ast_data = self.load_ron_file(ast_file)
            if not ast_data:
                print("Could not parse AST file")
                return
            
            # Analyze the AST structure
            structure_info = self._analyze_ast_structure(ast_data)
            
            print(f"{'Type':<20} {'Friendly Name':<25} {'Count':<8} {'Max Depth'}")
            print("-" * 70)
            
            for node_type, info in sorted(structure_info.items()):
                friendly_name = self._get_friendly_name(node_type)
                
                # Check if this type has failed functions
                type_failed = any(func.lower().startswith(node_type.lower()) for func in failed_functions)
                
                if type_failed:
                    # Red color for failed functions
                    print(f"\033[91m{node_type:<20} {friendly_name:<25} {info['count']:<8} {info['max_depth']}\033[0m")
                else:
                    print(f"{node_type:<20} {friendly_name:<25} {info['count']:<8} {info['max_depth']}")
            
        except Exception as e:
            print(f"Error analyzing AST structure: {e}")
    
    def _get_structure_overview(self, folder: str) -> dict:
        """
        Get structure overview data for use in summary.
        
        Args:
            folder (str): Selected folder name
            
        Returns:
            dict: Structure overview data
        """
        folder_path = self.base_path / folder
        ast_file = folder_path / "ast.ron"
        
        if not ast_file.exists():
            return {}
        
        try:
            ast_data = self.load_ron_file(ast_file)
            if not ast_data:
                return {}
            
            structure_info = self._analyze_ast_structure(ast_data)
            total_nodes = sum(info['count'] for info in structure_info.values())
            max_overall_depth = max((info['max_depth'] for info in structure_info.values()), default=0)
            
            return {
                'total_nodes': total_nodes,
                'max_depth': max_overall_depth,
                'node_types': len(structure_info)
            }
            
        except Exception:
            return {}
    
    def _show_syntax_structure(self, folder: str, failed_functions: Optional[set] = None) -> None:
        """
        Show syntax definitions with friendly names.
        
        Args:
            folder (str): Selected folder name
            failed_functions (set): Set of failed function names to mark in red
        """
        if failed_functions is None:
            failed_functions = set()
            
        folder_path = self.base_path / folder
        syntax_file = folder_path / "syntax.ron"
        
        if not syntax_file.exists():
            print("Syntax file not found")
            return
        
        try:
            syntax_data = self.load_ron_file(syntax_file)
            if not syntax_data:
                print("Could not parse syntax file")
                return
            
            # Analyze syntax definitions
            syntax_info = self._analyze_syntax_definitions(syntax_data)
            
            print(f"{'Definition':<20} {'Friendly Name':<25} {'Markdown Syntax'}")
            print("-" * 70)
            
            for def_name, info in sorted(syntax_info.items()):
                friendly_name = self._get_friendly_name(info.get('node_type', def_name))
                markdown_syntax = info.get('markdown_syntax', '-')
                
                # Check if this definition failed
                if def_name in failed_functions:
                    # Red color for failed functions
                    print(f"\033[91m{def_name:<20} {friendly_name:<25} {markdown_syntax}\033[0m")
                else:
                    print(f"{def_name:<20} {friendly_name:<25} {markdown_syntax}")
            
        except Exception as e:
            print(f"Error analyzing syntax structure: {e}")
    
    def _analyze_ast_structure(self, ast_data: Dict[str, Any]) -> Dict[str, Dict[str, Any]]:
        """
        Analyze AST structure to get node types, counts, and depths.
        
        Args:
            ast_data: Parsed AST data
            
        Returns:
            Dict containing structure information
        """
        structure_info = {}
        
        def analyze_node(node: Any, current_depth: int = 0):
            if isinstance(node, dict):
                # Check if this is a node with a type
                if 'type' in node:
                    node_type = node['type']
                    if node_type not in structure_info:
                        structure_info[node_type] = {'count': 0, 'max_depth': 0}
                    
                    structure_info[node_type]['count'] += 1
                    structure_info[node_type]['max_depth'] = max(
                        structure_info[node_type]['max_depth'], 
                        current_depth
                    )
                
                # Recursively analyze children
                for key, value in node.items():
                    if key == 'children' and isinstance(value, list):
                        for child in value:
                            analyze_node(child, current_depth + 1)
                    else:
                        analyze_node(value, current_depth)
            
            elif isinstance(node, list):
                for item in node:
                    analyze_node(item, current_depth)
        
        analyze_node(ast_data)
        return structure_info
    
    def _analyze_syntax_definitions(self, syntax_data: Dict[str, Any]) -> Dict[str, Dict[str, Any]]:
        """
        Analyze syntax definitions to extract information.
        
        Args:
            syntax_data: Parsed syntax data
            
        Returns:
            Dict containing syntax definition information
        """
        syntax_info = {}
        
        # Handle different possible structures in syntax data
        data_to_analyze = syntax_data
        if 'root' in syntax_data:
            data_to_analyze = syntax_data['root']
        
        for key, value in data_to_analyze.items():
            if isinstance(value, dict):
                syntax_info[key] = value
        
        return syntax_info
    
    def _get_friendly_name(self, node_type: str) -> str:
        """
        Get a friendly, human-readable name for a node type.
        
        Args:
            node_type: The technical node type name
            
        Returns:
            Friendly name for display
        """
        friendly_names = {
            'root': 'Document Root',
            'heading': 'Heading',
            'paragraph': 'Paragraph',
            'text': 'Text Content',
            'strong': 'Bold Text',
            'emphasis': 'Italic Text',
            'blockquote': 'Block Quote',
            'list': 'List',
            'listItem': 'List Item',
            'codeBlock': 'Code Block',
            'inlineCode': 'Inline Code',
            'thematicBreak': 'Horizontal Rule',
            'link': 'Link',
            'image': 'Image',
            'delete': 'Strikethrough',
            'table': 'Table',
            'tableRow': 'Table Row',
            'tableCell': 'Table Cell',
            'footnote': 'Footnote',
            'definition': 'Link Definition',
            'frontmatter': 'Front Matter',
            'htmlInline': 'Inline HTML',
            'htmlBlock': 'HTML Block',
            'comment': 'Comment',
            'mathInline': 'Inline Math',
            'mathBlock': 'Math Block',
            'emoji': 'Emoji',
            'hardbreak': 'Line Break',
            'softbreak': 'Soft Break',
        }
        
        return friendly_names.get(node_type, node_type.title())
    
    def _parse_and_display_ron_structure(self, content: str) -> None:
        """
        Parse and display RON structure manually for better visualization.
        
        Args:
            content (str): Raw RON content
        """
        lines = content.split('\n')
        current_depth = 0
        
        for line in lines:
            stripped = line.strip()
            
            # Skip comments and empty lines
            if not stripped or stripped.startswith('#'):
                continue
            
            # Count indentation
            indent_level = (len(line) - len(line.lstrip())) // 4
            
            # Look for node definitions
            if '{' in stripped:
                # Extract node type and info
                node_name = stripped.split('{')[0].strip()
                
                # Check if it's a type definition
                if 'type:' in stripped:
                    # Extract the type value
                    type_match = re.search(r'type:\s*["\']?([^"\'`,}\s]+)["\']?', stripped)
                    if type_match:
                        node_type = type_match.group(1)
                        friendly_name = self._get_friendly_name(node_type)
                        
                        # Extract additional info
                        extra_info = ""
                        
                        # Check for depth (headings)
                        depth_match = re.search(r'depth:\s*(\d+)', stripped)
                        if depth_match:
                            extra_info = f" (H{depth_match.group(1)})"
                        
                        # Check for value content
                        value_match = re.search(r'value:\s*["\']([^"\']*)["\']', stripped)
                        if value_match:
                            value = value_match.group(1)
                            if len(value) > 30:
                                value = value[:27] + "..."
                            extra_info = f" → \"{value}\""
                        
                        # Check for URL
                        url_match = re.search(r'url:\s*["\']([^"\']*)["\']', stripped)
                        if url_match:
                            url = url_match.group(1)
                            if len(url) > 30:
                                url = url[:27] + "..."
                            extra_info = f" → {url}"
                        
                        # Check for ordered/unordered lists
                        if 'ordered:' in stripped:
                            ordered_match = re.search(r'ordered:\s*(true|false)', stripped)
                            if ordered_match:
                                list_type = "ordered" if ordered_match.group(1) == 'true' else "unordered"
                                extra_info = f" ({list_type})"
                        
                        # Display the node
                        prefix = "  " * indent_level
                        tree_char = "├─ " if indent_level > 0 else ""
                        print(f"{prefix}{tree_char} {friendly_name}{extra_info}")
                
                elif node_name and node_name not in ['children', 'RootNode']:
                    # It's a named node without explicit type
                    friendly_name = self._get_friendly_name(node_name.lower())
                    prefix = "  " * indent_level
                    tree_char = "├─ " if indent_level > 0 else ""
                    print(f"{prefix}{tree_char} {friendly_name}")
            
            # Handle array elements
            elif stripped.startswith('[') and 'type:' in stripped:
                type_match = re.search(r'type:\s*["\']?([^"\'`,}\s]+)["\']?', stripped)
                if type_match:
                    node_type = type_match.group(1)
                    friendly_name = self._get_friendly_name(node_type)
                    
                    prefix = "  " * (indent_level + 1)
                    print(f"{prefix}├─ {friendly_name}")
        
        # If no structure was found, show a simple message
        if not any('type:' in line for line in lines):
            print("  📁 Document Root (structure not fully parsed)")
            print("  ├─ Complex RON structure detected")
            print("  └─ Use 'Show file contents' to view raw AST")
    
    def _display_ast_node(self, node: Any, depth: int = 0, is_root: bool = False, parent_type: Optional[str] = None) -> None:
        """
        Recursively display AST node structure with friendly names.
        
        Args:
            node: The AST node to display
            depth (int): Current depth in the tree
            is_root (bool): Whether this is the root node
            parent_type (str): Type of the parent node
        """
        indent = "  " * depth
        prefix = "├─ " if depth > 0 else ""
        
        if isinstance(node, dict):
            if 'type' in node:
                node_type = node['type']
                friendly_name = self._get_friendly_name(node_type)
                
                # Add additional info for specific node types
                extra_info = ""
                if node_type == "heading" and "depth" in node:
                    extra_info = f" (H{node['depth']})"
                elif node_type == "list" and "ordered" in node:
                    list_type = "ordered" if node.get('ordered') else "unordered"
                    extra_info = f" ({list_type})"
                elif node_type == "codeBlock" and "language" in node:
                    extra_info = f" ({node.get('language', 'no-lang')})"
                elif node_type in ["link", "image"] and "url" in node:
                    extra_info = f" → {node['url'][:30]}{'...' if len(node['url']) > 30 else ''}"
                
                print(f"{indent}{prefix} {friendly_name}{extra_info}")
                
                # Show children if they exist
                if 'children' in node and isinstance(node['children'], list):
                    for i, child in enumerate(node['children']):
                        self._display_ast_node(child, depth + 1, False, node_type)
            
            elif is_root:
                # Handle root-level structure
                print(f"{indent}Document Root")
                for key, value in node.items():
                    if key != 'raw_content':
                        print(f"{indent}├─ {key}:")
                        self._display_ast_node(value, depth + 1, False, None)
        
        elif isinstance(node, list):
            for i, item in enumerate(node):
                self._display_ast_node(item, depth, False, parent_type)
    
class ASTValidator:
    """Handles AST validation against syntax definitions."""
    
    def __init__(self, syntax_data: Dict[str, Any], ast_data: Dict[str, Any]):
        """
        Initialize the validator.
        
        Args:
            syntax_data (Dict[str, Any]): Loaded syntax definition
            ast_data (Dict[str, Any]): Loaded AST data
        """
        self.syntax = syntax_data
        self.ast = ast_data
        self.errors = []
        self.syntax_nodes = self._extract_syntax_nodes()
    
    def _extract_syntax_nodes(self) -> Dict[str, Dict[str, Any]]:
        """Extract node definitions from syntax data."""
        nodes = {}
        
        # Handle different possible structures in syntax.ron
        if 'root' in self.syntax:
            data = self.syntax['root']
        else:
            data = self.syntax
        
        # Extract node type definitions
        for key, value in data.items():
            if isinstance(value, dict) and 'node_type' in value:
                nodes[value['node_type']] = value
                # Also map by the key name for direct lookup
                nodes[key.lower()] = value
        
        return nodes
    
    def validate(self) -> List[str]:
        """
        Perform complete AST validation.
        
        Returns:
            List[str]: List of validation errors
        """
        self.errors = []
        
        # 1. Validate AST structure
        ast_root = self.ast
        if 'root' in self.ast:
            ast_root = self.ast['root']
        
        # Look for RootNode or similar structure
        if isinstance(ast_root, dict):
            for key, value in ast_root.items():
                if isinstance(value, dict) and 'type' in value:
                    self._validate_node(value, None, 0)
                elif key.lower() == 'rootnode' and isinstance(value, dict):
                    self._validate_node(value, None, 0)
        
        # 2. Validate syntax coverage (check if AST has examples for all syntax elements)
        self._validate_syntax_coverage()
        
        return self.errors
    
    def _validate_syntax_coverage(self) -> None:
        """Validate that AST provides examples for all defined syntax elements."""
        # Get all nodes from AST
        ast_nodes = self._collect_ast_nodes()
        
        # Check syntax definitions vs AST nodes
        syntax_definitions = self._get_syntax_definitions()
        
        for syntax_name, syntax_def in syntax_definitions.items():
            if isinstance(syntax_def, dict) and 'node_type' in syntax_def:
                node_type = syntax_def['node_type']
                
                # Special handling for headings with depths
                if node_type == 'heading' and 'depth' in syntax_def:
                    required_depth = syntax_def['depth']
                    # Convert to int for comparison (syntax might store as string)
                    if isinstance(required_depth, str) and required_depth.isdigit():
                        required_depth = int(required_depth)
                    
                    # Check if AST has a heading with this specific depth
                    found_depth = False
                    for ast_node in ast_nodes:
                        if (ast_node.get('type') == 'heading' and 
                            ast_node.get('depth') == required_depth):
                            found_depth = True
                            break
                    
                    if not found_depth:
                        self.errors.append(
                            f"Syntax defines {syntax_name} (heading depth {required_depth}) "
                            f"but AST has no example heading with depth {required_depth}"
                        )
                
                else:
                    # Check if AST has any node of this type
                    found_type = any(ast_node.get('type') == node_type for ast_node in ast_nodes)
                    
                    if not found_type:
                        self.errors.append(
                            f"Syntax defines {syntax_name} (node_type: {node_type}) "
                            f"but AST has no example of this node type"
                        )
    
    def _collect_ast_nodes(self) -> List[Dict[str, Any]]:
        """Recursively collect all nodes from the AST."""
        nodes = []
        
        def collect_nodes(node: Any):
            if isinstance(node, dict):
                if 'type' in node:
                    nodes.append(node)
                
                # Recursively collect from children
                for key, value in node.items():
                    if key == 'children' and isinstance(value, list):
                        for child in value:
                            collect_nodes(child)
                    else:
                        collect_nodes(value)
            elif isinstance(node, list):
                for item in node:
                    collect_nodes(item)
        
        collect_nodes(self.ast)
        return nodes
    
    def _get_syntax_definitions(self) -> Dict[str, Any]:
        """Get all syntax definitions from syntax data."""
        if 'root' in self.syntax:
            return self.syntax['root']
        return self.syntax
    
    def _validate_node(self, node: Dict[str, Any], parent_type: Optional[str], position: int) -> None:
        """
        Validate a single AST node.
        
        Args:
            node (Dict[str, Any]): The node to validate
            parent_type (Optional[str]): Type of the parent node
            position (int): Position among siblings
        """
        # Check if node has a type
        if 'type' not in node:
            self.errors.append(f"Node missing 'type' field: {node}")
            return
        
        node_type = node['type']
        
        # 1. Check if node type exists in syntax
        if not self._is_valid_node_type(node_type):
            self.errors.append(f"Unknown node type: {node_type}")
            return
        
        # 2. Check parent-child relationship
        if parent_type and not self._can_be_child_of(node_type, parent_type):
            # For now, be permissive about parent-child relationships
            pass
        
        # 3. Check position rules (e.g., frontmatter must be first)
        self._validate_position_rules(node_type, parent_type, position)
        
        # 4. Check children
        if 'children' in node and isinstance(node['children'], list):
            for i, child in enumerate(node['children']):
                if isinstance(child, dict):
                    self._validate_node(child, node_type, i)
        
        # 5. Check required attributes
        self._validate_attributes(node_type, node)
    
    def _is_valid_node_type(self, node_type: str) -> bool:
        """Check if a node type is defined in syntax."""
        # Check common node types that should be valid
        common_types = {
            'root', 'heading', 'paragraph', 'text', 'strong', 'emphasis', 
            'blockquote', 'list', 'listItem', 'codeBlock', 'inlineCode',
            'thematicBreak', 'link', 'image', 'delete'
        }
        
        return (node_type in common_types or 
                node_type in self.syntax_nodes or 
                node_type.lower() in self.syntax_nodes)
    
    def _can_be_child_of(self, child_type: str, parent_type: str) -> bool:
        """Check if child_type can be a child of parent_type."""
        # Define basic containment rules
        block_types = {'heading', 'paragraph', 'blockquote', 'list', 'codeBlock', 'thematicBreak'}
        inline_types = {'text', 'strong', 'emphasis', 'inlineCode', 'link', 'image', 'delete'}
        
        # Root can contain blocks
        if parent_type == 'root':
            return child_type in block_types
        
        # Paragraphs can contain inline elements
        if parent_type == 'paragraph':
            return child_type in inline_types
        
        # Strong/emphasis can contain inline elements
        if parent_type in {'strong', 'emphasis'}:
            return child_type in inline_types
        
        # Lists can contain list items
        if parent_type == 'list':
            return child_type == 'listItem'
        
        # List items can contain blocks
        if parent_type == 'listItem':
            return child_type in block_types
        
        # Blockquotes can contain blocks
        if parent_type == 'blockquote':
            return child_type in block_types
        
        return True  # Be permissive for now
    
    def _validate_position_rules(self, node_type: str, parent_type: Optional[str], position: int) -> None:
        """Validate position-specific rules."""
        # Example: Frontmatter must be first child of Document
        if node_type.lower() == "frontmatter" and position > 0:
            self.errors.append("Frontmatter node must be first child of Document")
    
    def _validate_attributes(self, node_type: str, node: Dict[str, Any]) -> None:
        """Validate node attributes based on syntax requirements."""
        # Check for required attributes based on node type
        if node_type == 'heading' and 'depth' not in node:
            self.errors.append(f"Heading node missing required 'depth' attribute")
        
        if node_type == 'link' and 'url' not in node:
            self.errors.append(f"Link node missing required 'url' attribute")
        
        if node_type == 'image':
            if 'url' not in node:
                self.errors.append(f"Image node missing required 'url' attribute")
            if 'alt' not in node:
                self.errors.append(f"Image node missing required 'alt' attribute")


class DisplayHintsGenerator:
    """Generates display hints from AST and syntax."""
    
    def __init__(self, syntax_data: Dict[str, Any], ast_data: Dict[str, Any]):
        """
        Initialize the generator.
        
        Args:
            syntax_data (Dict[str, Any]): Loaded syntax definition
            ast_data (Dict[str, Any]): Loaded AST data
        """
        self.syntax = syntax_data
        self.ast = ast_data
        self.syntax_nodes = self._extract_syntax_nodes()
    
    def _extract_syntax_nodes(self) -> Dict[str, Dict[str, Any]]:
        """Extract node definitions from syntax data."""
        nodes = {}
        
        # Handle different possible structures in syntax.ron
        if 'root' in self.syntax:
            data = self.syntax['root']
        else:
            data = self.syntax
        
        # Extract node type definitions
        for key, value in data.items():
            if isinstance(value, dict) and 'node_type' in value:
                nodes[value['node_type']] = value
                # Also map by the key name for direct lookup
                nodes[key.lower()] = value
        
        return nodes
    
    def generate(self) -> Optional[Dict[str, Any]]:
        """
        Generate display hints from AST and syntax.
        
        Returns:
            Optional[Dict[str, Any]]: Generated display hints or None if error
        """
        try:
            # Handle different AST structures
            ast_data = self.ast
            
            # Look for the main content
            root_node = None
            
            # Check if it's wrapped in a root object
            if isinstance(ast_data, dict):
                for key, value in ast_data.items():
                    if key.lower() in ['rootnode', 'document'] or ('type' in value if isinstance(value, dict) else False):
                        root_node = value
                        break
                
                # If no specific root found, use the whole structure
                if not root_node:
                    root_node = ast_data
            
            if not root_node:
                return None
            
            # Generate hints for the root structure
            hints = self._generate_simple_hints(root_node)
            
            return {"display_hints": hints} if hints else None
        
        except Exception as e:
            print(f"Error generating display hints: {e}")
            return None
    
    def _generate_simple_hints(self, data: Any) -> Any:
        """
        Generate simplified display hints from data.
        
        Args:
            data: The data structure to process
            
        Returns:
            Any: Generated hints (can be dict, list, or primitive)
        """
        if isinstance(data, dict):
            if 'type' in data:
                # This looks like an AST node
                node_type = data['type']
                display_info = self._get_display_info(node_type)
                
                hints = {
                    "type": node_type,
                    "style": display_info.get("style", "unknown"),
                    "decoration": display_info.get("decoration", ""),
                    "markdown_syntax": display_info.get("markdown_syntax", ""),
                }
                
                # Add specific attributes
                for key, value in data.items():
                    if key not in ['type', 'children']:
                        hints[f"attr_{key}"] = value
                
                # Process children if they exist
                if 'children' in data and isinstance(data['children'], list):
                    children_hints = []
                    for child in data['children']:
                        child_hints = self._generate_simple_hints(child)
                        if child_hints:
                            children_hints.append(child_hints)
                    
                    if children_hints:
                        hints["children"] = children_hints
                
                return hints
            
            else:
                # Regular dictionary - process all values
                result = {}
                for key, value in data.items():
                    processed = self._generate_simple_hints(value)
                    if processed:
                        result[key] = processed
                return result
        
        elif isinstance(data, list):
            # Process list items
            result = []
            for item in data:
                processed = self._generate_simple_hints(item)
                if processed:
                    result.append(processed)
            return result
        
        else:
            # Primitive value - return as is
            return data
    
    def _generate_node_hints(self, node: Dict[str, Any], parent_type: Optional[str]) -> Optional[Dict[str, Any]]:
        """
        Generate display hints for a single node.
        
        Args:
            node (Dict[str, Any]): The AST node
            parent_type (Optional[str]): Type of the parent node
            
        Returns:
            Optional[Dict[str, Any]]: Display hints for the node
        """
        if 'type' not in node:
            return None
        
        node_type = node['type']
        
        # Get display information from syntax
        display_info = self._get_display_info(node_type)
        
        hints = {
            "type": node_type,
            "style": display_info.get("style", "unknown"),
            "decoration": display_info.get("decoration", ""),
            "markdown_syntax": display_info.get("markdown_syntax", ""),
        }
        
        # Add position information
        if parent_type:
            hints["context"] = f"inside_{parent_type}"
        
        # Add depth information for headings
        if node_type == "heading" and "depth" in node:
            hints["depth"] = node["depth"]
            hints["decoration"] = "#" * node["depth"]
        
        # Add URL information for links and images
        if node_type in ["link", "image"]:
            if "url" in node:
                hints["url"] = node["url"]
            if "alt" in node and node_type == "image":
                hints["alt"] = node["alt"]
        
        # Process children recursively
        if 'children' in node and isinstance(node['children'], list):
            children_hints = []
            for child in node['children']:
                if isinstance(child, dict):
                    child_hints = self._generate_node_hints(child, node_type)
                    if child_hints:
                        children_hints.append(child_hints)
            
            if children_hints:
                hints["children"] = children_hints
        
        # Add node-specific attributes for display
        for key, value in node.items():
            if key not in ['type', 'children'] and not key.startswith('_'):
                hints[f"attr_{key}"] = value
        
        return hints
    
    def _get_display_info(self, node_type: str) -> Dict[str, Any]:
        """
        Get display information for a node type from syntax.
        
        Args:
            node_type (str): The node type
            
        Returns:
            Dict[str, Any]: Display information
        """
        # Check if we have syntax information for this node type
        if node_type in self.syntax_nodes:
            syntax_info = self.syntax_nodes[node_type]
            return {
                "style": self._determine_style(node_type),
                "decoration": syntax_info.get("markdown_syntax", ""),
                "markdown_syntax": syntax_info.get("markdown_syntax", ""),
            }
        
        # Fallback display information
        display_map = {
            "root": {"style": "container", "decoration": "", "markdown_syntax": ""},
            "heading": {"style": "block", "decoration": "#", "markdown_syntax": "#"},
            "paragraph": {"style": "block", "decoration": "", "markdown_syntax": ""},
            "text": {"style": "inline", "decoration": "", "markdown_syntax": ""},
            "strong": {"style": "inline", "decoration": "**", "markdown_syntax": "**"},
            "emphasis": {"style": "inline", "decoration": "*", "markdown_syntax": "*"},
            "blockquote": {"style": "block", "decoration": ">", "markdown_syntax": ">"},
            "list": {"style": "block", "decoration": "-", "markdown_syntax": "-"},
            "listItem": {"style": "block", "decoration": "-", "markdown_syntax": "-"},
            "codeBlock": {"style": "block", "decoration": "```", "markdown_syntax": "```"},
            "inlineCode": {"style": "inline", "decoration": "`", "markdown_syntax": "`"},
            "thematicBreak": {"style": "block", "decoration": "---", "markdown_syntax": "---"},
            "link": {"style": "inline", "decoration": "[]()", "markdown_syntax": "[text](url)"},
            "image": {"style": "inline", "decoration": "![]()", "markdown_syntax": "![alt](url)"},
            "delete": {"style": "inline", "decoration": "~~", "markdown_syntax": "~~"},
        }
        
        return display_map.get(node_type, {"style": "unknown", "decoration": "", "markdown_syntax": ""})
    
    def _determine_style(self, node_type: str) -> str:
        """
        Determine if a node type is block or inline.
        
        Args:
            node_type (str): The node type
            
        Returns:
            str: "block", "inline", or "container"
        """
        block_types = {
            'heading', 'paragraph', 'blockquote', 'list', 'listItem', 
            'codeBlock', 'thematicBreak'
        }
        
        inline_types = {
            'text', 'strong', 'emphasis', 'inlineCode', 'link', 'image', 'delete'
        }
        
        if node_type == 'root':
            return 'container'
        elif node_type in block_types:
            return 'block'
        elif node_type in inline_types:
            return 'inline'
        else:
            return 'unknown'


def main():
    """Main entry point for the CLI application."""
    # Determine the base path for markdown schemas
    script_dir = Path(__file__).parent
    # Use workspace root: /home/ranrar/Code/projects/marco2
    project_root = script_dir.parent.parent
    schema_path = project_root / "src" / "assets" / "markdown_schema"
    
    print(f"🔍 Looking for schemas in: {schema_path}")
    
    # Initialize and run the application
    checker = ASTSyntaxChecker(str(schema_path))
    checker.run()


if __name__ == "__main__":
    main()
