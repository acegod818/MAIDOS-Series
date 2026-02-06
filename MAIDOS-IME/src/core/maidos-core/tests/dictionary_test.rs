//! Dictionary module tests

use maidos_core::dictionary::Dictionary;

#[test]
fn test_dictionary_loading() {
    // Test dictionary loading
    let dict_result = Dictionary::load_from_file("../../dicts/pinyin.dict.json");
    assert!(dict_result.is_ok());
    
    let dict = dict_result.expect("Failed to load dictionary");
    assert!(!dict.entries.is_empty());
    
    // Test lookup functionality
    let entries = dict.lookup("ní hǎo");
    assert!(entries.is_some());
    
    let entries = entries.expect("Failed to get entries");
    assert!(!entries.is_empty());
    
    // Check the first entry
    let first_entry = &entries[0];
    assert_eq!(first_entry.word, "你好");
    assert_eq!(first_entry.pronunciation, "ní hǎo");
    assert_eq!(first_entry.frequency, 1000);
}

#[test]
fn test_pinyin_parser() {
    use maidos_core::dictionary::PinyinParser;
    
    // Test pinyin parser
    let parser_result = PinyinParser::new("../../dicts/pinyin.dict.json");
    assert!(parser_result.is_ok());
    
    let parser = parser_result.expect("Failed to create parser");
    let entries = parser.parse_pinyin("ní hǎo");
    
    assert!(!entries.is_empty());
    assert_eq!(entries[0].word, "你好");
}
