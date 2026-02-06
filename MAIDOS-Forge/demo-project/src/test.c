#include <stdio.h>

/**
 * 測試C語言文件
 * 
 * 用於測試MAIDOS Forge對C語言的支持
 */

int calculate(int a, int b) {
    return a + b * 2;
}

void greet(void) {
    printf("Hello from C language!\n");
    printf("Welcome to MAIDOS Forge v2.1\n");
}

int main(void) {
    greet();
    
    // 執行一些計算
    int result = calculate(10, 20);
    printf("Calculation result: %d\n", result);
    
    return 0;
}