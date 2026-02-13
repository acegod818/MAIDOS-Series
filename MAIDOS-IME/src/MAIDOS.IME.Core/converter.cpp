#include "pch.h"
#include "converter.h"
#include <algorithm>

// Constructor
CharsetConverter::CharsetConverter()
{
    InitializeMaps();
}

// Destructor
CharsetConverter::~CharsetConverter()
{
}

// Convert text
std::wstring CharsetConverter::Convert(const std::wstring& text, const std::wstring& from, const std::wstring& to)
{
    if (from == to)
    {
        return text;
    }

    std::wstring result = text;
    
    if (from == L"Simplified" && to == L"Traditional")
    {
        for (size_t i = 0; i < result.length(); ++i)
        {
            auto it = m_s2t.find(result[i]);
            if (it != m_s2t.end())
            {
                result[i] = it->second;
            }
        }
    }
    else if (from == L"Traditional" && to == L"Simplified")
    {
        for (size_t i = 0; i < result.length(); ++i)
        {
            auto it = m_t2s.find(result[i]);
            if (it != m_t2s.end())
            {
                result[i] = it->second;
            }
        }
    }

    return result;
}

// Convert candidates
std::vector<wchar_t> CharsetConverter::ConvertCandidates(const std::vector<wchar_t>& candidates, Charset from, Charset to)
{
    if (from == to)
    {
        return candidates;
    }

    std::vector<wchar_t> result = candidates;
    
    if (from == Charset::Simplified && to == Charset::Traditional)
    {
        for (size_t i = 0; i < result.size(); ++i)
        {
            auto it = m_s2t.find(result[i]);
            if (it != m_s2t.end())
            {
                result[i] = it->second;
            }
        }
    }
    else if (from == Charset::Traditional && to == Charset::Simplified)
    {
        for (size_t i = 0; i < result.size(); ++i)
        {
            auto it = m_t2s.find(result[i]);
            if (it != m_t2s.end())
            {
                result[i] = it->second;
            }
        }
    }

    return result;
}

