//! Japanese input scheme
//!
//! Romaji -> Kana -> Kanji three-stage conversion

use std::collections::HashMap;
use crate::Result;
use crate::schemes::{Candidate, InputScheme, SchemeType};

/// Kana -> Kanji mapping entry
#[derive(Debug, Clone, serde::Deserialize)]
struct KanjiEntry {
    kanji: String,
    freq: u32,
}

/// Japanese input scheme
pub struct JapaneseScheme {
    /// Romaji -> Kana conversion table
    romaji_table: HashMap<String, String>,
    /// Kana -> Kanji mapping table
    kana_to_kanji: HashMap<String, Vec<KanjiEntry>>,
    /// User words
    user_words: HashMap<String, u32>,
}

impl JapaneseScheme {
    pub fn new() -> Self {
        let romaji_table = Self::build_romaji_table();
        let kana_to_kanji = Self::load_kana_to_kanji();
        Self {
            romaji_table,
            kana_to_kanji,
            user_words: HashMap::new(),
        }
    }

    /// Romaji -> Hiragana conversion rules (~150 entries)
    fn build_romaji_table() -> HashMap<String, String> {
        let mut t = HashMap::new();

        // Vowels
        t.insert("a".into(), "\u{3042}".into());
        t.insert("i".into(), "\u{3044}".into());
        t.insert("u".into(), "\u{3046}".into());
        t.insert("e".into(), "\u{3048}".into());
        t.insert("o".into(), "\u{304A}".into());

        // Ka-row
        t.insert("ka".into(), "\u{304B}".into());
        t.insert("ki".into(), "\u{304D}".into());
        t.insert("ku".into(), "\u{304F}".into());
        t.insert("ke".into(), "\u{3051}".into());
        t.insert("ko".into(), "\u{3053}".into());

        // Sa-row
        t.insert("sa".into(), "\u{3055}".into());
        t.insert("si".into(), "\u{3057}".into());
        t.insert("shi".into(), "\u{3057}".into());
        t.insert("su".into(), "\u{3059}".into());
        t.insert("se".into(), "\u{305B}".into());
        t.insert("so".into(), "\u{305D}".into());

        // Ta-row
        t.insert("ta".into(), "\u{305F}".into());
        t.insert("ti".into(), "\u{3061}".into());
        t.insert("chi".into(), "\u{3061}".into());
        t.insert("tu".into(), "\u{3064}".into());
        t.insert("tsu".into(), "\u{3064}".into());
        t.insert("te".into(), "\u{3066}".into());
        t.insert("to".into(), "\u{3068}".into());

        // Na-row
        t.insert("na".into(), "\u{306A}".into());
        t.insert("ni".into(), "\u{306B}".into());
        t.insert("nu".into(), "\u{306C}".into());
        t.insert("ne".into(), "\u{306D}".into());
        t.insert("no".into(), "\u{306E}".into());

        // Ha-row
        t.insert("ha".into(), "\u{306F}".into());
        t.insert("hi".into(), "\u{3072}".into());
        t.insert("hu".into(), "\u{3075}".into());
        t.insert("fu".into(), "\u{3075}".into());
        t.insert("he".into(), "\u{3078}".into());
        t.insert("ho".into(), "\u{307B}".into());

        // Ma-row
        t.insert("ma".into(), "\u{307E}".into());
        t.insert("mi".into(), "\u{307F}".into());
        t.insert("mu".into(), "\u{3080}".into());
        t.insert("me".into(), "\u{3081}".into());
        t.insert("mo".into(), "\u{3082}".into());

        // Ya-row
        t.insert("ya".into(), "\u{3084}".into());
        t.insert("yu".into(), "\u{3086}".into());
        t.insert("yo".into(), "\u{3088}".into());

        // Ra-row
        t.insert("ra".into(), "\u{3089}".into());
        t.insert("ri".into(), "\u{308A}".into());
        t.insert("ru".into(), "\u{308B}".into());
        t.insert("re".into(), "\u{308C}".into());
        t.insert("ro".into(), "\u{308D}".into());

        // Wa-row
        t.insert("wa".into(), "\u{308F}".into());
        t.insert("wi".into(), "\u{3090}".into());
        t.insert("we".into(), "\u{3091}".into());
        t.insert("wo".into(), "\u{3092}".into());

        // N — ambiguity handling: nn -> N, n + non-vowel -> N
        t.insert("nn".into(), "\u{3093}".into());
        t.insert("n'".into(), "\u{3093}".into());

        // Voiced consonants: Ga-row
        t.insert("ga".into(), "\u{304C}".into());
        t.insert("gi".into(), "\u{304E}".into());
        t.insert("gu".into(), "\u{3050}".into());
        t.insert("ge".into(), "\u{3052}".into());
        t.insert("go".into(), "\u{3054}".into());

        // Za-row
        t.insert("za".into(), "\u{3056}".into());
        t.insert("zi".into(), "\u{3058}".into());
        t.insert("ji".into(), "\u{3058}".into());
        t.insert("zu".into(), "\u{305A}".into());
        t.insert("ze".into(), "\u{305C}".into());
        t.insert("zo".into(), "\u{305E}".into());

        // Da-row
        t.insert("da".into(), "\u{3060}".into());
        t.insert("di".into(), "\u{3062}".into());
        t.insert("du".into(), "\u{3065}".into());
        t.insert("de".into(), "\u{3067}".into());
        t.insert("do".into(), "\u{3069}".into());

        // Ba-row
        t.insert("ba".into(), "\u{3070}".into());
        t.insert("bi".into(), "\u{3073}".into());
        t.insert("bu".into(), "\u{3076}".into());
        t.insert("be".into(), "\u{3079}".into());
        t.insert("bo".into(), "\u{307C}".into());

        // Pa-row
        t.insert("pa".into(), "\u{3071}".into());
        t.insert("pi".into(), "\u{3074}".into());
        t.insert("pu".into(), "\u{3077}".into());
        t.insert("pe".into(), "\u{307A}".into());
        t.insert("po".into(), "\u{307D}".into());

        // Contracted sounds: Kya-row
        t.insert("kya".into(), "\u{304D}\u{3083}".into());
        t.insert("kyu".into(), "\u{304D}\u{3085}".into());
        t.insert("kyo".into(), "\u{304D}\u{3087}".into());

        // Sha-row
        t.insert("sha".into(), "\u{3057}\u{3083}".into());
        t.insert("shu".into(), "\u{3057}\u{3085}".into());
        t.insert("sho".into(), "\u{3057}\u{3087}".into());
        t.insert("sya".into(), "\u{3057}\u{3083}".into());
        t.insert("syu".into(), "\u{3057}\u{3085}".into());
        t.insert("syo".into(), "\u{3057}\u{3087}".into());

        // Cha-row
        t.insert("cha".into(), "\u{3061}\u{3083}".into());
        t.insert("chu".into(), "\u{3061}\u{3085}".into());
        t.insert("cho".into(), "\u{3061}\u{3087}".into());
        t.insert("tya".into(), "\u{3061}\u{3083}".into());
        t.insert("tyu".into(), "\u{3061}\u{3085}".into());
        t.insert("tyo".into(), "\u{3061}\u{3087}".into());

        // Nya-row
        t.insert("nya".into(), "\u{306B}\u{3083}".into());
        t.insert("nyu".into(), "\u{306B}\u{3085}".into());
        t.insert("nyo".into(), "\u{306B}\u{3087}".into());

        // Hya-row
        t.insert("hya".into(), "\u{3072}\u{3083}".into());
        t.insert("hyu".into(), "\u{3072}\u{3085}".into());
        t.insert("hyo".into(), "\u{3072}\u{3087}".into());

        // Mya-row
        t.insert("mya".into(), "\u{307F}\u{3083}".into());
        t.insert("myu".into(), "\u{307F}\u{3085}".into());
        t.insert("myo".into(), "\u{307F}\u{3087}".into());

        // Rya-row
        t.insert("rya".into(), "\u{308A}\u{3083}".into());
        t.insert("ryu".into(), "\u{308A}\u{3085}".into());
        t.insert("ryo".into(), "\u{308A}\u{3087}".into());

        // Gya-row
        t.insert("gya".into(), "\u{304E}\u{3083}".into());
        t.insert("gyu".into(), "\u{304E}\u{3085}".into());
        t.insert("gyo".into(), "\u{304E}\u{3087}".into());

        // Ja-row
        t.insert("ja".into(), "\u{3058}\u{3083}".into());
        t.insert("ju".into(), "\u{3058}\u{3085}".into());
        t.insert("jo".into(), "\u{3058}\u{3087}".into());
        t.insert("jya".into(), "\u{3058}\u{3083}".into());
        t.insert("jyu".into(), "\u{3058}\u{3085}".into());
        t.insert("jyo".into(), "\u{3058}\u{3087}".into());
        t.insert("zya".into(), "\u{3058}\u{3083}".into());
        t.insert("zyu".into(), "\u{3058}\u{3085}".into());
        t.insert("zyo".into(), "\u{3058}\u{3087}".into());

        // Bya-row
        t.insert("bya".into(), "\u{3073}\u{3083}".into());
        t.insert("byu".into(), "\u{3073}\u{3085}".into());
        t.insert("byo".into(), "\u{3073}\u{3087}".into());

        // Pya-row
        t.insert("pya".into(), "\u{3074}\u{3083}".into());
        t.insert("pyu".into(), "\u{3074}\u{3085}".into());
        t.insert("pyo".into(), "\u{3074}\u{3087}".into());

        // Geminate consonant (sokuon) -- double consonant (e.g., kk -> small-tsu + k, tt -> small-tsu + t)
        // Handled specially in romaji_to_hiragana()

        // Long vowel mark
        t.insert("-".into(), "\u{30FC}".into());

        t
    }

