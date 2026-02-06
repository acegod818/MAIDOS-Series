//! Input scheme implementations
//!
//! This module provides implementations for various input schemes, including:
//! - Bopomofo
//! - Pinyin
//! - Cangjie
//! - Quick -- based on Cangjie first/last code
//! - Wubi 86
//! - Handwriting -- requires platform hardware, marked unavailable
//! - Voice -- requires platform hardware, marked unavailable
//! - English
//! - Japanese

use std::collections::HashMap;
use crate::Result;
use crate::MaidosError;

/// Input scheme type
#[derive(Debug, Clone, PartialEq)]
pub enum SchemeType {
    /// Bopomofo
    Bopomofo,
    /// Pinyin
    Pinyin,
    /// Cangjie
    Cangjie,
    /// Quick
    Quick,
    /// Wubi
    Wubi,
    /// Handwriting
    Handwriting,
    /// Voice
    Voice,
    /// English
    English,
    /// Japanese
    Japanese,
}

/// Input scheme trait
pub trait InputScheme: Send + Sync {
    /// Get scheme type
    fn scheme_type(&self) -> SchemeType;

    /// Process input
    fn process_input(&self, input: &str) -> Result<Vec<Candidate>>;

    /// Get candidates
    fn get_candidates(&self, input: &str) -> Result<Vec<Candidate>>;

    /// Add user word
    fn add_user_word(&mut self, word: &str, frequency: u32) -> Result<()>;

    /// Remove user word
    fn remove_user_word(&mut self, word: &str) -> Result<()>;

    /// Process cross-input
    fn process_cross_input(&self, input: &str, _target_charset: &maidos_config::Charset) -> Result<Vec<Candidate>> {
        self.process_input(input)
    }
}

/// Candidate
#[derive(Debug, Clone)]
pub struct Candidate {
    /// Character
    pub character: char,
    /// Frequency
    pub frequency: u32,
    /// Pronunciation (pinyin/bopomofo)
    pub pronunciation: String,
}

// ============================================================
// Bopomofo input scheme
// ============================================================

/// Bopomofo table entry
#[derive(Debug, Clone, serde::Deserialize)]
struct BopomofoEntry {
    char: String,
    freq: u32,
}

/// Bopomofo input scheme
pub struct BopomofoScheme {
    table: HashMap<String, Vec<BopomofoEntry>>,
    user_words: HashMap<String, u32>,
}

impl BopomofoScheme {
    /// Create a new Bopomofo input scheme
    pub fn new() -> Self {
        let table = Self::load_builtin_table();
        Self {
            table,
            user_words: HashMap::new(),
        }
    }

    fn load_builtin_table() -> HashMap<String, Vec<BopomofoEntry>> {
        let data = include_str!("../../data/bopomofo_table.json");
        match serde_json::from_str::<HashMap<String, Vec<BopomofoEntry>>>(data) {
            Ok(t) => t,
            Err(_) => Self::build_minimal_table(),
        }
    }

