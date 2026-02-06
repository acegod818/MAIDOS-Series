//! 測試項目的主文件
//! 
//! 用於測試MAIDOS Forge的編譯功能

fn main() {
    println!("Hello, MAIDOS Forge!");
    greet_world();
}

fn greet_world() {
    println!("歡迎使用MAIDOS Forge v2.1!");
    
    // 顯示一些系統信息
    show_system_info();
}

fn show_system_info() {
    println!("系統信息:");
    println!("  - 平台: 跨平台編譯框架");
    println!("  - 語言: Rust");
    println!("  - 版本: 2.1");
    
    // 模擬一些計算
    let result = calculate_something(10, 20);
    println!("  - 計算結果: {}", result);
}

fn calculate_something(a: i32, b: i32) -> i32 {
    a + b * 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_something() {
        assert_eq!(calculate_something(10, 20), 50);
        assert_eq!(calculate_something(0, 0), 0);
        assert_eq!(calculate_something(-1, 1), 1);
    }
    
    #[test]
    fn test_greet_world() {
        // 這是一個簡單的測試，確保函數不會崩潰
        greet_world();
        assert!(true);
    }
}