    fn load_kana_to_kanji() -> HashMap<String, Vec<KanjiEntry>> {
        let data = include_str!("../../data/japanese_kana2kanji.json");
        match serde_json::from_str::<HashMap<String, Vec<KanjiEntry>>>(data) {
            Ok(t) => t,
            Err(_) => Self::build_minimal_kana_table(),
        }
    }

    fn build_minimal_kana_table() -> HashMap<String, Vec<KanjiEntry>> {
        let mut t = HashMap::new();
        let entries: Vec<(&str, Vec<(&str, u32)>)> = vec![
            ("\u{3042}\u{3044}", vec![("\u{611B}", 950), ("\u{4F1A}\u{3044}", 900), ("\u{5408}\u{3044}", 850)]),
            ("\u{3042}\u{3046}", vec![("\u{4F1A}\u{3046}", 950), ("\u{5408}\u{3046}", 900)]),
            ("\u{3042}\u{304B}", vec![("\u{8D64}", 950)]),
            ("\u{3042}\u{3055}", vec![("\u{671D}", 950), ("\u{9EBB}", 800)]),
            ("\u{3042}\u{3057}", vec![("\u{8DB3}", 950), ("\u{811A}", 900)]),
            ("\u{3042}\u{3081}", vec![("\u{96E8}", 950), ("\u{98F4}", 800)]),
            ("\u{3044}\u{3048}", vec![("\u{5BB6}", 960)]),
            ("\u{3044}\u{304F}", vec![("\u{884C}\u{304F}", 960)]),
            ("\u{3044}\u{307E}", vec![("\u{4ECA}", 970)]),
            ("\u{3044}\u{307F}", vec![("\u{610F}\u{5473}", 960)]),
            ("\u{3044}\u{308D}", vec![("\u{8272}", 960)]),
            ("\u{3046}\u{3048}", vec![("\u{4E0A}", 970)]),
            ("\u{3046}\u{307F}", vec![("\u{6D77}", 960)]),
            ("\u{3048}\u{304D}", vec![("\u{99C5}", 960)]),
            ("\u{304A}\u{3068}", vec![("\u{97F3}", 960)]),
            ("\u{304A}\u{3093}\u{306A}", vec![("\u{5973}", 960)]),
            ("\u{304A}\u{3068}\u{3053}", vec![("\u{7537}", 960)]),
            ("\u{304B}", vec![("\u{86CA}", 900), ("\u{79D1}", 890), ("\u{8AB2}", 880)]),
            ("\u{304B}\u{304A}", vec![("\u{9854}", 960)]),
            ("\u{304B}\u{305C}", vec![("\u{98A8}", 950), ("\u{98A8}\u{90AA}", 900)]),
            ("\u{304B}\u{306D}", vec![("\u{91D1}", 960), ("\u{9418}", 900)]),
            ("\u{304B}\u{307F}", vec![("\u{7D19}", 960), ("\u{9AEA}", 950), ("\u{795E}", 940)]),
            ("\u{304B}\u{3089}\u{3060}", vec![("\u{4F53}", 960)]),
            ("\u{304D}", vec![("\u{6728}", 960), ("\u{6C17}", 950)]),
            ("\u{304D}\u{304F}", vec![("\u{805E}\u{304F}", 960), ("\u{83CA}", 900)]),
            ("\u{304D}\u{305F}", vec![("\u{5317}", 960)]),
            ("\u{304F}\u{3061}", vec![("\u{53E3}", 960)]),
            ("\u{304F}\u{306B}", vec![("\u{56FD}", 970)]),
            ("\u{304F}\u{3082}", vec![("\u{96F2}", 950), ("\u{8718}\u{86DB}", 800)]),
            ("\u{304F}\u{308B}", vec![("\u{6765}\u{308B}", 970)]),
            ("\u{304F}\u{308B}\u{307E}", vec![("\u{8ECA}", 970)]),
            ("\u{3053}\u{3048}", vec![("\u{58F0}", 960)]),
            ("\u{3053}\u{3053}", vec![("\u{6B64}\u{51E6}", 950)]),
            ("\u{3053}\u{3053}\u{308D}", vec![("\u{5FC3}", 970)]),
            ("\u{3053}\u{3068}", vec![("\u{4E8B}", 980)]),
            ("\u{3053}\u{3068}\u{3070}", vec![("\u{8A00}\u{8449}", 970)]),
            ("\u{3053}\u{3069}\u{3082}", vec![("\u{5B50}\u{4F9B}", 970)]),
            ("\u{3055}\u{304F}\u{3089}", vec![("\u{685C}", 960)]),
            ("\u{3057}", vec![("\u{56DB}", 950), ("\u{6B7B}", 940)]),
            ("\u{3057}\u{3054}\u{3068}", vec![("\u{4ED5}\u{4E8B}", 980)]),
            ("\u{3057}\u{305F}", vec![("\u{4E0B}", 960), ("\u{820C}", 900)]),
            ("\u{3057}\u{307E}", vec![("\u{5CF6}", 950)]),
            ("\u{3057}\u{308B}", vec![("\u{77E5}\u{308B}", 960)]),
            ("\u{3058}\u{304B}\u{3093}", vec![("\u{6642}\u{9593}", 980)]),
            ("\u{3058}\u{3076}\u{3093}", vec![("\u{81EA}\u{5206}", 970)]),
            ("\u{3059}\u{304D}", vec![("\u{597D}\u{304D}", 970)]),
            ("\u{305B}\u{304B}\u{3044}", vec![("\u{4E16}\u{754C}", 980)]),
            ("\u{305B}\u{3093}\u{305B}\u{3044}", vec![("\u{5148}\u{751F}", 970)]),
            ("\u{305D}\u{3068}", vec![("\u{5916}", 950)]),
            ("\u{305D}\u{3089}", vec![("\u{7A7A}", 960)]),
            ("\u{305F}", vec![("\u{7530}", 900), ("\u{4ED6}", 890)]),
            ("\u{305F}\u{3079}\u{308B}", vec![("\u{98DF}\u{3079}\u{308B}", 970)]),
            ("\u{3061}", vec![("\u{8840}", 940), ("\u{5730}", 930), ("\u{77E5}", 920)]),
            ("\u{3061}\u{304B}\u{3089}", vec![("\u{529B}", 960)]),
            ("\u{3064}\u{304D}", vec![("\u{6708}", 970)]),
            ("\u{3066}", vec![("\u{624B}", 970)]),
            ("\u{3066}\u{3093}\u{304D}", vec![("\u{5929}\u{6C17}", 960)]),
            ("\u{3067}\u{3093}\u{308F}", vec![("\u{96FB}\u{8A71}", 970)]),
            ("\u{3068}", vec![("\u{6238}", 900), ("\u{90FD}", 890)]),
            ("\u{3068}\u{3082}\u{3060}\u{3061}", vec![("\u{53CB}\u{9054}", 970)]),
            ("\u{3068}\u{308A}", vec![("\u{9CE5}", 960)]),
            ("\u{306A}", vec![("\u{540D}", 930), ("\u{5948}", 900)]),
            ("\u{306A}\u{304B}", vec![("\u{4E2D}", 970)]),
            ("\u{306A}\u{3064}", vec![("\u{590F}", 960)]),
            ("\u{306A}\u{307E}\u{3048}", vec![("\u{540D}\u{524D}", 970)]),
            ("\u{306B}\u{3057}", vec![("\u{897F}", 950)]),
            ("\u{306B}\u{308F}", vec![("\u{5EAD}", 950)]),
            ("\u{306D}\u{3053}", vec![("\u{732B}", 960)]),
            ("\u{306F}\u{306A}", vec![("\u{82B1}", 960), ("\u{9F3B}", 950)]),
            ("\u{306F}\u{306F}", vec![("\u{6BCD}", 960)]),
            ("\u{306F}\u{308B}", vec![("\u{6625}", 960)]),
            ("\u{3072}\u{304B}\u{308A}", vec![("\u{5149}", 960)]),
            ("\u{3072}\u{3068}", vec![("\u{4EBA}", 990)]),
            ("\u{3075}\u{3086}", vec![("\u{51AC}", 960)]),
            ("\u{307B}\u{3057}", vec![("\u{661F}", 960)]),
            ("\u{307B}\u{3093}", vec![("\u{672C}", 980)]),
            ("\u{307E}\u{3048}", vec![("\u{524D}", 970)]),
            ("\u{307E}\u{3061}", vec![("\u{753A}", 960), ("\u{8857}", 950)]),
            ("\u{307E}\u{3069}", vec![("\u{7A93}", 960)]),
            ("\u{307F}", vec![("\u{5B9F}", 930), ("\u{8EAB}", 920), ("\u{5473}", 910)]),
            ("\u{307F}\u{304E}", vec![("\u{53F3}", 960)]),
            ("\u{307F}\u{305A}", vec![("\u{6C34}", 980)]),
            ("\u{307F}\u{305B}", vec![("\u{5E97}", 960)]),
            ("\u{307F}\u{3061}", vec![("\u{9053}", 970)]),
            ("\u{307F}\u{306A}\u{307F}", vec![("\u{5357}", 960)]),
            ("\u{307F}\u{307F}", vec![("\u{8033}", 960)]),
            ("\u{307F}\u{308B}", vec![("\u{898B}\u{308B}", 970)]),
            ("\u{3080}\u{3059}\u{3081}", vec![("\u{5A18}", 960)]),
            ("\u{3081}", vec![("\u{76EE}", 970), ("\u{82BD}", 900)]),
            ("\u{3082}\u{306E}", vec![("\u{7269}", 970), ("\u{8005}", 960)]),
            ("\u{3082}\u{308A}", vec![("\u{68EE}", 960)]),
            ("\u{3084}\u{307E}", vec![("\u{5C71}", 980)]),
            ("\u{3086}\u{304D}", vec![("\u{96EA}", 960)]),
            ("\u{3088}\u{308B}", vec![("\u{591C}", 970)]),
            ("\u{308F}\u{305F}\u{3057}", vec![("\u{79C1}", 990)]),
        ];
        for (key, vals) in entries {
            let v: Vec<KanjiEntry> = vals.into_iter()
                .map(|(k, f)| KanjiEntry { kanji: k.to_string(), freq: f })
                .collect();
            t.insert(key.to_string(), v);
        }
        t
    }

