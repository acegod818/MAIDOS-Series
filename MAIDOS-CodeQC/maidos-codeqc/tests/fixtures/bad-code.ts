// Bad TypeScript example for testing
// Contains multiple Code-QC violations

// R01: 硬編碼憑證
const password = "super_secret_password_123";
const api_key = "sk-1234567890abcdefghijklmnopqrstuvwxyz";
const AWS_SECRET = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";

// R05: 忽略錯誤處理
try {
  riskyOperation();
} catch (e) { }

fetchData()
  .then(data => process(data))
  .catch(() => {});

// R07: 關閉安全功能
const config = {
  rejectUnauthorized: false,
  ssl_verify: false,
};
process.env.NODE_TLS_REJECT_UNAUTHORIZED = "0";

// P05: 超長函數 (這裡簡化示意)
function veryLongFunction(param1: string, param2: number) {
  console.log("line 1");
  console.log("line 2");
  console.log("line 3");
  console.log("line 4");
  console.log("line 5");
  console.log("line 6");
  console.log("line 7");
  console.log("line 8");
  console.log("line 9");
  console.log("line 10");
  console.log("line 11");
  console.log("line 12");
  console.log("line 13");
  console.log("line 14");
  console.log("line 15");
  console.log("line 16");
  console.log("line 17");
  console.log("line 18");
  console.log("line 19");
  console.log("line 20");
  console.log("line 21");
  console.log("line 22");
  console.log("line 23");
  console.log("line 24");
  console.log("line 25");
  console.log("line 26");
  console.log("line 27");
  console.log("line 28");
  console.log("line 29");
  console.log("line 30");
  console.log("line 31");
  console.log("line 32");
  console.log("line 33");
  console.log("line 34");
  console.log("line 35");
  console.log("line 36");
  console.log("line 37");
  console.log("line 38");
  console.log("line 39");
  console.log("line 40");
  console.log("line 41");
  console.log("line 42");
  console.log("line 43");
  console.log("line 44");
  console.log("line 45");
  console.log("line 46");
  console.log("line 47");
  console.log("line 48");
  console.log("line 49");
  console.log("line 50");
  console.log("line 51");
  console.log("line 52");
}

// P06: 深層嵌套
function deepNesting(a: boolean, b: boolean, c: boolean, d: boolean) {
  if (a) {
    if (b) {
      if (c) {
        if (d) {
          if (true) {
            console.log("too deep!");
          }
        }
      }
    }
  }
}

// P09: 無意義命名
const temp = getValue();
const data = fetchData();
const info = getInfo();
const foo = 1;
const bar = 2;

// P10: 過長參數
function tooManyParams(a: string, b: number, c: boolean, d: object, e: any[], f: Function, g: Date) {
  return { a, b, c, d, e, f, g };
}

// P13: TODO 堆積
// TODO: 修復這個
// TODO: 重構這段
// TODO: 加入錯誤處理
// TODO: 增加測試
// TODO: 優化性能
// TODO: 文檔補充
// FIXME: 記憶體洩漏
// FIXME: 競態條件
// HACK: 臨時解法
// XXX: 需要審查
// TODO: 再來一個
// TODO: 超過十個了
