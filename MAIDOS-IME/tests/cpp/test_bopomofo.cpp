#include <gtest/gtest.h>
#include "../../src/MAIDOS.IME.Core/bopomofo_scheme.h"

class BopomofoSchemeTest : public ::testing::Test {
protected:
    BopomofoScheme scheme;

    void SetUp() override {
        // Initialize with a mock or test dictionary if needed
        scheme.Initialize();
    }
};

TEST_F(BopomofoSchemeTest, BasicMapping) {
    // 測試單個注音符號轉換
    // 預期 ㄅ -> b, ㄚ -> a
    // 組合 ㄅㄚ -> ba
    std::vector<InputScheme::Candidate> candidates = scheme.GetCandidates(L"ㄅㄚ");
    
    // 如果字典裡有 "吧" 或 "八"，頻率應該大於 0
    bool found_ba = false;
    for (const auto& cand : candidates) {
        if (cand.character == L'八' || cand.character == L'吧') {
            found_ba = true;
            break;
        }
    }
    // 目前的實作可能無法正確組合注音，此測試預期會失敗或返回空
    EXPECT_TRUE(found_ba) << "[MAIDOS-AUDIT] Failed to find candidate for Bopomofo 'ㄅㄚ'";
}

TEST_F(BopomofoSchemeTest, ToneMapping) {
    // 測試聲調處理
    // 預期 ㄇㄚˇ -> ma3
    std::vector<InputScheme::Candidate> candidates = scheme.GetCandidates(L"ㄇㄚˇ");
    EXPECT_FALSE(candidates.empty());
}