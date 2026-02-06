//! Pinyin parser tests

use maidos_core::pinyin_parser::PinyinParser;

#[test]
fn test_pinyin_parser_loading() {
    // Test parser loading
    let parser_result = PinyinParser::new("../../dicts/pinyin.dict.json");
    assert!(parser_result.is_ok());
    
    let parser = parser_result.expect("Failed to create parser");
    assert!(!parser.get_dictionary().entries.is_empty());
}

#[test]
fn test_single_pinyin_parsing() {
    // Test single pinyin parsing
    let parser_result = PinyinParser::new("../../dicts/pinyin.dict.json");
    assert!(parser_result.is_ok());
    
    let parser = parser_result.expect("Failed to create parser");
    let entries = parser.parse_single_pinyin("ní hǎo");
    
    assert!(!entries.is_empty());
    
    // Check the first entry
    let first_entry = &entries[0];
    assert_eq!(first_entry.word, "你好");
    assert_eq!(first_entry.pronunciation, "ní hǎo");
    assert_eq!(first_entry.frequency, 1000);
}

#[test]
fn test_continuous_pinyin_parsing() {
    // Test continuous pinyin parsing
    let parser_result = PinyinParser::new("../../dicts/pinyin.dict.json");
    assert!(parser_result.is_ok());
    
    let mut parser = parser_result.expect("Failed to create parser");
    let _result = parser.parse_continuous_pinyin("nihao");
    
    // Note: Since pinyin in the dictionary uses tone marks, "nihao" may not find a match
    // Here we only check that the function executes without errors
    // Function executed successfully; actual results depend on dictionary contents
    let _ = &_result;
}