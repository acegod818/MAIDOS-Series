/**
 * CodeQC v3.3 — SaaS API Server
 * 
 * REST API for CodeQC-as-a-Service
 * 
 * Endpoints:
 *   POST /api/v1/scan          — 掃描代碼 (快速)
 *   POST /api/v1/pipeline      — 完整 Pipeline (十步走線)
 *   POST /api/v1/fraud         — 反詐欺掃描
 *   GET  /api/v1/rules         — 查詢規則
 *   GET  /api/v1/health        — 健康檢查
 *   GET  /api/v1/version       — 版本資訊
 * 
 * 啟動: npx maidos-codeqc serve --port 3333
 */

import { createServer, IncomingMessage, ServerResponse } from 'node:http';
import type { Server } from 'node:http';
import { existsSync, readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { analyze, VERSION, CODEQC_VERSION, checkFraud } from '../index.js';
import { runPipeline } from '../engine/pipeline.js';
import type { CheckLevel } from '../types.js';

// =============================================================================
// Types
// =============================================================================

export interface ServerConfig {
  port: number;
  host: string;
  cors: boolean;
  apiKey?: string;          // optional auth
  rateLimit?: number;       // req/min
}

const DEFAULT_SERVER_CONFIG: ServerConfig = {
  port: 3333,
  host: '0.0.0.0',
  cors: true,
};

// =============================================================================
// Helpers
// =============================================================================

function readBody(req: IncomingMessage): Promise<string> {
  return new Promise((resolve, reject) => {
    let data = '';
    req.on('data', (chunk: Buffer) => { data += chunk.toString(); });
    req.on('end', () => resolve(data));
    req.on('error', reject);
  });
}

function json(res: ServerResponse, status: number, body: unknown) {
  res.writeHead(status, { 'Content-Type': 'application/json' });
  res.end(JSON.stringify(body));
}

function html(res: ServerResponse, status: number, body: string) {
  res.writeHead(status, { 'Content-Type': 'text/html; charset=utf-8' });
  res.end(body);
}

function cors(res: ServerResponse) {
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type, Authorization');
}

// =============================================================================
// Route Handlers
// =============================================================================

async function handleScan(req: IncomingMessage, res: ServerResponse) {
  const body = JSON.parse(await readBody(req));
  const { files, level = 'D' } = body as { files: Array<{ path: string; content: string }>; level?: CheckLevel };

  if (!files || !Array.isArray(files)) {
    return json(res, 400, { error: 'files array required' });
  }

  const result = analyze({ files, level, targetPath: '.' });
  json(res, 200, { ok: true, result });
}

async function handlePipeline(req: IncomingMessage, res: ServerResponse) {
  const body = JSON.parse(await readBody(req));
  const { files, projectPath = '.', grade = 'E' } = body as {
    files: Array<{ file: string; source: string }>;
    projectPath?: string;
    grade?: 'E' | 'F';
  };

  if (!files || !Array.isArray(files)) {
    return json(res, 400, { error: 'files array required' });
  }

  const ctx = {
    targetPath: projectPath,
    files: files.map(f => ({ path: f.file, content: f.source })),
    grade: grade || 'E' as const,
    evidenceDir: 'evidence',
  };

  const result = runPipeline(ctx);
  json(res, 200, { ok: true, result });
}

async function handleFraud(req: IncomingMessage, res: ServerResponse) {
  const body = JSON.parse(await readBody(req));
  const { files } = body as { files: Array<{ path: string; content: string }> };

  if (!files || !Array.isArray(files)) {
    return json(res, 400, { error: 'files array required' });
  }

  const violations = files.flatMap(f => checkFraud(f.content, f.path));
  json(res, 200, { ok: true, fraudCount: violations.length, violations });
}

function handleRules(_req: IncomingMessage, res: ServerResponse) {
  // Dynamic import to avoid circular
  import('../rules/index.js').then(mod => {
    json(res, 200, {
      ok: true,
      axioms: mod.AXIOMS?.length ?? 0,
      redlines: mod.REDLINES?.length ?? 0,
      prohibitions: mod.PROHIBITIONS?.length ?? 0,
      gates: mod.GATES?.length ?? 0,
    });
  });
}

function handleHealth(_req: IncomingMessage, res: ServerResponse) {
  json(res, 200, { ok: true, status: 'healthy', uptime: process.uptime() });
}

function handleVersion(_req: IncomingMessage, res: ServerResponse) {
  json(res, 200, { ok: true, version: VERSION, codeqc: CODEQC_VERSION, engine: 'v3.3' });
}

function getDashboardPath(): string {
  // Prefer locating dashboard relative to the CLI entry (dist/cli.js -> ../web-ui/dashboard.html).
  // This keeps USB deployments stable even when cwd is arbitrary.
  const argv1 = process.argv[1] || '';
  try {
    const distDir = dirname(argv1);
    const candidate = resolve(distDir, '..', 'web-ui', 'dashboard.html');
    if (existsSync(candidate)) return candidate;
  } catch { /* ignore */ }

  // Fallback: relative to cwd (dev mode)
  return resolve(process.cwd(), 'web-ui', 'dashboard.html');
}

// =============================================================================
// Router
// =============================================================================

async function router(req: IncomingMessage, res: ServerResponse, config: ServerConfig) {
  if (config.cors) cors(res);

  // Preflight
  if (req.method === 'OPTIONS') {
    res.writeHead(204);
    return res.end();
  }

  // Auth check
  if (config.apiKey) {
    const auth = req.headers.authorization;
    if (auth !== `Bearer ${config.apiKey}`) {
      return json(res, 401, { error: 'unauthorized' });
    }
  }

  const url = req.url || '/';
  const method = req.method || 'GET';

  try {
    // Dashboard UI (optional, but makes the USB experience much easier).
    // GET /dashboard -> serve embedded dashboard.html from web-ui/
    if (method === 'GET' && (url === '/dashboard' || url.startsWith('/dashboard?'))) {
      const p = getDashboardPath();
      if (!existsSync(p)) {
        return html(res, 404, `<h1>CodeQC dashboard not found</h1><p>Missing: ${p}</p>`);
      }
      const content = readFileSync(p, 'utf-8');
      return html(res, 200, content);
    }

    // POST routes
    if (method === 'POST' && url === '/api/v1/scan') return handleScan(req, res);
    if (method === 'POST' && url === '/api/v1/pipeline') return handlePipeline(req, res);
    if (method === 'POST' && url === '/api/v1/fraud') return handleFraud(req, res);

    // GET routes
    if (method === 'GET' && url === '/api/v1/rules') return handleRules(req, res);
    if (method === 'GET' && url === '/api/v1/health') return handleHealth(req, res);
    if (method === 'GET' && url === '/api/v1/version') return handleVersion(req, res);
    if (method === 'GET' && url === '/') return json(res, 200, {
      name: 'MAIDOS CodeQC API',
      version: VERSION,
      engine: 'Code-QC v3.3',
      endpoints: [
        'POST /api/v1/scan',
        'POST /api/v1/pipeline',
        'POST /api/v1/fraud',
        'GET  /api/v1/rules',
        'GET  /api/v1/health',
        'GET  /api/v1/version',
      ],
    });

    json(res, 404, { error: 'not found' });
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : '';
    json(res, 500, { error: msg || 'internal server error' });
  }
}

// =============================================================================
// Server
// =============================================================================

export function startServer(config: Partial<ServerConfig> = {}): Promise<void> {
  const cfg: ServerConfig = { ...DEFAULT_SERVER_CONFIG, ...config };

  return new Promise((resolve) => {
    const server = createServer((req, res) => router(req, res, cfg));
    server.listen(cfg.port, cfg.host, () => {
      console.log(`\n🔧 MAIDOS CodeQC API v${VERSION} (Code-QC v${CODEQC_VERSION})`);
      console.log(`   Server: http://${cfg.host}:${cfg.port}`);
      console.log(`   Engine: v3.3 軟體工程硬體化`);
      console.log(`   Auth:   ${cfg.apiKey ? 'enabled' : 'disabled'}`);
      console.log(`\n   POST /api/v1/scan      — 快速掃描`);
      console.log(`   POST /api/v1/pipeline  — 十步走線`);
      console.log(`   POST /api/v1/fraud     — 反詐欺掃描`);
      console.log(`   GET  /api/v1/rules     — 規則查詢`);
      console.log(`   GET  /api/v1/health    — 健康檢查\n`);
      resolve();
    });
  });
}

/**
 * Start server and return the Server instance for tests/embedders.
 * This keeps the default CLI behaviour (process stays alive), while enabling clean shutdown in vitest.
 */
export async function startServerWithHandle(config: Partial<ServerConfig> = {}): Promise<Server> {
  const cfg: ServerConfig = { ...DEFAULT_SERVER_CONFIG, ...config };
  return new Promise((resolve) => {
    const server = createServer((req, res) => router(req, res, cfg));
    server.listen(cfg.port, cfg.host, () => resolve(server));
  });
}
