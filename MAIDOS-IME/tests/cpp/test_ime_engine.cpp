#include <gtest/gtest.h>
#include "ime_engine.h"
#include "dictionary.h"
#include "pinyin_parser.h"

// ImeEngine 測試
TEST(ImeEngineTest, Constructor) {
    ImeEngine engine;
    EXPECT_TRUE(true); // Basic construction test
}

TEST(ImeEngineTest, Initialize) {
    ImeEngine engine;
    EXPECT_TRUE(engine.Initialize("src/config/maidos.toml"));
}

// Dictionary 測試
TEST(DictionaryTest, LoadDictionary) {
    Dictionary dict;
    EXPECT_TRUE(dict.Load("src/dicts/pinyin.dict.json"));
}

TEST(DictionaryTest, QueryCharacters) {
    Dictionary dict;
    dict.Load("src/dicts/pinyin.dict.json");
    auto candidates = dict.Query("ni");
    EXPECT_GT(candidates.size(), 0);
}

// PinyinParser 測試
TEST(PinyinParserTest, BasicParsing) {
    PinyinParser parser;
    auto result = parser.Parse("nihao");
    EXPECT_FALSE(result.empty());
}

TEST(PinyinParserTest, InvalidPinyin) {
    PinyinParser parser;
    auto result = parser.Parse("xyz");
    EXPECT_TRUE(result.empty());
}

int main(int argc, char** argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}
