# @maidos/codeqc-plugin-jvm

> JVM languages support for MAIDOS CodeQC

## Supported Languages

| Language | Extensions | Features |
|:---------|:-----------|:---------|
| **Java** | `.java` | 資源洩漏、反序列化風險、靜態可變狀態 |
| **Kotlin** | `.kt`, `.kts` | !! 強制解包、lateinit 過度使用 |
| **Scala** | `.scala`, `.sc` | null 使用、var 過度使用 |
| **Groovy** | `.groovy`, `.gradle` | GString SQL 注入、Eval 使用 |
| **Clojure** | `.clj`, `.cljs`, `.cljc` | 函數內 def、過多可變狀態 |

## Installation

```bash
npm install @maidos/codeqc-plugin-jvm
```

## Usage

### With CLI (Automatic)

```bash
# Plugin is auto-loaded when installed
npx maidos-codeqc ./src
```

### Programmatic

```typescript
import { checkJVMFile, isJVMFile } from '@maidos/codeqc-plugin-jvm';

if (isJVMFile('App.java')) {
  const violations = checkJVMFile(sourceCode, 'App.java');
  console.log(violations);
}
```

### Individual Language

```typescript
import { checkJava, checkKotlin, checkScala } from '@maidos/codeqc-plugin-jvm';

const javaViolations = checkJava(javaSource, 'App.java');
const kotlinViolations = checkKotlin(kotlinSource, 'App.kt');
```

## Language-Specific Rules

### Java

| Rule | Description |
|:-----|:------------|
| R01 | 硬編碼憑證（密碼、API Key、Token） |
| R02 | 不安全的反序列化 (ObjectInputStream) |
| R05 | 空的 catch 區塊、資源未關閉 |
| P05 | 超長方法 (>50 行) |
| P07 | 公開的可變靜態變數 |

### Kotlin

| Rule | Description |
|:-----|:------------|
| R01 | 硬編碼憑證 |
| R05 | !! 強制解包、空 catch、getOrNull 靜默失敗 |
| P05 | 超長函數 (>50 行) |
| P07 | 過多 lateinit 變數 (>5 個) |

### Scala

| Rule | Description |
|:-----|:------------|
| R01 | 硬編碼憑證 |
| R05 | null 使用（應使用 Option） |
| P05 | 超長函數 (>50 行) |
| P07 | 過多 var 可變變數 (>3 個) |

### Groovy

| Rule | Description |
|:-----|:------------|
| R01 | 硬編碼憑證（含 Gradle 簽名密碼） |
| R02 | GString SQL 注入、Eval 使用 |
| R05 | 空的 catch 區塊 |
| P05 | 超長函數 (Gradle 檔案 >80 行，其他 >50 行) |

### Clojure

| Rule | Description |
|:-----|:------------|
| R01 | 硬編碼憑證（def、map key） |
| R05 | 靜默忽略異常 (catch _ ) |
| P05 | 超長函數 (>50 行) |
| P07 | 函數內 def、過多 atom/ref/agent (>5 個) |

## License

MIT © MAIDOS
