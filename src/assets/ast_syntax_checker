#!/bin/bash
# --------------------------------------------
# AST + Syntax Validator Runner
# --------------------------------------------
# Usage:
# --------------------------------------------

set -e  # Exit immediately if a command fails

# ----------------------------
# Determine script directory
# ----------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VALIDATOR_DIR="$SCRIPT_DIR/tools/ast_syntax_checker"
VENV_DIR="$VALIDATOR_DIR/.venv"

# ----------------------------
# Create virtual environment if missing
# ----------------------------
if [ ! -d "$VENV_DIR" ]; then
    echo "Creating virtual environment at $VENV_DIR..."
    python3 -m venv "$VENV_DIR"
fi

# ----------------------------
# Activate virtual environment and install dependencies
# ----------------------------
source "$VENV_DIR/bin/activate"
pip install --upgrade pip
pip install -r "$VALIDATOR_DIR/requirements.txt"

# ----------------------------
# Run the validator
# ----------------------------
# run main.py in interactive mode (no argument passed)
echo "Launching AST + Syntax Validator in interactive mode..."
"$VENV_DIR/bin/python" "$VALIDATOR_DIR/main.py"


# ----------------------------
# Deactivate virtual environment
# ----------------------------
deactivate 2>/dev/null || true
echo "Done."