    fn build_minimal_table() -> HashMap<String, Vec<BopomofoEntry>> {
        let mut t = HashMap::new();
        let entries = vec![
            ("ㄅㄚ", vec![("八", 950), ("巴", 900), ("吧", 880), ("把", 870), ("爸", 860)]),
            ("ㄅㄞ", vec![("百", 950), ("白", 940), ("拜", 880)]),
            ("ㄅㄢ", vec![("半", 950), ("班", 940), ("般", 900), ("版", 880)]),
            ("ㄅㄤ", vec![("帮", 950), ("棒", 900)]),
            ("ㄅㄠ", vec![("包", 950), ("報", 940), ("保", 930), ("寶", 920)]),
            ("ㄅㄟ", vec![("北", 950), ("背", 940), ("被", 930), ("杯", 920)]),
            ("ㄅㄣ", vec![("本", 950), ("笨", 900)]),
            ("ㄅㄧ", vec![("比", 950), ("必", 940), ("筆", 930), ("鼻", 920)]),
            ("ㄅㄨ", vec![("不", 999), ("部", 950), ("步", 940), ("布", 930)]),
            ("ㄆㄚ", vec![("怕", 950), ("爬", 940)]),
            ("ㄆㄧ", vec![("皮", 950), ("批", 940)]),
            ("ㄆㄨ", vec![("鋪", 950), ("普", 940)]),
            ("ㄇㄚ", vec![("馬", 950), ("嗎", 940), ("媽", 930), ("麻", 920)]),
            ("ㄇㄟ", vec![("沒", 960), ("美", 950), ("每", 940)]),
            ("ㄇㄣ", vec![("門", 950), ("們", 940)]),
            ("ㄇㄧ", vec![("米", 950), ("密", 940), ("迷", 930)]),
            ("ㄈㄚ", vec![("法", 960), ("發", 950), ("髮", 900)]),
            ("ㄈㄢ", vec![("反", 950), ("飯", 940), ("犯", 930)]),
            ("ㄈㄤ", vec![("方", 960), ("放", 950), ("房", 940)]),
            ("ㄈㄥ", vec![("風", 960), ("封", 940)]),
            ("ㄈㄨ", vec![("夫", 950), ("服", 940), ("福", 930)]),
            ("ㄉㄚ", vec![("大", 999), ("打", 960), ("答", 940)]),
            ("ㄉㄜ", vec![("的", 999), ("得", 980), ("德", 940)]),
            ("ㄉㄧ", vec![("的", 999), ("地", 980), ("低", 950), ("底", 940)]),
            ("ㄉㄨ", vec![("都", 960), ("讀", 950), ("度", 940)]),
            ("ㄉㄨㄥ", vec![("動", 960), ("東", 950), ("冬", 900)]),
            ("ㄊㄚ", vec![("他", 999), ("她", 998), ("它", 990)]),
            ("ㄊㄞ", vec![("太", 960), ("台", 950), ("態", 940)]),
            ("ㄊㄧㄢ", vec![("天", 999), ("田", 960), ("填", 900)]),
            ("ㄊㄨ", vec![("土", 950), ("突", 940), ("圖", 930)]),
            ("ㄋㄚ", vec![("那", 980), ("拿", 950), ("哪", 940)]),
            ("ㄋㄧ", vec![("你", 999), ("泥", 900)]),
            ("ㄋㄩ", vec![("女", 960)]),
            ("ㄌㄞ", vec![("來", 970), ("賴", 900)]),
            ("ㄌㄧ", vec![("力", 960), ("里", 950), ("理", 940), ("利", 930)]),
            ("ㄌㄨ", vec![("路", 950), ("錄", 940)]),
            ("ㄍㄜ", vec![("個", 980), ("各", 960), ("歌", 940)]),
            ("ㄍㄨㄥ", vec![("公", 960), ("工", 950), ("共", 940)]),
            ("ㄎㄜ", vec![("可", 980), ("科", 960), ("課", 940)]),
            ("ㄏㄠ", vec![("好", 999), ("號", 950)]),
            ("ㄏㄜ", vec![("和", 990), ("合", 970), ("河", 960)]),
            ("ㄏㄨㄟ", vec![("會", 990), ("回", 970)]),
            ("ㄐㄧ", vec![("幾", 960), ("機", 950), ("記", 940)]),
            ("ㄐㄧㄡ", vec![("就", 990), ("九", 960), ("久", 950)]),
            ("ㄑㄧ", vec![("其", 970), ("起", 960), ("七", 950)]),
            ("ㄒㄧ", vec![("西", 960), ("洗", 950), ("系", 940)]),
            ("ㄒㄧㄤ", vec![("想", 970), ("向", 960), ("相", 950)]),
            ("ㄓ", vec![("之", 980), ("只", 970), ("知", 960)]),
            ("ㄓㄜ", vec![("這", 990), ("者", 970)]),
            ("ㄓㄨ", vec![("主", 960), ("住", 950), ("注", 940)]),
            ("ㄓㄨㄥ", vec![("中", 990), ("終", 960), ("鍾", 950)]),
            ("ㄔ", vec![("吃", 960), ("尺", 940)]),
            ("ㄔㄨ", vec![("出", 980), ("初", 960), ("除", 950)]),
            ("ㄕ", vec![("是", 999), ("十", 980), ("時", 970)]),
            ("ㄕㄜ", vec![("社", 950), ("設", 940)]),
            ("ㄕㄣ", vec![("身", 960), ("深", 950), ("神", 940)]),
            ("ㄕㄥ", vec![("生", 980), ("聲", 960)]),
            ("ㄕㄨ", vec![("書", 960), ("數", 950), ("術", 940)]),
            ("ㄖ", vec![("日", 970)]),
            ("ㄖㄣ", vec![("人", 999), ("認", 960)]),
            ("ㄗ", vec![("字", 960), ("自", 950)]),
            ("ㄗㄨㄛ", vec![("做", 970), ("作", 960), ("坐", 950)]),
            ("ㄘ", vec![("此", 960), ("次", 950)]),
            ("ㄙ", vec![("四", 960), ("死", 950), ("思", 940)]),
            ("ㄧ", vec![("一", 999), ("以", 990), ("已", 980)]),
            ("ㄧㄠ", vec![("要", 999), ("搖", 950)]),
            ("ㄧㄡ", vec![("有", 999), ("又", 990), ("由", 970)]),
            ("ㄨ", vec![("五", 960), ("物", 950), ("無", 940)]),
            ("ㄨㄛ", vec![("我", 999)]),
            ("ㄨㄟ", vec![("為", 990), ("位", 970), ("味", 960)]),
            ("ㄩ", vec![("魚", 950), ("雨", 940), ("語", 930)]),
            ("ㄩㄢ", vec![("元", 960), ("原", 950), ("遠", 940)]),
            ("ㄚ", vec![("啊", 960), ("阿", 940)]),
            ("ㄞ", vec![("愛", 970), ("挨", 940)]),
            ("ㄢ", vec![("安", 960), ("暗", 940)]),
            ("ㄣ", vec![("恩", 950)]),
            ("ㄦ", vec![("二", 980), ("耳", 960), ("而", 950), ("兒", 940)]),
        ];
        for (key, chars) in entries {
            let v: Vec<BopomofoEntry> = chars.into_iter()
                .map(|(c, f)| BopomofoEntry { char: c.to_string(), freq: f })
                .collect();
            t.insert(key.to_string(), v);
        }
        t
    }

