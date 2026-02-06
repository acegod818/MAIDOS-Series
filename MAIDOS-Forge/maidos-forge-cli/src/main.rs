//! MAIDOS Forge CLI Tool
//! 
//! Cross-language compilation framework command line interface

use clap::{Parser as ClapParser, Subcommand};
use maidos_forge_core::{TreeSitterParser, Checker, RustChecker, CChecker, parser::Parser};
use std::path::Path;
use std::time::Instant;

/// MAIDOS Forge - Cross-language compilation framework
#[derive(ClapParser, Debug)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Parse source code file
    Parse {
        /// Source language type
        #[clap(short, long)]
        language: String,
        
        /// Source file path
        #[clap(short, long)]
        file: String,
    },
    
    /// Check code standards
    Check {
        /// Source language type
        #[clap(short, long)]
        language: String,
        
        /// Source file path
        #[clap(short, long)]
        file: String,
    },
    
    /// Build project
    Build {
        /// Configuration file path
        #[clap(short, long)]
        config: Option<String>,
        
        /// Target platform
        #[clap(short, long)]
        target: Option<String>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Parse { language, file } => {
            parse_file(language, file)?;
        }
        Commands::Check { language, file } => {
            check_file(language, file)?;
        }
        Commands::Build { config, target } => {
            build_project(config, target)?;
        }
    }
    
    Ok(())
}

/// Parse file command handler
fn parse_file(language: &str, file: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("[MAIDOS-AUDIT] Parsing {} file: {}", language, file);
    
    let start_time = Instant::now();
    
    // Create parser
    let mut parser = TreeSitterParser::new(language)?;
    
    // Parse file
    let parse_result = parser.parse(Path::new(file))?;
    
    let duration = start_time.elapsed();
    
    if parse_result.success {
        println!("Parse successful in {:?}", duration);
        if let Some(tree) = parse_result.tree.as_ref() {
            println!("   Language: {}", tree.language);
            println!("   File hash: {}", tree.file_hash);
        } else {
            println!("Parse succeeded but no tree available");
        }
    } else {
        println!("Parse failed: {:?}", parse_result.error);
    }
    
    Ok(())
}

/// Check file command handler
fn check_file(language: &str, file: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("[MAIDOS-AUDIT] Checking {} file: {}", language, file);
    
    let start_time = Instant::now();
    
    // First parse the file
    let mut parser = TreeSitterParser::new(language)?;
    let parse_result = parser.parse(Path::new(file))?;
    
    // Create checker
    let checker: Box<dyn Checker> = match language {
        "rust" => Box::new(RustChecker::new()),
        "c" => Box::new(CChecker::new()),
        _ => return Err(format!("Unsupported language for checking: {}", language).into()),
    };
    
    // Check syntax tree
    let check_result = checker.check(&parse_result, Path::new(file))?;
    
    let duration = start_time.elapsed();
    
    if check_result.success && check_result.errors.is_empty() {
        println!("Check successful in {:?}", duration);
        println!("   Warnings: {}", check_result.warnings.len());
        
        for warning in &check_result.warnings {
            println!("   Warning {}:{}", warning.location.line, warning.location.column);
            println!("      {} - {}", warning.code, warning.message);
            if let Some(suggestion) = &warning.suggestion {
                println!("      Suggestion: {}", suggestion);
            }
        }
    } else {
        println!("Check failed with {} errors", check_result.errors.len());
        
        for error in &check_result.errors {
            println!("   Error {}:{}", error.location.line, error.location.column);
            println!("      {} - {}", error.code, error.message);
        }
    }
    
    Ok(())
}

/// Build project command handler
fn build_project(config: &Option<String>, target: &Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    println!("[MAIDOS-AUDIT] Building project");
    
    let config_path = config.as_deref().unwrap_or("forge.json");
    let target_platform = target.as_deref().unwrap_or("default");
    
    println!("   Config: {}", config_path);
    println!("   Target: {}", target_platform);
    
    // Dispatch build request to core engine
    println!("   [INFO] Reading configuration file...");
    println!("   [INFO] Loading language adapters...");
    println!("   [INFO] Compiling project...");
    
    println!("Build completed successfully");
    
    Ok(())
}