// Initialize maps
void CharsetConverter::InitializeMaps()
{
    // Real Simplified↔Traditional Chinese character mapping
    // 378 character pairs from Unicode Unihan database

    m_s2t[L'\x8FD9'] = L'\x9019'; // 这→這
    m_s2t[L'\x4E3A'] = L'\x70BA'; // 为→為
    m_s2t[L'\x6765'] = L'\x4F86'; // 来→來
    m_s2t[L'\x4E2A'] = L'\x500B'; // 个→個
    m_s2t[L'\x4EEC'] = L'\x5011'; // 们→們
    m_s2t[L'\x8BF4'] = L'\x8AAA'; // 说→說
    m_s2t[L'\x56FD'] = L'\x570B'; // 国→國
    m_s2t[L'\x65F6'] = L'\x6642'; // 时→時
    m_s2t[L'\x4F1A'] = L'\x6703'; // 会→會
    m_s2t[L'\x5BF9'] = L'\x5C0D'; // 对→對
    m_s2t[L'\x91CC'] = L'\x88E1'; // 里→裡
    m_s2t[L'\x8FD8'] = L'\x9084'; // 还→還
    m_s2t[L'\x6CA1'] = L'\x6C92'; // 没→沒
    m_s2t[L'\x540E'] = L'\x5F8C'; // 后→後
    m_s2t[L'\x8FC7'] = L'\x904E'; // 过→過
    m_s2t[L'\x5B66'] = L'\x5B78'; // 学→學
    m_s2t[L'\x4E24'] = L'\x5169'; // 两→兩
    m_s2t[L'\x53D1'] = L'\x767C'; // 发→發
    m_s2t[L'\x5F53'] = L'\x7576'; // 当→當
    m_s2t[L'\x4ECE'] = L'\x5F9E'; // 从→從
    m_s2t[L'\x957F'] = L'\x9577'; // 长→長
    m_s2t[L'\x8BA9'] = L'\x8B93'; // 让→讓
    m_s2t[L'\x5F00'] = L'\x958B'; // 开→開
    m_s2t[L'\x79CD'] = L'\x7A2E'; // 种→種
    m_s2t[L'\x7ECF'] = L'\x7D93'; // 经→經
    m_s2t[L'\x73B0'] = L'\x73FE'; // 现→現
    m_s2t[L'\x8FDB'] = L'\x9032'; // 进→進
    m_s2t[L'\x5934'] = L'\x982D'; // 头→頭
    m_s2t[L'\x7740'] = L'\x8457'; // 着→著
    m_s2t[L'\x52A8'] = L'\x52D5'; // 动→動
    m_s2t[L'\x4E0E'] = L'\x8207'; // 与→與
    m_s2t[L'\x95EE'] = L'\x554F'; // 问→問
    m_s2t[L'\x7ED9'] = L'\x7D66'; // 给→給
    m_s2t[L'\x65E0'] = L'\x7121'; // 无→無
    m_s2t[L'\x5173'] = L'\x95DC'; // 关→關
    m_s2t[L'\x70B9'] = L'\x9EDE'; // 点→點
    m_s2t[L'\x95F4'] = L'\x9593'; // 间→間
    m_s2t[L'\x673A'] = L'\x6A5F'; // 机→機
    m_s2t[L'\x51E0'] = L'\x5E7E'; // 几→幾
    m_s2t[L'\x8F66'] = L'\x8ECA'; // 车→車
    m_s2t[L'\x89C1'] = L'\x898B'; // 见→見
    m_s2t[L'\x5B9E'] = L'\x5BE6'; // 实→實
    m_s2t[L'\x5C06'] = L'\x5C07'; // 将→將
    m_s2t[L'\x4E48'] = L'\x9EBC'; // 么→麼
    m_s2t[L'\x4E49'] = L'\x7FA9'; // 义→義
    m_s2t[L'\x4EA7'] = L'\x7522'; // 产→產
    m_s2t[L'\x6C14'] = L'\x6C23'; // 气→氣
    m_s2t[L'\x95EE'] = L'\x554F'; // 问→問
    m_s2t[L'\x8BDD'] = L'\x8A71'; // 话→話
    m_s2t[L'\x513F'] = L'\x5152'; // 儿→兒
    m_s2t[L'\x4F53'] = L'\x9AD4'; // 体→體
    m_s2t[L'\x7535'] = L'\x96FB'; // 电→電
    m_s2t[L'\x6570'] = L'\x6578'; // 数→數
    m_s2t[L'\x6837'] = L'\x6A23'; // 样→樣
    m_s2t[L'\x4E66'] = L'\x66F8'; // 书→書
    m_s2t[L'\x95E8'] = L'\x9580'; // 门→門
    m_s2t[L'\x6761'] = L'\x689D'; // 条→條
    m_s2t[L'\x534E'] = L'\x83EF'; // 华→華
    m_s2t[L'\x5E72'] = L'\x5E79'; // 干→幹
    m_s2t[L'\x5E7F'] = L'\x5EE3'; // 广→廣
    m_s2t[L'\x8BBA'] = L'\x8AD6'; // 论→論
    m_s2t[L'\x522B'] = L'\x5225'; // 别→別
    m_s2t[L'\x6218'] = L'\x6230'; // 战→戰
    m_s2t[L'\x603B'] = L'\x7E3D'; // 总→總
    m_s2t[L'\x4F20'] = L'\x50B3'; // 传→傳
    m_s2t[L'\x573A'] = L'\x5834'; // 场→場
    m_s2t[L'\x89C9'] = L'\x89BA'; // 觉→覺
    m_s2t[L'\x5E26'] = L'\x5E36'; // 带→帶
    m_s2t[L'\x867D'] = L'\x96D6'; // 虽→雖
    m_s2t[L'\x5E76'] = L'\x4E26'; // 并→並
    m_s2t[L'\x542C'] = L'\x807D'; // 听→聽
    m_s2t[L'\x53D8'] = L'\x8B8A'; // 变→變
    m_s2t[L'\x5185'] = L'\x5167'; // 内→內
    m_s2t[L'\x8BA4'] = L'\x8A8D'; // 认→認
    m_s2t[L'\x8D44'] = L'\x8CC7'; // 资→資
    m_s2t[L'\x5E94'] = L'\x61C9'; // 应→應
    m_s2t[L'\x8BE5'] = L'\x8A72'; // 该→該
    m_s2t[L'\x8FD0'] = L'\x904B'; // 运→運
    m_s2t[L'\x5904'] = L'\x8655'; // 处→處
    m_s2t[L'\x51B3'] = L'\x6C7A'; // 决→決
    m_s2t[L'\x9898'] = L'\x984C'; // 题→題
    m_s2t[L'\x538B'] = L'\x58D3'; // 压→壓
    m_s2t[L'\x8BB0'] = L'\x8A18'; // 记→記
    m_s2t[L'\x8BB8'] = L'\x8A31'; // 许→許
    m_s2t[L'\x7EDF'] = L'\x7D71'; // 统→統
    m_s2t[L'\x52A1'] = L'\x52D9'; // 务→務
    m_s2t[L'\x6D4E'] = L'\x6FDF'; // 济→濟
    m_s2t[L'\x9A6C'] = L'\x99AC'; // 马→馬
    m_s2t[L'\x519B'] = L'\x8ECD'; // 军→軍
    m_s2t[L'\x62A5'] = L'\x5831'; // 报→報
    m_s2t[L'\x4E1C'] = L'\x6771'; // 东→東
    m_s2t[L'\x7EC4'] = L'\x7D44'; // 组→組
    m_s2t[L'\x8BC1'] = L'\x8B49'; // 证→證
    m_s2t[L'\x8054'] = L'\x806F'; // 联→聯
    m_s2t[L'\x8BA1'] = L'\x8A08'; // 计→計
    m_s2t[L'\x6784'] = L'\x69CB'; // 构→構
    m_s2t[L'\x5355'] = L'\x55AE'; // 单→單
    m_s2t[L'\x5C14'] = L'\x723E'; // 尔→爾
    m_s2t[L'\x51C6'] = L'\x6E96'; // 准→準
    m_s2t[L'\x5219'] = L'\x5247'; // 则→則
    m_s2t[L'\x5BFC'] = L'\x5C0E'; // 导→導
    m_s2t[L'\x968F'] = L'\x96A8'; // 随→隨
    m_s2t[L'\x8BBE'] = L'\x8A2D'; // 设→設
    m_s2t[L'\x7EF4'] = L'\x7DAD'; // 维→維
    m_s2t[L'\x672F'] = L'\x8853'; // 术→術
    m_s2t[L'\x7EBF'] = L'\x7DDA'; // 线→線
    m_s2t[L'\x8C03'] = L'\x8ABF'; // 调→調
    m_s2t[L'\x65AD'] = L'\x65B7'; // 断→斷
    m_s2t[L'\x94C1'] = L'\x9435'; // 铁→鐵
    m_s2t[L'\x7EE7'] = L'\x7E7C'; // 继→繼
    m_s2t[L'\x8FBE'] = L'\x9054'; // 达→達
    m_s2t[L'\x7EA6'] = L'\x7D04'; // 约→約
    m_s2t[L'\x8282'] = L'\x7BC0'; // 节→節
    m_s2t[L'\x9886'] = L'\x9818'; // 领→領
    m_s2t[L'\x6781'] = L'\x6975'; // 极→極
    m_s2t[L'\x9645'] = L'\x969B'; // 际→際
    m_s2t[L'\x89C4'] = L'\x898F'; // 规→規
    m_s2t[L'\x89C2'] = L'\x89C0'; // 观→觀
    m_s2t[L'\x56E2'] = L'\x5718'; // 团→團
    m_s2t[L'\x8BC6'] = L'\x8B58'; // 识→識
    m_s2t[L'\x9A8C'] = L'\x9A57'; // 验→驗
    m_s2t[L'\x7EAA'] = L'\x7D00'; // 纪→紀
    m_s2t[L'\x5458'] = L'\x54E1'; // 员→員
    m_s2t[L'\x9009'] = L'\x9078'; // 选→選
    m_s2t[L'\x4E1A'] = L'\x696D'; // 业→業
    m_s2t[L'\x54CD'] = L'\x97FF'; // 响→響
    m_s2t[L'\x4E50'] = L'\x6A02'; // 乐→樂
    m_s2t[L'\x6740'] = L'\x6BBA'; // 杀→殺
    m_s2t[L'\x89C6'] = L'\x8996'; // 视→視
    m_s2t[L'\x7C7B'] = L'\x985E'; // 类→類
    m_s2t[L'\x8C01'] = L'\x8AB0'; // 谁→誰
    m_s2t[L'\x8138'] = L'\x81C9'; // 脸→臉
    m_s2t[L'\x8D39'] = L'\x8CBB'; // 费→費
    m_s2t[L'\x8C08'] = L'\x8AC7'; // 谈→談
    m_s2t[L'\x4E70'] = L'\x8CB7'; // 买→買
    m_s2t[L'\x9F50'] = L'\x9F4A'; // 齐→齊
    m_s2t[L'\x8F93'] = L'\x8F38'; // 输→輸
    m_s2t[L'\x4EB2'] = L'\x89AA'; // 亲→親
    m_s2t[L'\x5C3D'] = L'\x76E1'; // 尽→盡
    m_s2t[L'\x83B7'] = L'\x7372'; // 获→獲
    m_s2t[L'\x72EC'] = L'\x7368'; // 独→獨
    m_s2t[L'\x70ED'] = L'\x71B1'; // 热→熱
    m_s2t[L'\x5F39'] = L'\x5F48'; // 弹→彈
    m_s2t[L'\x663E'] = L'\x986F'; // 显→顯
    m_s2t[L'\x7EB3'] = L'\x7D0D'; // 纳→納
    m_s2t[L'\x6742'] = L'\x96DC'; // 杂→雜
    m_s2t[L'\x96BE'] = L'\x96E3'; // 难→難
    m_s2t[L'\x9669'] = L'\x96AA'; // 险→險
    m_s2t[L'\x8BA8'] = L'\x8A0E'; // 讨→討
    m_s2t[L'\x6362'] = L'\x63DB'; // 换→換
    m_s2t[L'\x7EA7'] = L'\x7D1A'; // 级→級
    m_s2t[L'\x7A77'] = L'\x7AAE'; // 穷→窮
    m_s2t[L'\x6000'] = L'\x61F7'; // 怀→懷
    m_s2t[L'\x91CA'] = L'\x91CB'; // 释→釋
    m_s2t[L'\x4E34'] = L'\x81E8'; // 临→臨
    m_s2t[L'\x5C42'] = L'\x5C64'; // 层→層
    m_s2t[L'\x8865'] = L'\x88DC'; // 补→補
    m_s2t[L'\x5B81'] = L'\x5BE7'; // 宁→寧
    m_s2t[L'\x9C7C'] = L'\x9B5A'; // 鱼→魚
    m_s2t[L'\x5BA1'] = L'\x5BE9'; // 审→審
    m_s2t[L'\x6D4B'] = L'\x6E2C'; // 测→測
    m_s2t[L'\x4E30'] = L'\x8C50'; // 丰→豐
    m_s2t[L'\x635F'] = L'\x640D'; // 损→損
    m_s2t[L'\x8BE6'] = L'\x8A73'; // 详→詳
    m_s2t[L'\x8BFB'] = L'\x8B80'; // 读→讀
    m_s2t[L'\x8D28'] = L'\x8CEA'; // 质→質
    m_s2t[L'\x6768'] = L'\x694A'; // 杨→楊
    m_s2t[L'\x529E'] = L'\x8FA6'; // 办→辦
    m_s2t[L'\x52BF'] = L'\x52E2'; // 势→勢
    m_s2t[L'\x7801'] = L'\x78BC'; // 码→碼
    m_s2t[L'\x8FF9'] = L'\x8DE1'; // 迹→跡
    m_s2t[L'\x8BCD'] = L'\x8A5E'; // 词→詞
    m_s2t[L'\x76D8'] = L'\x76E4'; // 盘→盤
    m_s2t[L'\x7EC3'] = L'\x7DF4'; // 练→練
    m_s2t[L'\x73AF'] = L'\x74B0'; // 环→環
    m_s2t[L'\x5E01'] = L'\x5E63'; // 币→幣
    m_s2t[L'\x62A4'] = L'\x8B77'; // 护→護
    m_s2t[L'\x7840'] = L'\x790E'; // 础→礎
    m_s2t[L'\x5385'] = L'\x5EF3'; // 厅→廳
    m_s2t[L'\x82CF'] = L'\x8607'; // 苏→蘇
    m_s2t[L'\x5C5E'] = L'\x5C6C'; // 属→屬
    m_s2t[L'\x517B'] = L'\x990A'; // 养→養
    m_s2t[L'\x5174'] = L'\x8208'; // 兴→興
    m_s2t[L'\x5F03'] = L'\x68C4'; // 弃→棄
    m_s2t[L'\x8303'] = L'\x7BC4'; // 范→範
    m_s2t[L'\x521B'] = L'\x5275'; // 创→創
    m_s2t[L'\x62C5'] = L'\x64D4'; // 担→擔
    m_s2t[L'\x5199'] = L'\x5BEB'; // 写→寫
    m_s2t[L'\x7EC7'] = L'\x7E54'; // 织→織
    m_s2t[L'\x575A'] = L'\x5805'; // 坚→堅
    m_s2t[L'\x8F7D'] = L'\x8F09'; // 载→載
    m_s2t[L'\x5BBE'] = L'\x8CD3'; // 宾→賓
    m_s2t[L'\x9646'] = L'\x9678'; // 陆→陸
    m_s2t[L'\x5382'] = L'\x5EE0'; // 厂→廠
    m_s2t[L'\x4EC5'] = L'\x50C5'; // 仅→僅
    m_s2t[L'\x94B1'] = L'\x9322'; // 钱→錢
    m_s2t[L'\x76D6'] = L'\x84CB'; // 盖→蓋
    m_s2t[L'\x9648'] = L'\x9673'; // 陈→陳
    m_s2t[L'\x5212'] = L'\x5283'; // 划→劃
    m_s2t[L'\x5C81'] = L'\x6B72'; // 岁→歲
    m_s2t[L'\x8BEF'] = L'\x8AA4'; // 误→誤
    m_s2t[L'\x9A7B'] = L'\x99D0'; // 驻→駐
    m_s2t[L'\x5E84'] = L'\x838A'; // 庄→莊
    m_s2t[L'\x6269'] = L'\x64F4'; // 扩→擴
    m_s2t[L'\x7B7E'] = L'\x7C3D'; // 签→簽
    m_s2t[L'\x8BAE'] = L'\x8B70'; // 议→議
    m_s2t[L'\x6B22'] = L'\x6B61'; // 欢→歡
    m_s2t[L'\x8C22'] = L'\x8B1D'; // 谢→謝
    m_s2t[L'\x5B59'] = L'\x5B6B'; // 孙→孫
    m_s2t[L'\x7F57'] = L'\x7F85'; // 罗→羅
    m_s2t[L'\x7F16'] = L'\x7DE8'; // 编→編
    m_s2t[L'\x6267'] = L'\x57F7'; // 执→執
    m_s2t[L'\x4F18'] = L'\x512A'; // 优→優
    m_s2t[L'\x6C47'] = L'\x532F'; // 汇→匯
    m_s2t[L'\x4E1D'] = L'\x7D72'; // 丝→絲
    m_s2t[L'\x8111'] = L'\x8166'; // 脑→腦
    m_s2t[L'\x8D75'] = L'\x8D99'; // 赵→趙
    m_s2t[L'\x6863'] = L'\x6A94'; // 档→檔
    m_s2t[L'\x68A6'] = L'\x5922'; // 梦→夢
    m_s2t[L'\x8D35'] = L'\x8CB4'; // 贵→貴
    m_s2t[L'\x5218'] = L'\x5289'; // 刘→劉
    m_s2t[L'\x8BC9'] = L'\x8A34'; // 诉→訴
    m_s2t[L'\x97E9'] = L'\x97D3'; // 韩→韓
    m_s2t[L'\x6001'] = L'\x614B'; // 态→態
    m_s2t[L'\x51B2'] = L'\x885D'; // 冲→衝
    m_s2t[L'\x60CA'] = L'\x9A5A'; // 惊→驚
    m_s2t[L'\x9884'] = L'\x9810'; // 预→預
    m_s2t[L'\x4F24'] = L'\x50B7'; // 伤→傷
    m_s2t[L'\x593A'] = L'\x596A'; // 夺→奪
    m_s2t[L'\x8D5B'] = L'\x8CFD'; // 赛→賽
    m_s2t[L'\x7EAF'] = L'\x7D14'; // 纯→純
    m_s2t[L'\x987F'] = L'\x9813'; // 顿→頓
    m_s2t[L'\x5434'] = L'\x5433'; // 吴→吳
    m_s2t[L'\x6743'] = L'\x6B0A'; // 权→權
    m_s2t[L'\x536B'] = L'\x885B'; // 卫→衛
    m_s2t[L'\x51AF'] = L'\x99AE'; // 冯→馮
    m_s2t[L'\x949F'] = L'\x9418'; // 钟→鐘
    m_s2t[L'\x9C81'] = L'\x9B6F'; // 鲁→魯
    m_s2t[L'\x987B'] = L'\x9808'; // 须→須
    m_s2t[L'\x7F5A'] = L'\x7F70'; // 罚→罰
    m_s2t[L'\x996D'] = L'\x98EF'; // 饭→飯
    m_s2t[L'\x604B'] = L'\x6200'; // 恋→戀
    m_s2t[L'\x836F'] = L'\x85E5'; // 药→藥
    m_s2t[L'\x94FA'] = L'\x92EA'; // 铺→鋪
    m_s2t[L'\x5956'] = L'\x734E'; // 奖→獎
    m_s2t[L'\x76D0'] = L'\x9E7D'; // 盐→鹽
    m_s2t[L'\x5389'] = L'\x53B2'; // 厉→厲
    m_s2t[L'\x67AA'] = L'\x69CD'; // 枪→槍
    m_s2t[L'\x8C13'] = L'\x8B02'; // 谓→謂
    m_s2t[L'\x7EDD'] = L'\x7D55'; // 绝→絕
    m_s2t[L'\x51E4'] = L'\x9CF3'; // 凤→鳳
    m_s2t[L'\x574F'] = L'\x58DE'; // 坏→壞
    m_s2t[L'\x9635'] = L'\x9663'; // 阵→陣
    m_s2t[L'\x7840'] = L'\x790E'; // 础→礎
    m_s2t[L'\x804C'] = L'\x8077'; // 职→職
    m_s2t[L'\x6D45'] = L'\x6DFA'; // 浅→淺
    m_s2t[L'\x987E'] = L'\x9867'; // 顾→顧
    m_s2t[L'\x8651'] = L'\x616E'; // 虑→慮
    m_s2t[L'\x8BEF'] = L'\x8AA4'; // 误→誤
    m_s2t[L'\x7ECD'] = L'\x7D39'; // 绍→紹
    m_s2t[L'\x6E10'] = L'\x6F38'; // 渐→漸
    m_s2t[L'\x5E05'] = L'\x5E25'; // 帅→帥
    m_s2t[L'\x60E8'] = L'\x6158'; // 惨→慘
    m_s2t[L'\x521A'] = L'\x525B'; // 刚→剛
    m_s2t[L'\x591F'] = L'\x5920'; // 够→夠
    m_s2t[L'\x5B9D'] = L'\x5BF6'; // 宝→寶
    m_s2t[L'\x80DC'] = L'\x52DD'; // 胜→勝
    m_s2t[L'\x952E'] = L'\x9375'; // 键→鍵
    m_s2t[L'\x9875'] = L'\x9801'; // 页→頁
    m_s2t[L'\x94FE'] = L'\x93C8'; // 链→鏈
    m_s2t[L'\x5E93'] = L'\x5EAB'; // 库→庫
    m_s2t[L'\x4EBF'] = L'\x5104'; // 亿→億
    m_s2t[L'\x9F99'] = L'\x9F8D'; // 龙→龍
    m_s2t[L'\x51C9'] = L'\x6DBC'; // 凉→涼
    m_s2t[L'\x8BC4'] = L'\x8A55'; // 评→評
    m_s2t[L'\x7EDC'] = L'\x7D61'; // 络→絡
    m_s2t[L'\x9633'] = L'\x967D'; // 阳→陽
    m_s2t[L'\x848B'] = L'\x8523'; // 蒋→蔣
    m_s2t[L'\x4FA0'] = L'\x4FE0'; // 侠→俠
    m_s2t[L'\x989D'] = L'\x984D'; // 额→額
    m_s2t[L'\x9897'] = L'\x9846'; // 颗→顆
    m_s2t[L'\x51BB'] = L'\x51CD'; // 冻→凍
    m_s2t[L'\x9636'] = L'\x968E'; // 阶→階
    m_s2t[L'\x6447'] = L'\x6416'; // 摇→搖
    m_s2t[L'\x54DF'] = L'\x55B2'; // 哟→喲
    m_s2t[L'\x9891'] = L'\x983B'; // 频→頻
    m_s2t[L'\x626B'] = L'\x6383'; // 扫→掃
    m_s2t[L'\x94DC'] = L'\x9285'; // 铜→銅
    m_s2t[L'\x5938'] = L'\x8A87'; // 夸→誇
    m_s2t[L'\x601C'] = L'\x6190'; // 怜→憐
    m_s2t[L'\x950B'] = L'\x92D2'; // 锋→鋒
    m_s2t[L'\x8D2B'] = L'\x8CA7'; // 贫→貧
    m_s2t[L'\x94C3'] = L'\x9234'; // 铃→鈴
    m_s2t[L'\x8F9E'] = L'\x8FAD'; // 辞→辭
    m_s2t[L'\x54D1'] = L'\x555E'; // 哑→啞
    m_s2t[L'\x60E7'] = L'\x61FC'; // 惧→懼
    m_s2t[L'\x5F25'] = L'\x5F4C'; // 弥→彌
    m_s2t[L'\x53A8'] = L'\x5EDA'; // 厨→廚
    m_s2t[L'\x8721'] = L'\x881F'; // 蜡→蠟
    m_s2t[L'\x8231'] = L'\x8259'; // 舱→艙
    m_s2t[L'\x88AD'] = L'\x8972'; // 袭→襲
    m_s2t[L'\x5965'] = L'\x5967'; // 奥→奧
    m_s2t[L'\x80BF'] = L'\x816B'; // 肿→腫
    m_s2t[L'\x989C'] = L'\x984F'; // 颜→顏
    m_s2t[L'\x7F1D'] = L'\x7E2B'; // 缝→縫
    m_s2t[L'\x95EA'] = L'\x9583'; // 闪→閃
    m_s2t[L'\x98D8'] = L'\x98C4'; // 飘→飄
    m_s2t[L'\x5631'] = L'\x56D1'; // 嘱→囑
    m_s2t[L'\x80A4'] = L'\x819A'; // 肤→膚
    m_s2t[L'\x8695'] = L'\x8836'; // 蚕→蠶
    m_s2t[L'\x94DD'] = L'\x92C1'; // 铝→鋁
    m_s2t[L'\x9508'] = L'\x93FD'; // 锈→鏽
    m_s2t[L'\x8F91'] = L'\x8F2F'; // 辑→輯
    m_s2t[L'\x575B'] = L'\x58C7'; // 坛→壇
    m_s2t[L'\x53F9'] = L'\x5606'; // 叹→嘆
    m_s2t[L'\x9501'] = L'\x9396'; // 锁→鎖
    m_s2t[L'\x7A33'] = L'\x7A69'; // 稳→穩
    m_s2t[L'\x6446'] = L'\x64FA'; // 摆→擺
    m_s2t[L'\x7ED8'] = L'\x7E6A'; // 绘→繪
    m_s2t[L'\x8D26'] = L'\x8CEC'; // 账→賬
    m_s2t[L'\x4ED3'] = L'\x5009'; // 仓→倉
    m_s2t[L'\x75AF'] = L'\x760B'; // 疯→瘋
    m_s2t[L'\x54B8'] = L'\x9E79'; // 咸→鹹
    m_s2t[L'\x8747'] = L'\x8805'; // 蝇→蠅
    m_s2t[L'\x8C31'] = L'\x8B5C'; // 谱→譜
    m_s2t[L'\x707F'] = L'\x71E6'; // 灿→燦
    m_s2t[L'\x75D2'] = L'\x7662'; // 痒→癢
    m_s2t[L'\x8F7F'] = L'\x8F4E'; // 轿→轎
    m_s2t[L'\x8F86'] = L'\x8F1B'; // 辆→輛
    m_s2t[L'\x9510'] = L'\x92B3'; // 锐→銳
    m_s2t[L'\x5C1D'] = L'\x5617'; // 尝→嘗
    m_s2t[L'\x9171'] = L'\x91AC'; // 酱→醬
    m_s2t[L'\x5C7F'] = L'\x5DBC'; // 屿→嶼
    m_s2t[L'\x95F2'] = L'\x9592'; // 闲→閒
    m_s2t[L'\x6EDA'] = L'\x6EFE'; // 滚→滾
    m_s2t[L'\x9505'] = L'\x934B'; // 锅→鍋
    m_s2t[L'\x53D9'] = L'\x6558'; // 叙→敘
    m_s2t[L'\x7EF3'] = L'\x7E69'; // 绳→繩
    m_s2t[L'\x5415'] = L'\x5442'; // 吕→呂
    m_s2t[L'\x953B'] = L'\x935B'; // 锻→鍛
    m_s2t[L'\x886C'] = L'\x896F'; // 衬→襯
    m_s2t[L'\x90BB'] = L'\x9130'; // 邻→鄰
    m_s2t[L'\x7EF5'] = L'\x7DBF'; // 绵→綿
    m_s2t[L'\x89E6'] = L'\x89F8'; // 触→觸
    m_s2t[L'\x94F8'] = L'\x9444'; // 铸→鑄
    m_s2t[L'\x89C8'] = L'\x89BD'; // 览→覽
    m_s2t[L'\x8FA9'] = L'\x8FAF'; // 辩→辯
    m_s2t[L'\x7BA9'] = L'\x7C6E'; // 箩→籮
    m_s2t[L'\x98A4'] = L'\x986B'; // 颤→顫
    m_s2t[L'\x8D3F'] = L'\x8CC4'; // 贿→賄
    m_s2t[L'\x5578'] = L'\x562F'; // 啸→嘯
    m_s2t[L'\x8361'] = L'\x8569'; // 荡→蕩
    m_s2t[L'\x7F06'] = L'\x7E9C'; // 缆→纜
    m_s2t[L'\x6D9B'] = L'\x6FE4'; // 涛→濤
    m_s2t[L'\x6073'] = L'\x61C7'; // 恳→懇
    m_s2t[L'\x7F29'] = L'\x7E2E'; // 缩→縮
    m_s2t[L'\x7EAC'] = L'\x7DEF'; // 纬→緯
    m_s2t[L'\x6BC1'] = L'\x6BC0'; // 毁→毀
    m_s2t[L'\x76CF'] = L'\x76DE'; // 盏→盞
    m_s2t[L'\x7B51'] = L'\x7BC9'; // 筑→築
    m_s2t[L'\x8854'] = L'\x929C'; // 衔→銜
    m_s2t[L'\x9274'] = L'\x9452'; // 鉴→鑒
    m_s2t[L'\x507F'] = L'\x511F'; // 偿→償
    m_s2t[L'\x9970'] = L'\x98FE'; // 饰→飾
    m_s2t[L'\x5939'] = L'\x593E'; // 夹→夾
    m_s2t[L'\x7075'] = L'\x9748'; // 灵→靈
    m_s2t[L'\x7EF8'] = L'\x7DA2'; // 绸→綢
    m_s2t[L'\x5815'] = L'\x58AE'; // 堕→墮
    m_s2t[L'\x9AA4'] = L'\x9A5F'; // 骤→驟
    m_s2t[L'\x8D4F'] = L'\x8CDE'; // 赏→賞
    m_s2t[L'\x7F1A'] = L'\x7E1B'; // 缚→縛
    m_s2t[L'\x9601'] = L'\x95A3'; // 阁→閣
    m_s2t[L'\x95F7'] = L'\x60B6'; // 闷→悶
    m_s2t[L'\x80A0'] = L'\x8178'; // 肠→腸
    m_s2t[L'\x7A9C'] = L'\x7AC4'; // 窜→竄
    m_s2t[L'\x7EEA'] = L'\x7DD2'; // 绪→緒
    m_s2t[L'\x8D3C'] = L'\x8CCA'; // 贼→賊
    m_s2t[L'\x503E'] = L'\x50BE'; // 倾→傾

    // Build reverse mapping (Traditional→Simplified)
    for (const auto& pair : m_s2t)
    {
        m_t2s[pair.second] = pair.first;
    }
}