    fn lookup(&self, input: &str) -> Vec<Candidate> {
        let mut results = Vec::new();

        // Exact match
        if let Some(entries) = self.table.get(input) {
            for e in entries {
                if let Some(ch) = e.char.chars().next() {
                    results.push(Candidate {
                        character: ch,
                        frequency: e.freq,
                        pronunciation: input.to_string(),
                    });
                }
            }
        }

        // Prefix match (if no exact match)
        if results.is_empty() {
            for (key, entries) in &self.table {
                if key.starts_with(input) {
                    for e in entries {
                        if let Some(ch) = e.char.chars().next() {
                            results.push(Candidate {
                                character: ch,
                                frequency: e.freq,
                                pronunciation: key.clone(),
                            });
                        }
                    }
                }
            }
        }

        // Merge user-learned entries
        crate::user_learning::merge_user_learned(&mut results, "bopomofo", input);
        results
    }
}

impl Default for BopomofoScheme {
    fn default() -> Self {
        Self::new()
    }
}

impl InputScheme for BopomofoScheme {
    fn scheme_type(&self) -> SchemeType {
        SchemeType::Bopomofo
    }

    fn process_input(&self, input: &str) -> Result<Vec<Candidate>> {
        Ok(self.lookup(input))
    }

    fn get_candidates(&self, input: &str) -> Result<Vec<Candidate>> {
        Ok(self.lookup(input))
    }

    fn add_user_word(&mut self, word: &str, frequency: u32) -> Result<()> {
        self.user_words.insert(word.to_string(), frequency);
        Ok(())
    }

    fn remove_user_word(&mut self, word: &str) -> Result<()> {
        self.user_words.remove(word);
        Ok(())
    }
}

// ============================================================
// Pinyin input scheme — table-driven, same as Bopomofo/Cangjie/Wubi
// ============================================================

/// Pinyin table entry
#[derive(Debug, Clone, serde::Deserialize)]
struct PinyinEntry {
    char: String,
    freq: u32,
}

/// Pinyin input scheme (table-driven)
pub struct PinyinScheme {
    table: HashMap<String, Vec<PinyinEntry>>,
    user_words: HashMap<String, u32>,
}

impl PinyinScheme {
    /// Create a new Pinyin input scheme with builtin table
    pub fn new() -> Self {
        let table = Self::load_builtin_table();
        Self {
            table,
            user_words: HashMap::new(),
        }
    }

    /// Create from external dictionary file (advanced usage)
    pub fn new_from_file(dict_path: &str) -> Result<Self> {
        let parser = crate::pinyin_parser::PinyinParser::new(dict_path)?;
        // Convert parser dictionary to our table format
        let mut table: HashMap<String, Vec<PinyinEntry>> = HashMap::new();
        for (key, entries) in parser.get_dictionary().get_all_entries() {
            let converted: Vec<PinyinEntry> = entries.iter().map(|e| PinyinEntry {
                char: e.word.clone(),
                freq: e.frequency,
            }).collect();
            table.insert(key.clone(), converted);
        }
        Ok(Self { table, user_words: HashMap::new() })
    }

    /// Alias for backward compatibility
    pub fn new_default() -> Self {
        Self::new()
    }

    fn load_builtin_table() -> HashMap<String, Vec<PinyinEntry>> {
        let data = include_str!("../../data/pinyin_table.json");
        match serde_json::from_str::<HashMap<String, Vec<PinyinEntry>>>(data) {
            Ok(t) => t,
            Err(_) => Self::build_minimal_table(),
        }
    }

    fn build_minimal_table() -> HashMap<String, Vec<PinyinEntry>> {
        let mut t = HashMap::new();
        let entries: Vec<(&str, Vec<(&str, u32)>)> = vec![
            ("a", vec![("啊", 960), ("阿", 950)]),
            ("ai", vec![("愛", 990), ("哀", 940)]),
            ("an", vec![("安", 980), ("暗", 960)]),
            ("ba", vec![("八", 980), ("巴", 960), ("把", 950)]),
            ("bai", vec![("百", 980), ("白", 975)]),
            ("ban", vec![("半", 980), ("班", 975)]),
            ("bu", vec![("不", 999), ("部", 985)]),
            ("da", vec![("大", 999), ("打", 985)]),
            ("de", vec![("的", 999), ("得", 995)]),
            ("di", vec![("地", 999), ("低", 980)]),
            ("dou", vec![("都", 999), ("斗", 970)]),
            ("duo", vec![("多", 999), ("朵", 960)]),
            ("er", vec![("二", 990), ("耳", 970)]),
            ("ge", vec![("個", 999), ("各", 985)]),
            ("guo", vec![("國", 999), ("過", 995)]),
            ("hao", vec![("好", 999), ("號", 985)]),
            ("he", vec![("和", 999), ("合", 990)]),
            ("hen", vec![("很", 999), ("恨", 970)]),
            ("ji", vec![("幾", 995), ("機", 990)]),
            ("jiu", vec![("就", 999), ("九", 985)]),
            ("kan", vec![("看", 999), ("砍", 960)]),
            ("ke", vec![("可", 999), ("科", 985)]),
            ("lai", vec![("來", 999), ("賴", 960)]),
            ("le", vec![("了", 999), ("樂", 990)]),
            ("ma", vec![("馬", 990), ("嗎", 985)]),
            ("me", vec![("麼", 990)]),
            ("men", vec![("門", 990), ("們", 985)]),
            ("na", vec![("那", 999), ("拿", 985)]),
            ("ni", vec![("你", 999), ("泥", 960)]),
            ("ren", vec![("人", 999), ("認", 990)]),
            ("shi", vec![("是", 999), ("時", 998)]),
            ("shui", vec![("水", 999), ("睡", 985)]),
            ("ta", vec![("他", 999), ("她", 998)]),
            ("wo", vec![("我", 999), ("握", 975)]),
            ("xiang", vec![("想", 999), ("向", 995)]),
            ("yi", vec![("一", 999), ("以", 998)]),
            ("you", vec![("有", 999), ("又", 995)]),
            ("zai", vec![("在", 999), ("再", 995)]),
            ("zhe", vec![("這", 999), ("者", 995)]),
            ("zhi", vec![("之", 999), ("只", 998)]),
            ("zhong", vec![("中", 999), ("重", 995)]),
            ("zuo", vec![("做", 999), ("作", 995)]),
        ];
        for (key, chars) in entries {
            let v: Vec<PinyinEntry> = chars.into_iter()
                .map(|(c, f)| PinyinEntry { char: c.to_string(), freq: f })
                .collect();
            t.insert(key.to_string(), v);
        }
        t
    }

