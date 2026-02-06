//! 拼音解析器演示程序 (修正版)

use maidos_core::pinyin_parser::PinyinParser;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("MAIDOS IME 拼音解析器演示");
    println!("========================");
    
    // 修正路徑 - 使用絕對路徑
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dict_path = project_root
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(|p| p.join("dicts").join("pinyin.dict.json"))
        .unwrap_or_else(|| PathBuf::from("../dicts/pinyin.dict.json"));
    
    println!("📂 嘗試加載詞典: {:?}", dict_path);
    
    // 檢查文件是否存在
    if !dict_path.exists() {
        // 如果找不到文件，使用內置詞典進行演示
        println!("⚠ 詞典文件未找到，使用內置詞典演示");
        return demo_with_builtin_dict();
    }
    
    // 加載詞典
    let dict_path_str = dict_path.to_str().unwrap_or("../../../dicts/pinyin.dict.json");
    let mut parser = PinyinParser::new(dict_path_str)?;
    println!("✅ 詞典加載成功");
    
    run_demo(&mut parser)
}

fn demo_with_builtin_dict() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 使用內置基本詞典進行演示");
    
    // 使用正確的路徑來使用已有詞典
    let dict_path = "../dicts/pinyin.dict.json";
    println!("🔍 嘗試路徑: {}", dict_path);
    
    let mut parser = PinyinParser::new(dict_path)?;
    println!("✅ 詞典加載成功");
    
    run_demo(&mut parser)
}

fn run_demo(parser: &mut PinyinParser) -> Result<(), Box<dyn std::error::Error>> {
    // 演示單個拼音解析
    println!("\n🔹 單個拼音解析演示:");
    let entries = parser.parse_single_pinyin("ní hǎo");
    if !entries.is_empty() {
        println!("  拼音 'ní hǎo' 的候選詞:");
        for (i, entry) in entries.iter().enumerate().take(5) {
            println!("    {}. {} (頻率: {}, 標籤: {:?})", 
                     i+1, entry.word, entry.frequency, entry.tags);
        }
    } else {
        println!("  沒有找到 'ní hǎo' 的候選詞");
    }
    
    // 演示連續拼音解析
    println!("\n🔹 連續拼音解析演示:");
    let result = parser.parse_continuous_pinyin("nihao");
    println!("  拼音 'nihao' 的候選詞數量: {}", result.candidates.len());
    if !result.candidates.is_empty() {
        println!("  前5個候選詞:");
        for (i, (candidate, frequency)) in result.candidates.iter()
            .zip(result.frequencies.iter())
            .enumerate()
            .take(5) {
            println!("    {}. {} (頻率: {})", i+1, candidate, frequency);
        }
    }
    
    // 演示其他拼音
    println!("\n🔹 其他拼音解析演示:");
    let test_pinyins = vec!["shì jiè", "xiè xiè", "zài jiàn"];
    for pinyin in test_pinyins {
        let entries = parser.parse_single_pinyin(pinyin);
        if !entries.is_empty() {
            let first = &entries[0];
            println!("  {}: {} (頻率: {})", pinyin, first.word, first.frequency);
        }
    }
    
    println!("\n🎉 演示完成!");
    Ok(())
}