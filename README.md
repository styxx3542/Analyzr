# Complexity Audit

A CLI tool to analyze Python code for cyclomatic complexity, printing summaries and highlighting functions exceeding a given threshold.

## Installation

```bash
cargo install --path .
```

## Usage

```bash
complexity-audit <path> [OPTIONS]
```

### Options

- `--threshold <n>`: Highlight functions with complexity > n (default: 10)
- `--output <table|json>`: Output format (default: table)
- `--summary`: Display summary statistics (mean, max, count, etc.)

### Examples

Analyze a Python project with default settings:
```bash
complexity-audit ./my_project
```

Analyze with a custom threshold and JSON output:
```bash
complexity-audit ./my_project --threshold 15 --output json
```

Show summary statistics:
```bash
complexity-audit ./my_project --summary
```

## Features

- Recursively scans Python files in the given directory
- Excludes `__pycache__` and `venv` directories
- Calculates cyclomatic complexity using tree-sitter
- Supports both table and JSON output formats
- Highlights functions exceeding the complexity threshold
- Provides summary statistics

## How it Works

The tool uses tree-sitter to parse Python code and calculate cyclomatic complexity by counting:
- Base complexity (1)
- if statements
- elif clauses
- for loops
- while loops
- try blocks
- except clauses
- with statements
- boolean operators (and/or)
