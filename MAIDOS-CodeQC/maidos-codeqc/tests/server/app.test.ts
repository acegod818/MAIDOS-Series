/**
 * CodeQC v3.3 — server smoke tests
 *
 * Covers the USB/SaaS-style API server behaviour:
 * - start/stop (important for vitest stability)
 * - core endpoints + dashboard path resolution
 */

import { describe, it, expect } from 'vitest';
import { startServerWithHandle } from '../../src/server/app.js';

async function closeServer(server: import('node:http').Server) {
  await new Promise<void>((resolve, reject) => {
    server.close((err?: Error) => (err ? reject(err) : resolve()));
  });
}

describe('server/app', () => {
  it('serves health/version/rules/dashboard', async () => {
    const server = await startServerWithHandle({ host: '127.0.0.1', port: 0, cors: false });
    try {
      const addr = server.address();
      expect(addr && typeof addr === 'object').toBe(true);
      const port = (addr as { port: number }).port;
      const base = `http://127.0.0.1:${port}`;

      const health = await fetch(`${base}/api/v1/health`);
      expect(health.status).toBe(200);
      const healthJson = await health.json();
      expect(healthJson.ok).toBe(true);

      const ver = await fetch(`${base}/api/v1/version`);
      expect(ver.status).toBe(200);
      const verJson = await ver.json();
      expect(verJson.ok).toBe(true);
      expect(typeof verJson.version).toBe('string');
      expect(verJson.engine).toBe('v3.3');

      const rules = await fetch(`${base}/api/v1/rules`);
      expect(rules.status).toBe(200);
      const rulesJson = await rules.json();
      expect(rulesJson.ok).toBe(true);
      expect(rulesJson.redlines).toBeGreaterThan(0);

      const dash = await fetch(`${base}/dashboard`);
      expect(dash.status).toBe(200);
      const dashHtml = await dash.text();
      expect(dashHtml).toContain('CodeQC');
    } finally {
      await closeServer(server);
    }
  });

  it('enforces apiKey when configured', async () => {
    const server = await startServerWithHandle({ host: '127.0.0.1', port: 0, cors: false, apiKey: 'secret' });
    try {
      const addr = server.address();
      const port = (addr as { port: number }).port;
      const base = `http://127.0.0.1:${port}`;

      const noAuth = await fetch(`${base}/api/v1/health`);
      expect(noAuth.status).toBe(401);

      const ok = await fetch(`${base}/api/v1/health`, { headers: { Authorization: 'Bearer secret' } });
      expect(ok.status).toBe(200);
      expect((await ok.json()).ok).toBe(true);
    } finally {
      await closeServer(server);
    }
  });

  it('accepts scan + fraud payloads', async () => {
    const server = await startServerWithHandle({ host: '127.0.0.1', port: 0, cors: false });
    try {
      const addr = server.address();
      const port = (addr as { port: number }).port;
      const base = `http://127.0.0.1:${port}`;

      const scan = await fetch(`${base}/api/v1/scan`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          level: 'D',
          files: [{ path: 'src/a.ts', content: 'export const x = 1;\\n' }],
        }),
      });
      expect(scan.status).toBe(200);
      const scanJson = await scan.json();
      expect(scanJson.ok).toBe(true);
      expect(scanJson.result).toBeTruthy();

      const fraud = await fetch(`${base}/api/v1/fraud`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          files: [{ path: 'src/a.ts', content: 'export const x = 1;\\n' }],
        }),
      });
      expect(fraud.status).toBe(200);
      const fraudJson = await fraud.json();
      expect(fraudJson.ok).toBe(true);
      expect(fraudJson.fraudCount).toBe(0);
    } finally {
      await closeServer(server);
    }
  });
});