    fn lookup(&self, input: &str) -> Vec<Candidate> {
        let lower = input.to_lowercase();
        let mut results = Vec::new();

        // Exact match
        if let Some(entries) = self.table.get(&lower) {
            for e in entries {
                if let Some(ch) = e.char.chars().next() {
                    results.push(Candidate {
                        character: ch,
                        frequency: e.freq,
                        pronunciation: lower.clone(),
                    });
                }
            }
        }

        // Prefix match (if no exact match)
        if results.is_empty() {
            for (key, entries) in &self.table {
                if key.starts_with(&lower) {
                    for e in entries {
                        if let Some(ch) = e.char.chars().next() {
                            results.push(Candidate {
                                character: ch,
                                frequency: e.freq,
                                pronunciation: key.clone(),
                            });
                        }
                    }
                }
            }
        }

        // Merge user-learned entries
        crate::user_learning::merge_user_learned(&mut results, "pinyin", &lower);
        results
    }
}

impl Default for PinyinScheme {
    fn default() -> Self {
        Self::new()
    }
}

impl InputScheme for PinyinScheme {
    fn scheme_type(&self) -> SchemeType {
        SchemeType::Pinyin
    }

    fn process_input(&self, input: &str) -> Result<Vec<Candidate>> {
        Ok(self.lookup(input))
    }

    fn get_candidates(&self, input: &str) -> Result<Vec<Candidate>> {
        Ok(self.lookup(input))
    }

    fn add_user_word(&mut self, word: &str, frequency: u32) -> Result<()> {
        self.user_words.insert(word.to_lowercase(), frequency);
        Ok(())
    }

    fn remove_user_word(&mut self, word: &str) -> Result<()> {
        self.user_words.remove(&word.to_lowercase());
        Ok(())
    }
}

// ============================================================
// Cangjie input scheme
// ============================================================

/// Cangjie table entry
#[derive(Debug, Clone, serde::Deserialize)]
struct CangjieEntry {
    char: String,
    freq: u32,
}

/// Cangjie input scheme
pub struct CangjieScheme {
    table: HashMap<String, Vec<CangjieEntry>>,
    user_words: HashMap<String, u32>,
}

impl CangjieScheme {
    /// Create a new Cangjie input scheme
    pub fn new() -> Self {
        let table = Self::load_builtin_table();
        Self {
            table,
            user_words: HashMap::new(),
        }
    }

    fn load_builtin_table() -> HashMap<String, Vec<CangjieEntry>> {
        let data = include_str!("../../data/cangjie_table.json");
        match serde_json::from_str::<HashMap<String, Vec<CangjieEntry>>>(data) {
            Ok(t) => t,
            Err(_) => Self::build_minimal_table(),
        }
    }

