/**
 * v3.5 R25-R28 驗證腳本
 * 用深掃發現的真實案例驗證新規則
 */

import { R25_CHECKER, R26_CHECKER, R27_CHECKER, R28_CHECKER } from './src/rules/b-redlines-r25-r28.js';

interface TestCase {
  name: string;
  source: string;
  file: string;
  expectedRule: string;
  shouldCatch: boolean;
}

const CASES: TestCase[] = [
  // === R25 日誌灌水 ===
  {
    name: 'backup_drivers 只 log (Driver)',
    source: `pub fn backup_drivers(&self, device_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("[MAIDOS-AUDIT] Backing up drivers for device: {}", device_id);
        Ok(())
    }`,
    file: 'src/core/backup/manager.rs',
    expectedRule: 'R25',
    shouldCatch: true,
  },
  {
    name: 'verify_file_signature 只 log (Driver)',
    source: `pub fn verify_file_signature(&self, file_path: &str) -> Result<bool, Box<dyn std::error::Error>> {
        log::info!("[MAIDOS-AUDIT] Verifying signature for: {}", file_path);
        Ok(true)
    }`,
    file: 'src/core/verify/signature_verifier.rs',
    expectedRule: 'R25',
    shouldCatch: true,
  },
  {
    name: 'send_message 只 log (Discord)',
    source: `pub fn send_message(&self, target: &str, message: &str) -> Result<()> {
        info!("[MAIDOS-AUDIT] Discord send_message to={} len={}", target, message.len());
        Ok(())
    }`,
    file: 'src/discord.rs',
    expectedRule: 'R25',
    shouldCatch: true,
  },
  {
    name: '真實函數不應命中 (有實際邏輯)',
    source: `pub fn compute_hash(&self, data: &[u8]) -> Result<String> {
        log::info!("Computing hash for {} bytes", data.len());
        let mut hasher = Sha256::new();
        hasher.update(data);
        Ok(format!("{:x}", hasher.finalize()))
    }`,
    file: 'src/hash.rs',
    expectedRule: 'R25',
    shouldCatch: false,
  },

  // === R26 幽靈異步 ===
  {
    name: 'async context_based_selection 無 .await (IME)',
    source: `pub async fn context_based_selection(&self, candidates: &[Candidate], context: &str) -> Result<Vec<Candidate>> {
        // If context understanding is disabled, return the first candidate
        if !self.config.enable_context {
            return Ok(candidates.to_vec());
        }
        let filtered = candidates.iter().filter(|c| c.text.contains(context)).cloned().collect();
        Ok(filtered)
    }`,
    file: 'src/ai.rs',
    expectedRule: 'R26',
    shouldCatch: true,
  },
  {
    name: 'async publish 無 .await (Bus)',
    source: `pub async fn publish(&self, topic: &str, data: &[u8]) -> Result<()> {
        info!("[MAIDOS-AUDIT] Publishing event: topic={}", topic);
        Ok(())
    }`,
    file: 'src/publisher.rs',
    expectedRule: 'R26',
    shouldCatch: true,
  },
  {
    name: '真實 async (有 .await) 不應命中',
    source: `pub async fn fetch_data(&self, url: &str) -> Result<String> {
        let response = self.client.get(url).send().await?;
        let body = response.text().await?;
        Ok(body)
    }`,
    file: 'src/fetcher.rs',
    expectedRule: 'R26',
    shouldCatch: false,
  },

  // === R27 錯誤洗白 ===
  {
    name: 'wallet sign 用 unwrap_or_default (Chain)',
    source: `pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>> {
        let private_key_bytes = hex::decode(&self.private_key).unwrap_or_default();
        let signature = sign_ecdsa(&private_key_bytes, message)?;
        Ok(signature)
    }`,
    file: 'src/wallet.rs',
    expectedRule: 'R27',
    shouldCatch: true,
  },
  {
    name: 'verify_checksum 用 unwrap_or(false)',
    source: `pub fn verify_checksum(&self, file_path: &str, expected: &str) -> bool {
        let actual = compute_hash(file_path).unwrap_or_default();
        actual == expected
    }`,
    file: 'src/verify.rs',
    expectedRule: 'R27',
    shouldCatch: true,
  },
  {
    name: '安全上下文外 unwrap_or 不應命中',
    source: `pub fn get_display_name(&self) -> String {
        self.name.clone().unwrap_or_default()
    }`,
    file: 'src/display.rs',
    expectedRule: 'R27',
    shouldCatch: false,
  },

  // === R28 複製軍團 ===
  {
    name: 'prod 代碼中 TcpListener::bind 127.0.0.1:0',
    source: `fn spawn_http_server(status: &str, body: &str) -> (String, JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let base_url = format!("http://127.0.0.1:{}", port);
        (base_url, handle)
    }`,
    file: 'src/slack.rs',
    expectedRule: 'R28',
    shouldCatch: true,
  },
  {
    name: 'test 檔案中 TcpListener 不應命中',
    source: `fn spawn_http_server(status: &str, body: &str) -> (String, JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        (format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port()), handle)
    }`,
    file: 'tests/integration_test.rs',
    expectedRule: 'R28',
    shouldCatch: false,
  },
];

// === Run ===
let passed = 0;
let failed = 0;
const ALL = [R25_CHECKER, R26_CHECKER, R27_CHECKER, R28_CHECKER];

for (const tc of CASES) {
  const violations = ALL.flatMap(c => c.checkSource?.(tc.source, tc.file) ?? []);
  const caught = violations.some(v => v.ruleId === tc.expectedRule);

  if (tc.shouldCatch && caught) {
    console.log(`✅ PASS | ${tc.expectedRule} caught: ${tc.name}`);
    passed++;
  } else if (!tc.shouldCatch && !caught) {
    console.log(`✅ PASS | ${tc.expectedRule} skipped (correctly): ${tc.name}`);
    passed++;
  } else if (tc.shouldCatch && !caught) {
    console.log(`❌ MISS | ${tc.expectedRule} should catch: ${tc.name}`);
    if (violations.length > 0) console.log(`   (caught by ${violations.map(v => v.ruleId).join(',')} instead)`);
    failed++;
  } else {
    console.log(`❌ FALSE+ | ${tc.expectedRule} should skip: ${tc.name}`);
    console.log(`   (incorrectly caught: ${violations.map(v => v.ruleId + ':' + v.message.substring(0, 40)).join(', ')})`);
    failed++;
  }
}

console.log(`\n═══ 結果: ${passed}/${CASES.length} 通過, ${failed} 失敗 ═══`);
process.exit(failed > 0 ? 1 : 0);
