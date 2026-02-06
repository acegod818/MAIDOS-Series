//! 拼音解析器演示程序

use maidos_core::pinyin_parser::PinyinParser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("MAIDOS IME 拼音解析器演示");
    println!("========================");
    
    // 加載詞典
    let mut parser = PinyinParser::new("../../dicts/pinyin.dict.json")?;
    println!("✅ 詞典加載成功");
    
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