    fn build_minimal_table() -> HashMap<String, Vec<CangjieEntry>> {
        let mut t = HashMap::new();
        // Cangjie root keys: a=日 b=月 c=金 d=木 e=水 f=火 g=土 h=竹 i=戈 j=十
        // k=大 l=中 m=一 n=弓 o=人 p=心 q=手 r=口 s=尸 t=廿 u=山 v=女 w=田 y=卜
        let entries: Vec<(&str, Vec<(&str, u32)>)> = vec![
            ("a", vec![("日", 999)]),
            ("b", vec![("月", 999)]),
            ("c", vec![("金", 999)]),
            ("d", vec![("木", 999)]),
            ("e", vec![("水", 999)]),
            ("f", vec![("火", 999)]),
            ("g", vec![("土", 999)]),
            ("h", vec![("竹", 999)]),
            ("i", vec![("戈", 999)]),
            ("j", vec![("十", 999)]),
            ("k", vec![("大", 999)]),
            ("l", vec![("中", 999)]),
            ("m", vec![("一", 999)]),
            ("n", vec![("弓", 999)]),
            ("o", vec![("人", 999)]),
            ("p", vec![("心", 999)]),
            ("q", vec![("手", 999)]),
            ("r", vec![("口", 999)]),
            ("s", vec![("尸", 999)]),
            ("t", vec![("廿", 999)]),
            ("u", vec![("山", 999)]),
            ("v", vec![("女", 999)]),
            ("w", vec![("田", 999)]),
            ("y", vec![("卜", 999)]),
            ("ab", vec![("明", 998)]),
            ("am", vec![("旦", 990)]),
            ("km", vec![("天", 995)]),
            ("od", vec![("休", 990)]),
            ("og", vec![("住", 988)]),
            ("oi", vec![("代", 985)]),
            ("oj", vec![("什", 980)]),
            ("om", vec![("任", 978)]),
            ("or", vec![("估", 975)]),
            ("ro", vec![("合", 990)]),
            ("rr", vec![("哥", 985)]),
            ("rv", vec![("如", 988)]),
            ("mi", vec![("我", 999)]),
            ("mr", vec![("可", 995)]),
            ("mp", vec![("必", 990)]),
            ("im", vec![("成", 995)]),
            ("tb", vec![("苗", 970)]),
            ("te", vec![("茶", 980)]),
            ("tg", vec![("苦", 975)]),
            ("dm", vec![("本", 990)]),
            ("wi", vec![("國", 995)]),
            ("wr", vec![("界", 985)]),
            ("wl", vec![("電", 990)]),
            ("jm", vec![("古", 985)]),
            ("jr", vec![("叶", 980)]),
        ];
        for (key, chars) in entries {
            let v: Vec<CangjieEntry> = chars.into_iter()
                .map(|(c, f)| CangjieEntry { char: c.to_string(), freq: f })
                .collect();
            t.insert(key.to_string(), v);
        }
        t
    }

    /// Cangjie code lookup
    pub fn lookup(&self, input: &str) -> Vec<Candidate> {
        let lower = input.to_lowercase();
        let mut results = Vec::new();

        // Exact match
        if let Some(entries) = self.table.get(&lower) {
            for e in entries {
                if let Some(ch) = e.char.chars().next() {
                    results.push(Candidate {
                        character: ch,
                        frequency: e.freq,
                        pronunciation: lower.clone(),
                    });
                }
            }
        }

        // Merge user-learned entries
        crate::user_learning::merge_user_learned(&mut results, "cangjie", &lower);
        results
    }
}

impl Default for CangjieScheme {
    fn default() -> Self {
        Self::new()
    }
}

impl InputScheme for CangjieScheme {
    fn scheme_type(&self) -> SchemeType {
        SchemeType::Cangjie
    }

    fn process_input(&self, input: &str) -> Result<Vec<Candidate>> {
        Ok(self.lookup(input))
    }

    fn get_candidates(&self, input: &str) -> Result<Vec<Candidate>> {
        Ok(self.lookup(input))
    }

    fn add_user_word(&mut self, word: &str, frequency: u32) -> Result<()> {
        self.user_words.insert(word.to_lowercase(), frequency);
        Ok(())
    }

    fn remove_user_word(&mut self, word: &str) -> Result<()> {
        self.user_words.remove(&word.to_lowercase());
        Ok(())
    }
}

// ============================================================
// Quick input scheme -- Cangjie first + last code
// ============================================================

/// Quick input scheme (based on Cangjie first/last code)
pub struct QuickScheme {
    /// Quick table: 2-code -> candidates
    table: HashMap<String, Vec<CangjieEntry>>,
    user_words: HashMap<String, u32>,
}

impl QuickScheme {
    /// Create a new Quick input scheme
    pub fn new() -> Self {
        let table = Self::build_quick_table();
        Self {
            table,
            user_words: HashMap::new(),
        }
    }

    fn build_quick_table() -> HashMap<String, Vec<CangjieEntry>> {
        // Derive Quick table from Cangjie table: take first + last code
        let cangjie_data = include_str!("../../data/cangjie_table.json");
        let cangjie_table: HashMap<String, Vec<CangjieEntry>> =
            serde_json::from_str(cangjie_data).unwrap_or_default();

        let mut quick: HashMap<String, Vec<CangjieEntry>> = HashMap::new();

        for (code, entries) in &cangjie_table {
            let chars: Vec<char> = code.chars().collect();
            let quick_code = if chars.len() <= 1 {
                code.clone()
            } else {
                format!("{}{}", chars[0], chars[chars.len() - 1])
            };
            quick.entry(quick_code)
                .or_default()
                .extend(entries.iter().cloned());
        }

        // Sort candidates for each Quick code
        for entries in quick.values_mut() {
            entries.sort_by(|a, b| b.freq.cmp(&a.freq));
        }

        quick
    }

    fn lookup(&self, input: &str) -> Vec<Candidate> {
        let lower = input.to_lowercase();
        let mut results = Vec::new();

        if let Some(entries) = self.table.get(&lower) {
            for e in entries {
                if let Some(ch) = e.char.chars().next() {
                    results.push(Candidate {
                        character: ch,
                        frequency: e.freq,
                        pronunciation: lower.clone(),
                    });
                }
            }
        }

        // Merge user-learned entries
        crate::user_learning::merge_user_learned(&mut results, "quick", &lower);
        results
    }
}

impl Default for QuickScheme {
    fn default() -> Self {
        Self::new()
    }
}

impl InputScheme for QuickScheme {
    fn scheme_type(&self) -> SchemeType {
        SchemeType::Quick
    }

    fn process_input(&self, input: &str) -> Result<Vec<Candidate>> {
        Ok(self.lookup(input))
    }

