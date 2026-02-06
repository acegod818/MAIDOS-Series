//! MAIDOS-IME core example
//!
//! This file demonstrates how to use the core features of MAIDOS-IME.

use maidos_core::{ime_engine::ImeEngine, schemes::SchemeFactory, converter::CharsetConverter};
use maidos_config::{MaidosConfig, Charset};
use std::path::PathBuf;

fn resolve_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Soft-config priority:
    // 1) CLI: --config <path>
    // 2) Env: MAIDOS_CONFIG_PATH
    // 3) Common relative paths (repo / package layouts)
    // 4) Exe directory (portable distribution)

    // (1) CLI arg
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        if a == "--config" {
            if let Some(p) = args.next() {
                return Ok(PathBuf::from(p));
            }
        }
    }

    // (2) Env var
    if let Ok(p) = std::env::var("MAIDOS_CONFIG_PATH") {
        let p = p.trim();
        if !p.is_empty() {
            return Ok(PathBuf::from(p));
        }
    }

    // (3) Repo / working-dir relative fallbacks
    for rel in ["maidos.toml", "config/maidos.toml", "src/config/maidos.toml"] {
        let p = PathBuf::from(rel);
        if p.exists() {
            return Ok(p);
        }
    }

    // (4) Exe-directory fallbacks
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for rel in ["maidos.toml", "config/maidos.toml"] {
                let p = dir.join(rel);
                if p.exists() {
                    return Ok(p);
                }
            }
        }
    }

    Err("Config not found. Provide --config <path> or set MAIDOS_CONFIG_PATH.".into())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("MAIDOS-IME Core Example");
    println!("==================");

    // Load config
    let config_path = resolve_config_path()?;
    let config = MaidosConfig::load(config_path.to_string_lossy().as_ref())?;
    println!("Config loaded successfully");

    // Create IME engine
    let ime_engine = ImeEngine::new(config)?;
    println!("IME engine created successfully");

    // Demonstrate input scheme creation
    let schemes = SchemeFactory::get_supported_schemes();
    println!("Supported input schemes:");
    for scheme in &schemes {
        let scheme_instance = SchemeFactory::create_scheme_simple(scheme);
        println!("  - {:?}", scheme_instance.scheme_type());
    }

    // Demonstrate AI features
    if ime_engine.config.features.ai_selection {
        println!("\nDemonstrating AI character selection:");
        let context = "Today the weather is very";
        let candidates = vec!['\u{597D}', '\u{68D2}', '\u{5DEE}', '\u{7CDF}'];
        println!("Context: {}", context);
        println!("Candidates: {:?}", candidates);

        // Note: This requires an actual Ollama service to work
        // let selected_char = ime_engine.select_character(context, &candidates).await?;
        // println!("AI selected character: {}", selected_char);
        println!("(Requires Ollama service to actually select a character)");
    }

    // Demonstrate auto-correction
    if ime_engine.config.features.auto_correction {
        println!("\nDemonstrating auto-correction:");
        let text = "Today the weather is realy nice";
        println!("Original text: {}", text);

        // Note: This requires an actual Ollama service to work
        // let corrected_text = ime_engine.auto_correct(text).await?;
        // println!("Corrected text: {}", corrected_text);
        println!("(Requires Ollama service to actually correct text)");
    }

    // Demonstrate smart suggestions
    if ime_engine.config.features.smart_suggestions {
        println!("\nDemonstrating smart suggestions:");
        let text = "Today the weather is nice, suitable for";
        println!("Input text: {}", text);

        // Note: This requires an actual Ollama service to work
        // let suggestions = ime_engine.smart_suggestions(text).await?;
        // println!("Suggestions: {:?}", suggestions);
        println!("(Requires Ollama service to actually generate suggestions)");
    }

    // Demonstrate cross-input
    println!("\nDemonstrating cross-input:");
    let input = "nihao";
    let context = "Hello";
    let scheme = "pinyin";
    let charset = Charset::Simplified;

    println!("Input: {}", input);
    println!("Context: {}", context);
    println!("Input scheme: {}", scheme);
    println!("Charset: {:?}", charset);

    // Demonstrate simplified to traditional conversion
    let simplified_text = "\u{4F60}\u{597D}\u{4E16}\u{754C}";
    let traditional_text = CharsetConverter::convert(simplified_text, &Charset::Simplified, &Charset::Traditional);
    println!("Simplified to Traditional: {} -> {}", simplified_text, traditional_text);

    // Demonstrate traditional to simplified conversion
    let traditional_text2 = "\u{6211}\u{5011}\u{8AAA}\u{8A71}";
    let simplified_text2 = CharsetConverter::convert(traditional_text2, &Charset::Traditional, &Charset::Simplified);
    println!("Traditional to Simplified: {} -> {}", traditional_text2, simplified_text2);

    println!("\nExample completed");
    Ok(())
}
