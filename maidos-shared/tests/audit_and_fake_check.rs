use maidos_chain::{EthMncContract, MncContract, ChainError};
use ethers::types::Address;

#[tokio::test]
async fn test_verify_no_more_fake_implementation() {
    // 建立合約實例
    let contract = EthMncContract::new(Address::zero());

    // 調用 get_locked
    let result = contract.get_locked("0x1234567890123456789012345678901234567890").await;

    // 驗證：如果還是「坑貨」假實現，會回傳 Ok(0)
    // 如果是「去坑化」後的實作，必須回傳 Err(ChainError::ProviderNotConfigured)
    match result {
        Err(ChainError::ProviderNotConfigured(msg)) => {
            println!("實地驗證成功：偵測到 ProviderNotConfigured 錯誤，訊息為: {}", msg);
            assert!(msg.contains("requires a configured RPC provider"));
        },
        Ok(val) => {
            panic!("驗證失敗！系統仍處於「坑貨」假實現狀態，回傳了虛假數據: {:?}", val);
        },
        Err(e) => {
            panic!("驗證失敗！回傳了非預期的錯誤類型: {:?}", e);
        }
    }
}