    fn get_candidates(&self, input: &str) -> Result<Vec<Candidate>> {
        Ok(self.lookup(input))
    }

    fn add_user_word(&mut self, word: &str, frequency: u32) -> Result<()> {
        self.user_words.insert(word.to_lowercase(), frequency);
        Ok(())
    }

    fn remove_user_word(&mut self, word: &str) -> Result<()> {
        self.user_words.remove(&word.to_lowercase());
        Ok(())
    }
}

// ============================================================
// Wubi 86 input scheme
// ============================================================

/// Wubi table entry
#[derive(Debug, Clone, serde::Deserialize)]
struct WubiEntry {
    char: String,
    freq: u32,
}

/// Wubi input scheme
pub struct WubiScheme {
    table: HashMap<String, Vec<WubiEntry>>,
    user_words: HashMap<String, u32>,
}

impl WubiScheme {
    /// Create a new Wubi input scheme
    pub fn new() -> Self {
        let table = Self::load_builtin_table();
        Self {
            table,
            user_words: HashMap::new(),
        }
    }

    fn load_builtin_table() -> HashMap<String, Vec<WubiEntry>> {
        let data = include_str!("../../data/wubi_table.json");
        match serde_json::from_str::<HashMap<String, Vec<WubiEntry>>>(data) {
            Ok(t) => t,
            Err(_) => Self::build_minimal_table(),
        }
    }

    fn build_minimal_table() -> HashMap<String, Vec<WubiEntry>> {
        let mut t = HashMap::new();
        let entries: Vec<(&str, Vec<(&str, u32)>)> = vec![
            ("g", vec![("一", 999)]),
            ("gg", vec![("一", 999)]),
            ("gh", vec![("下", 990)]),
            ("gi", vec![("不", 998)]),
            ("gj", vec![("理", 980)]),
            ("h", vec![("目", 990)]),
            ("hh", vec![("上", 995)]),
            ("i", vec![("水", 990)]),
            ("ii", vec![("小", 988)]),
            ("j", vec![("日", 985)]),
            ("jj", vec![("早", 980)]),
            ("k", vec![("口", 988)]),
            ("kk", vec![("中", 995)]),
            ("l", vec![("田", 980)]),
            ("ll", vec![("四", 975)]),
            ("d", vec![("大", 995)]),
            ("dd", vec![("太", 988)]),
            ("de", vec![("有", 998)]),
            ("e", vec![("月", 980)]),
            ("f", vec![("土", 975)]),
            ("ff", vec![("地", 990)]),
            ("w", vec![("人", 999)]),
            ("ww", vec![("食", 960)]),
            ("y", vec![("言", 970)]),
            ("r", vec![("白", 960)]),
            ("t", vec![("禾", 955)]),
            ("o", vec![("火", 950)]),
            ("p", vec![("之", 980)]),
            ("n", vec![("已", 960)]),
            ("m", vec![("山", 970)]),
            ("s", vec![("木", 975)]),
            ("a", vec![("工", 965)]),
            ("b", vec![("了", 998)]),
            ("c", vec![("又", 960)]),
            ("q", vec![("金", 970)]),
            ("u", vec![("立", 960)]),
            ("v", vec![("女", 970)]),
            ("x", vec![("幺", 940)]),
        ];
        for (key, chars) in entries {
            let v: Vec<WubiEntry> = chars.into_iter()
                .map(|(c, f)| WubiEntry { char: c.to_string(), freq: f })
                .collect();
            t.insert(key.to_string(), v);
        }
        t
    }

    fn lookup(&self, input: &str) -> Vec<Candidate> {
        let lower = input.to_lowercase();
        let mut results = Vec::new();

        // Exact match
        if let Some(entries) = self.table.get(&lower) {
            for e in entries {
                if let Some(ch) = e.char.chars().next() {
                    results.push(Candidate {
                        character: ch,
                        frequency: e.freq,
                        pronunciation: lower.clone(),
                    });
                }
            }
        }

        // Prefix match (Wubi supports incremental output)
        if results.is_empty() {
            for (key, entries) in &self.table {
                if key.starts_with(&lower) {
                    for e in entries {
                        if let Some(ch) = e.char.chars().next() {
                            results.push(Candidate {
                                character: ch,
                                frequency: e.freq,
                                pronunciation: key.clone(),
                            });
                        }
                    }
                }
            }
        }

        // Merge user-learned entries
        crate::user_learning::merge_user_learned(&mut results, "wubi", &lower);
        results
    }
}

impl Default for WubiScheme {
    fn default() -> Self {
        Self::new()
    }
}

impl InputScheme for WubiScheme {
    fn scheme_type(&self) -> SchemeType {
        SchemeType::Wubi
    }

    fn process_input(&self, input: &str) -> Result<Vec<Candidate>> {
        Ok(self.lookup(input))
    }

    fn get_candidates(&self, input: &str) -> Result<Vec<Candidate>> {
        Ok(self.lookup(input))
    }

    fn add_user_word(&mut self, word: &str, frequency: u32) -> Result<()> {
        self.user_words.insert(word.to_lowercase(), frequency);
        Ok(())
    }

    fn remove_user_word(&mut self, word: &str) -> Result<()> {
        self.user_words.remove(&word.to_lowercase());
        Ok(())
    }
}