    /// Romaji -> Hiragana conversion
    pub fn romaji_to_hiragana(&self, romaji: &str) -> String {
        let input = romaji.to_lowercase();
        let chars: Vec<char> = input.chars().collect();
        let mut result = String::new();
        let mut i = 0;

        while i < chars.len() {
            // Geminate consonant (sokuon): double consonant
            if i + 1 < chars.len()
                && chars[i] == chars[i + 1]
                && !"aiueon".contains(chars[i])
                && chars[i].is_ascii_alphabetic()
            {
                result.push('\u{3063}');
                i += 1;
                continue;
            }

            // N handling: n + non-vowel and non-y
            if chars[i] == 'n' && i + 1 < chars.len() {
                let next = chars[i + 1];
                if !"aiueoy".contains(next) && next != 'n' {
                    result.push('\u{3093}');
                    i += 1;
                    continue;
                }
            }
            // Trailing n at end of input
            if chars[i] == 'n' && i + 1 == chars.len() {
                // Standalone n at end can be N (or incomplete input, keep as-is)
                result.push('\u{3093}');
                i += 1;
                continue;
            }

            // Try 3-character match
            let mut matched = false;
            if i + 3 <= chars.len() {
                let key: String = chars[i..i + 3].iter().collect();
                if let Some(kana) = self.romaji_table.get(&key) {
                    result.push_str(kana);
                    i += 3;
                    matched = true;
                }
            }

            // Try 2-character match
            if !matched && i + 2 <= chars.len() {
                let key: String = chars[i..i + 2].iter().collect();
                if let Some(kana) = self.romaji_table.get(&key) {
                    result.push_str(kana);
                    i += 2;
                    matched = true;
                }
            }

            // Try 1-character match
            if !matched {
                let key: String = chars[i..i + 1].iter().collect();
                if let Some(kana) = self.romaji_table.get(&key) {
                    result.push_str(kana);
                } else {
                    // Cannot convert; keep original character
                    result.push(chars[i]);
                }
                i += 1;
            }
        }

        result
    }

