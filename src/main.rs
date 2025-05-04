use anyhow::{Context, Result};
use clap::Parser;
use comfy_table::{Cell, Table};
use serde::Serialize;
use std::path::PathBuf;
use tree_sitter::{Query, QueryCursor};
use tree_sitter::Parser as TSParser;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to analyze
    path: PathBuf,

    /// Complexity threshold to highlight
    #[arg(short, long, default_value_t = 10)]
    threshold: u32,

    /// Output format
    #[arg(short, long, default_value = "table")]
    output: String,

    /// Display summary statistics
    #[arg(short, long)]
    summary: bool,
}

#[derive(Debug, Serialize)]
struct FunctionComplexity {
    name: String,
    file: String,
    line: u32,
    complexity: u32,
}

#[derive(Debug, Serialize)]
struct AnalysisResult {
    functions: Vec<FunctionComplexity>,
    summary: Option<Summary>,
}

#[derive(Debug, Serialize)]
struct Summary {
    mean_complexity: f64,
    max_complexity: u32,
    total_functions: usize,
    functions_above_threshold: usize,
}

fn calculate_complexity(source: &str) -> Result<Vec<FunctionComplexity>> {
    let mut parser = TSParser::new();
    let language = tree_sitter_python::language();
    parser.set_language(language).unwrap();

    let tree = parser.parse(source, None).context("Failed to parse Python code")?;
    let mut results = Vec::new();

    let query = Query::new(
        language,
        "(function_definition
            name: (identifier) @name
            body: (block) @body) @function",
    )?;

    let mut query_cursor = QueryCursor::new();
    let matches = query_cursor.matches(&query, tree.root_node(), source.as_bytes());

    for m in matches {
        let function_node = m.captures[0].node;
        let name_node = m.captures[1].node;
        let body_node = m.captures[2].node;

        let name = name_node.utf8_text(source.as_bytes())?;
        let mut complexity = 1; // Base complexity

        let control_flow_query = Query::new(
            language,
            "(if_statement) @if
             (elif_clause) @elif
             (for_statement) @for
             (while_statement) @while
             (try_statement) @try
             (except_clause) @except
             (with_statement) @with
             (boolean_operator) @bool_op",
        )?;

        let mut control_cursor = QueryCursor::new();
        let control_matches = control_cursor.matches(&control_flow_query, body_node, source.as_bytes());

        for _ in control_matches {
            complexity += 1;
        }

        results.push(FunctionComplexity {
            name: name.to_string(),
            file: "".to_string(), // Will be set by caller
            line: function_node.start_position().row as u32 + 1,
            complexity,
        });
    }

    Ok(results)
}

fn analyze_directory(path: &PathBuf, threshold: u32) -> Result<AnalysisResult> {
    let mut all_functions = Vec::new();
    let mut total_complexity = 0u64;
    let mut max_complexity = 0u32;

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "py"))
    {
        let file_path = entry.path();
        if file_path.to_string_lossy().contains("__pycache__")
            || file_path.to_string_lossy().contains("venv")
        {
            continue;
        }

        let source = std::fs::read_to_string(file_path)?;
        let mut functions = calculate_complexity(&source)?;

        for func in &mut functions {
            func.file = file_path.to_string_lossy().to_string();
            total_complexity += func.complexity as u64;
            max_complexity = max_complexity.max(func.complexity);
        }

        all_functions.extend(functions);
    }

    let summary = if !all_functions.is_empty() {
        Some(Summary {
            mean_complexity: total_complexity as f64 / all_functions.len() as f64,
            max_complexity,
            total_functions: all_functions.len(),
            functions_above_threshold: all_functions
                .iter()
                .filter(|f| f.complexity > threshold)
                .count(),
        })
    } else {
        None
    };

    Ok(AnalysisResult {
        functions: all_functions,
        summary,
    })
}

fn print_table(result: &AnalysisResult, threshold: u32) {
    let mut table = Table::new();
    table.set_header(vec!["Function", "File", "Line", "Complexity"]);

    for func in &result.functions {
        let mut row = vec![
            Cell::new(&func.name),
            Cell::new(&func.file),
            Cell::new(func.line.to_string()),
            Cell::new(func.complexity.to_string()),
        ];

        if func.complexity > threshold {
            row[3] = Cell::new(func.complexity.to_string()).fg(comfy_table::Color::Red);
        }

        table.add_row(row);
    }

    println!("{}", table);

    if let Some(summary) = &result.summary {
        println!("\nSummary:");
        println!("Mean Complexity: {:.2}", summary.mean_complexity);
        println!("Max Complexity: {}", summary.max_complexity);
        println!("Total Functions: {}", summary.total_functions);
        println!(
            "Functions above threshold ({}): {}",
            threshold, summary.functions_above_threshold
        );
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let result = analyze_directory(&args.path, args.threshold)?;

    match args.output.as_str() {
        "table" => print_table(&result, args.threshold),
        "json" => println!("{}", serde_json::to_string_pretty(&result)?),
        _ => anyhow::bail!("Invalid output format"),
    }

    Ok(())
} 