// ============================================================
// Handwriting input scheme -- platform hardware, marked unavailable
// ============================================================

/// Handwriting input scheme (requires platform hardware support)
pub struct HandwritingScheme;

impl HandwritingScheme {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HandwritingScheme {
    fn default() -> Self {
        Self::new()
    }
}

impl InputScheme for HandwritingScheme {
    fn scheme_type(&self) -> SchemeType {
        SchemeType::Handwriting
    }

    fn process_input(&self, _input: &str) -> Result<Vec<Candidate>> {
        Err(MaidosError::SchemeError(
            "Handwriting requires platform-specific integration (touchpad/stylus hardware)".to_string()
        ))
    }

    fn get_candidates(&self, _input: &str) -> Result<Vec<Candidate>> {
        Err(MaidosError::SchemeError(
            "Handwriting requires platform-specific integration (touchpad/stylus hardware)".to_string()
        ))
    }

    fn add_user_word(&mut self, _word: &str, _frequency: u32) -> Result<()> {
        Err(MaidosError::SchemeError(
            "Handwriting scheme is not available without platform hardware".to_string()
        ))
    }

    fn remove_user_word(&mut self, _word: &str) -> Result<()> {
        Err(MaidosError::SchemeError(
            "Handwriting scheme is not available without platform hardware".to_string()
        ))
    }
}

// ============================================================
// Voice input scheme -- platform hardware, marked unavailable
// ============================================================

/// Voice input scheme (requires platform hardware support)
pub struct VoiceScheme;

impl VoiceScheme {
    pub fn new() -> Self {
        Self
    }
}

impl Default for VoiceScheme {
    fn default() -> Self {
        Self::new()
    }
}

impl InputScheme for VoiceScheme {
    fn scheme_type(&self) -> SchemeType {
        SchemeType::Voice
    }

    fn process_input(&self, _input: &str) -> Result<Vec<Candidate>> {
        Err(MaidosError::SchemeError(
            "Voice input requires platform-specific integration (microphone hardware)".to_string()
        ))
    }

    fn get_candidates(&self, _input: &str) -> Result<Vec<Candidate>> {
        Err(MaidosError::SchemeError(
            "Voice input requires platform-specific integration (microphone hardware)".to_string()
        ))
    }

    fn add_user_word(&mut self, _word: &str, _frequency: u32) -> Result<()> {
        Err(MaidosError::SchemeError(
            "Voice scheme is not available without platform hardware".to_string()
        ))
    }

    fn remove_user_word(&mut self, _word: &str) -> Result<()> {
        Err(MaidosError::SchemeError(
            "Voice scheme is not available without platform hardware".to_string()
        ))
    }
}

// ============================================================
// Scheme factory
// ============================================================

/// Input scheme factory
pub struct SchemeFactory;

impl SchemeFactory {
    /// Create an input scheme of the specified type
    pub fn create_scheme(scheme_type: &SchemeType, dict_path: Option<&str>) -> Result<Box<dyn InputScheme>> {
        match scheme_type {
            SchemeType::Bopomofo => Ok(Box::new(BopomofoScheme::new())),
            SchemeType::Pinyin => {
                if let Some(path) = dict_path {
                    Ok(Box::new(PinyinScheme::new_from_file(path)?))
                } else {
                    Ok(Box::new(PinyinScheme::new()))
                }
            },
            SchemeType::Cangjie => Ok(Box::new(CangjieScheme::new())),
            SchemeType::Quick => Ok(Box::new(QuickScheme::new())),
            SchemeType::Wubi => Ok(Box::new(WubiScheme::new())),
            SchemeType::Handwriting => Ok(Box::new(HandwritingScheme::new())),
            SchemeType::Voice => Ok(Box::new(VoiceScheme::new())),
            SchemeType::English => Ok(Box::new(crate::english::EnglishScheme::new())),
            SchemeType::Japanese => Ok(Box::new(crate::japanese::JapaneseScheme::new())),
        }
    }

    /// Create an input scheme of the specified type (simplified version)
    pub fn create_scheme_simple(scheme_type: &SchemeType) -> Box<dyn InputScheme> {
        match scheme_type {
            SchemeType::Bopomofo => Box::new(BopomofoScheme::new()),
            SchemeType::Pinyin => Box::new(PinyinScheme::new()),
            SchemeType::Cangjie => Box::new(CangjieScheme::new()),
            SchemeType::Quick => Box::new(QuickScheme::new()),
            SchemeType::Wubi => Box::new(WubiScheme::new()),
            SchemeType::Handwriting => Box::new(HandwritingScheme::new()),
            SchemeType::Voice => Box::new(VoiceScheme::new()),
            SchemeType::English => Box::new(crate::english::EnglishScheme::new()),
            SchemeType::Japanese => Box::new(crate::japanese::JapaneseScheme::new()),
        }
    }

    /// Get all supported input scheme types
    pub fn get_supported_schemes() -> Vec<SchemeType> {
        vec![
            SchemeType::Bopomofo,
            SchemeType::Pinyin,
            SchemeType::Cangjie,
            SchemeType::Quick,
            SchemeType::Wubi,
            SchemeType::Handwriting,
            SchemeType::Voice,
            SchemeType::English,
            SchemeType::Japanese,
        ]
    }