    /// Hiragana -> Katakana (Unicode offset +0x60)
    pub fn hiragana_to_katakana(hiragana: &str) -> String {
        hiragana.chars().map(|c| {
            let code = c as u32;
            // Hiragana range: U+3041 - U+3096
            // Katakana range: U+30A1 - U+30F6
            if (0x3041..=0x3096).contains(&code) {
                char::from_u32(code + 0x60).unwrap_or(c)
            } else {
                c
            }
        }).collect()
    }

    fn lookup(&self, input: &str) -> Vec<Candidate> {
        let mut results = Vec::new();

        // If input is romaji, convert first
        let kana = if input.chars().all(|c| c.is_ascii_alphabetic() || c == '\'') {
            self.romaji_to_hiragana(input)
        } else {
            input.to_string()
        };

        // Look up kana -> kanji
        if let Some(entries) = self.kana_to_kanji.get(&kana) {
            for e in entries {
                if let Some(ch) = e.kanji.chars().next() {
                    results.push(Candidate {
                        character: ch,
                        frequency: e.freq,
                        pronunciation: kana.clone(),
                    });
                }
            }
        }

        // If no kanji match, return kana itself
        if results.is_empty() && !kana.is_empty() {
            if let Some(ch) = kana.chars().next() {
                results.push(Candidate {
                    character: ch,
                    frequency: 500,
                    pronunciation: kana.clone(),
                });
            }
            // Also provide katakana option
            let katakana = Self::hiragana_to_katakana(&kana);
            if katakana != kana {
                if let Some(ch) = katakana.chars().next() {
                    results.push(Candidate {
                        character: ch,
                        frequency: 400,
                        pronunciation: katakana,
                    });
                }
            }
        }

        // Prefix match
        if results.is_empty() {
            for (key, entries) in &self.kana_to_kanji {
                if key.starts_with(&kana) {
                    for e in entries {
                        if let Some(ch) = e.kanji.chars().next() {
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

        // User words
        for (word, &freq) in &self.user_words {
            if word.starts_with(&kana) || kana.starts_with(word.as_str()) {
                if let Some(ch) = word.chars().next() {
                    results.push(Candidate {
                        character: ch,
                        frequency: freq,
                        pronunciation: word.clone(),
                    });
                }
            }
        }

        results.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        results
    }
}

impl Default for JapaneseScheme {
    fn default() -> Self {
        Self::new()
    }
}

impl InputScheme for JapaneseScheme {
    fn scheme_type(&self) -> SchemeType {
        SchemeType::Japanese
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_romaji_to_hiragana_basic() {
        let scheme = JapaneseScheme::new();
        assert_eq!(scheme.romaji_to_hiragana("ka"), "\u{304B}");
        assert_eq!(scheme.romaji_to_hiragana("ki"), "\u{304D}");
        assert_eq!(scheme.romaji_to_hiragana("ku"), "\u{304F}");
        assert_eq!(scheme.romaji_to_hiragana("ke"), "\u{3051}");
        assert_eq!(scheme.romaji_to_hiragana("ko"), "\u{3053}");
    }

    #[test]
    fn test_romaji_to_hiragana_compound() {
        let scheme = JapaneseScheme::new();
        assert_eq!(scheme.romaji_to_hiragana("sha"), "\u{3057}\u{3083}");
        assert_eq!(scheme.romaji_to_hiragana("chi"), "\u{3061}");
        assert_eq!(scheme.romaji_to_hiragana("tsu"), "\u{3064}");
    }

    #[test]
    fn test_romaji_sokuon() {
        let scheme = JapaneseScheme::new();
        // Geminate consonant kk -> small-tsu + k
        let result = scheme.romaji_to_hiragana("kka");
        assert!(result.contains('\u{3063}'));
    }

    #[test]
    fn test_romaji_n_handling() {
        let scheme = JapaneseScheme::new();
        // n before consonant -> N
        let result = scheme.romaji_to_hiragana("kanka");
        assert!(result.contains('\u{3093}'));
    }

    #[test]
    fn test_hiragana_to_katakana() {
        assert_eq!(JapaneseScheme::hiragana_to_katakana("\u{3042}\u{3044}\u{3046}\u{3048}\u{304A}"), "\u{30A2}\u{30A4}\u{30A6}\u{30A8}\u{30AA}");
        assert_eq!(JapaneseScheme::hiragana_to_katakana("\u{304B}\u{304D}\u{304F}\u{3051}\u{3053}"), "\u{30AB}\u{30AD}\u{30AF}\u{30B1}\u{30B3}");
    }

    #[test]
    fn test_kana_to_kanji_lookup() {
        let scheme = JapaneseScheme::new();
        let candidates = scheme.process_input("\u{3042}\u{3044}").unwrap();
        assert!(!candidates.is_empty(), "\u{3042}\u{3044} should return kanji candidates like \u{611B}");
    }

    #[test]
    fn test_romaji_to_kanji() {
        let scheme = JapaneseScheme::new();
        // "ai" -> "\u{3042}\u{3044}" -> \u{611B}
        let candidates = scheme.process_input("ai").unwrap();
        assert!(!candidates.is_empty(), "romaji 'ai' should convert to \u{3042}\u{3044} and find kanji");
    }

    #[test]
    fn test_japanese_user_word() {
        let mut scheme = JapaneseScheme::new();
        scheme.add_user_word("\u{30C6}\u{30B9}\u{30C8}", 999).unwrap();
        scheme.remove_user_word("\u{30C6}\u{30B9}\u{30C8}").unwrap();
    }
}