    /// Create a cross-input scheme
    pub fn create_cross_scheme(scheme_type: &SchemeType, _target_charset: &maidos_config::Charset) -> Box<dyn InputScheme> {
        Self::create_scheme_simple(scheme_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheme_factory() {
        let schemes = SchemeFactory::get_supported_schemes();
        assert_eq!(schemes.len(), 9);

        for scheme_type in schemes {
            let scheme = SchemeFactory::create_scheme_simple(&scheme_type);
            assert_eq!(scheme.scheme_type(), scheme_type);
        }
    }

    #[test]
    fn test_bopomofo_scheme_returns_candidates() {
        let scheme = BopomofoScheme::new();
        assert_eq!(scheme.scheme_type(), SchemeType::Bopomofo);

        let candidates = scheme.process_input("ㄅㄚ").unwrap();
        assert!(!candidates.is_empty(), "Bopomofo ㄅㄚ should return candidates");
        // 八 should be among the candidates
        assert!(candidates.iter().any(|c| c.character == '八'));
    }

    #[test]
    fn test_bopomofo_user_word() {
        let mut scheme = BopomofoScheme::new();
        scheme.add_user_word("ㄊㄜㄙㄊ", 999).unwrap();
        scheme.remove_user_word("ㄊㄜㄙㄊ").unwrap();
    }

    #[test]
    fn test_pinyin_scheme_returns_candidates() {
        let scheme = PinyinScheme::new();
        assert_eq!(scheme.scheme_type(), SchemeType::Pinyin);

        // Exact match: "ni" should return 你
        let candidates = scheme.process_input("ni").unwrap();
        assert!(!candidates.is_empty(), "Pinyin 'ni' should return candidates");
        assert!(candidates.iter().any(|c| c.character == '你'), "Pinyin 'ni' should include 你");

        // Exact match: "hao" should return 好
        let candidates = scheme.process_input("hao").unwrap();
        assert!(!candidates.is_empty(), "Pinyin 'hao' should return candidates");
        assert!(candidates.iter().any(|c| c.character == '好'), "Pinyin 'hao' should include 好");

        // Exact match: "shi" should return 是
        let candidates = scheme.process_input("shi").unwrap();
        assert!(!candidates.is_empty(), "Pinyin 'shi' should return candidates");
        assert!(candidates.iter().any(|c| c.character == '是'), "Pinyin 'shi' should include 是");
    }

    #[test]
    fn test_pinyin_user_word() {
        let mut scheme = PinyinScheme::new();
        scheme.add_user_word("test", 999).unwrap();
        scheme.remove_user_word("test").unwrap();
    }

    #[test]
    fn test_cangjie_scheme_returns_candidates() {
        let scheme = CangjieScheme::new();
        assert_eq!(scheme.scheme_type(), SchemeType::Cangjie);

        let candidates = scheme.process_input("a").unwrap();
        assert!(!candidates.is_empty(), "Cangjie 'a' should return 日");
        assert!(candidates.iter().any(|c| c.character == '日'));
    }

    #[test]
    fn test_cangjie_multi_code() {
        let scheme = CangjieScheme::new();
        let candidates = scheme.process_input("ab").unwrap();
        assert!(!candidates.is_empty(), "Cangjie 'ab' should return 明");
        assert!(candidates.iter().any(|c| c.character == '明'));
    }

    #[test]
    fn test_quick_scheme_returns_candidates() {
        let scheme = QuickScheme::new();
        assert_eq!(scheme.scheme_type(), SchemeType::Quick);

        // Quick = Cangjie first+last code, single keys should work
        let candidates = scheme.process_input("a").unwrap();
        assert!(!candidates.is_empty(), "Quick 'a' should return candidates");
    }

    #[test]
    fn test_wubi_scheme_returns_candidates() {
        let scheme = WubiScheme::new();
        assert_eq!(scheme.scheme_type(), SchemeType::Wubi);

        let candidates = scheme.process_input("g").unwrap();
        assert!(!candidates.is_empty(), "Wubi 'g' should return 一");
        assert!(candidates.iter().any(|c| c.character == '一'));
    }

    #[test]
    fn test_handwriting_returns_error() {
        let scheme = HandwritingScheme::new();
        assert_eq!(scheme.scheme_type(), SchemeType::Handwriting);

        let result = scheme.process_input("test");
        assert!(result.is_err(), "Handwriting should return platform error");
    }

    #[test]
    fn test_voice_returns_error() {
        let scheme = VoiceScheme::new();
        assert_eq!(scheme.scheme_type(), SchemeType::Voice);

        let result = scheme.process_input("test");
        assert!(result.is_err(), "Voice should return platform error");
    }

    #[test]
    fn test_english_scheme_returns_candidates() {
        let scheme = crate::english::EnglishScheme::new();
        assert_eq!(scheme.scheme_type(), SchemeType::English);

        let candidates = scheme.process_input("th").unwrap();
        assert!(!candidates.is_empty(), "English 'th' should return candidates like 'the'");
    }

    #[test]
    fn test_japanese_scheme_returns_candidates() {
        let scheme = crate::japanese::JapaneseScheme::new();
        assert_eq!(scheme.scheme_type(), SchemeType::Japanese);

        let candidates = scheme.process_input("あい").unwrap();
        assert!(!candidates.is_empty(), "Japanese あい should return kanji candidates");
    }
}
