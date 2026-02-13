# Deployment Guide - MAIDOS-CodeQC

## Overview

This document describes deployment strategies for MAIDOS-CodeQC, including npm publication, standalone ZIP packages, and CI/CD integration. Version: 0.3.5.

---

## 1. Deployment Targets

MAIDOS-CodeQC supports three deployment methods:

| Method | Use Case | Distribution |
|--------|----------|--------------|
| **npm Package** | Node.js projects, CI/CD pipelines | npm registry |
| **Standalone ZIP** | Manual installation, offline environments | GitHub Releases, internal mirrors |
| **CLI Binary** | Global install for developers | npm global install |

---

## 2. npm Package Deployment

### 2.1 Prerequisites

- Node.js 18+ installed
- npm account with publish permissions
- Access to `@maidos` npm organization (if scoped package)

### 2.2 Build Process

```bash
# Clean previous builds
rm -rf dist/

# Install dependencies
npm install

# Run full build
npm run build

# Verify build outputs
ls dist/
# Expected: index.js, index.cjs, index.d.ts, cli.js, cli.cjs
```

### 2.3 Pre-Publish Checklist

- [ ] All tests pass: `npm test`
- [ ] Type check clean: `npm run typecheck`
- [ ] Lint clean: `npm run lint`
- [ ] Version bumped in `package.json`
- [ ] CHANGELOG.md updated
- [ ] README.md accurate
- [ ] LICENSE file present

### 2.4 Publish to npm

```bash
# Dry run (test package contents)
npm pack
tar -xzf maidos-codeqc-0.3.5.tgz
cat package/package.json  # Verify metadata

# Login to npm
npm login

# Publish (runs prepublishOnly script automatically)
npm publish --access public

# Verify publication
npm view @maidos/codeqc
```

### 2.5 Post-Publish Verification

```bash
# Install from npm in clean directory
mkdir /tmp/test-install
cd /tmp/test-install
npm install @maidos/codeqc

# Test CLI
npx maidos-codeqc --version
npx maidos-codeqc scan ./node_modules/@maidos/codeqc/dist

# Expected: v0.3.5, zero violations in own code
```

---

## 3. Standalone ZIP Package

### 3.1 Build ZIP Package

```bash
# Create distribution directory
mkdir -p maidos-codeqc-v0.3.5

# Copy necessary files
cp -r dist/ maidos-codeqc-v0.3.5/
cp -r templates/ maidos-codeqc-v0.3.5/
cp package.json maidos-codeqc-v0.3.5/
cp package-lock.json maidos-codeqc-v0.3.5/
cp README.md maidos-codeqc-v0.3.5/
cp TUTORIAL.md maidos-codeqc-v0.3.5/
cp LICENSE maidos-codeqc-v0.3.5/

# Create ZIP
zip -r maidos-codeqc-v0.3.5.zip maidos-codeqc-v0.3.5/

# Verify size (should be < 1 MB)
ls -lh maidos-codeqc-v0.3.5.zip
```

### 3.2 ZIP Package Contents

```
maidos-codeqc-v0.3.5/
├── dist/                 # Compiled TypeScript
│   ├── index.js          # ESM entry
│   ├── index.cjs         # CJS entry
│   ├── index.d.ts        # TypeScript definitions
│   ├── cli.js            # CLI entry (ESM)
│   └── cli.cjs           # CLI entry (CJS)
├── templates/            # Config templates
│   └── .codeqcrc.yml
├── package.json          # Package metadata
├── package-lock.json     # Dependency lock
├── README.md             # Documentation
├── TUTORIAL.md           # Chinese tutorial
└── LICENSE               # MIT license
```

### 3.3 Installation from ZIP

Users install from ZIP as follows:

```bash
# Extract ZIP
unzip maidos-codeqc-v0.3.5.zip
cd maidos-codeqc-v0.3.5/

# Install dependencies (including Tree-sitter grammars)
npm install

# Test CLI
npx tsx dist/cli.js --version

# Optional: Add to PATH
npm link  # Creates global symlink
```

---

## 4. CLI Binary Deployment

### 4.1 Global Installation

```bash
# From npm registry
npm install -g @maidos/codeqc

# Verify binary in PATH
which codeqc
which maidos-codeqc

# Test
codeqc --version
```

### 4.2 Binary Behavior

The `bin` field in `package.json` exposes two commands:

```json
{
  "bin": {
    "maidos-codeqc": "./dist/cli.js",
    "codeqc": "./dist/cli.js"
  }
}
```

Both commands execute the same CLI entry point. The shebang in `cli.js` ensures Node.js execution:

```javascript
#!/usr/bin/env node
```

---

## 5. CI/CD Integration

### 5.1 GitHub Actions Workflow

```yaml
name: Code Quality
on: [push, pull_request]

jobs:
  codeqc:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install CodeQC
        run: npm install @maidos/codeqc

      - name: Run scan
        run: npx maidos-codeqc scan --ci ./src

      - name: Upload report
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: codeqc-report
          path: codeqc-report.json
```

### 5.2 GitLab CI Configuration

```yaml
codeqc:
  image: node:20
  stage: test
  script:
    - npm install @maidos/codeqc
    - npx maidos-codeqc scan --ci ./src
  artifacts:
    when: on_failure
    paths:
      - codeqc-report.json
    expire_in: 7 days
```

### 5.3 Docker Deployment

```dockerfile
FROM node:20-alpine

# Install CodeQC globally
RUN npm install -g @maidos/codeqc

# Verify installation
RUN codeqc --version

# Set working directory
WORKDIR /workspace

# Default command: scan current directory
CMD ["codeqc", "scan", "."]
```

**Usage**:

```bash
# Build image
docker build -t maidos-codeqc:0.3.5 .

# Run scan on host directory
docker run --rm -v $(pwd):/workspace maidos-codeqc:0.3.5
```

---

## 6. Serve Mode Deployment

### 6.1 Systemd Service (Linux)

Create `/etc/systemd/system/maidos-codeqc.service`:

```ini
[Unit]
Description=MAIDOS CodeQC API Server
After=network.target

[Service]
Type=simple
User=codeqc
WorkingDirectory=/opt/maidos-codeqc
ExecStart=/usr/bin/node /opt/maidos-codeqc/dist/cli.js serve --port 3000 --path /var/repos
Restart=on-failure
Environment="NODE_ENV=production"

[Install]
WantedBy=multi-user.target
```

**Enable and start**:

```bash
sudo systemctl daemon-reload
sudo systemctl enable maidos-codeqc
sudo systemctl start maidos-codeqc
sudo systemctl status maidos-codeqc
```

### 6.2 PM2 Deployment (Node.js Process Manager)

```bash
# Install PM2
npm install -g pm2

# Start serve mode
pm2 start /opt/maidos-codeqc/dist/cli.js \
  --name codeqc-api \
  -- serve --port 3000 --path /var/repos

# Save process list
pm2 save

# Auto-start on boot
pm2 startup
```

### 6.3 Nginx Reverse Proxy

Configure Nginx to proxy requests to CodeQC API:

```nginx
server {
    listen 80;
    server_name codeqc.example.com;

    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }
}
```

**Enable WebSocket support** for dashboard real-time updates.

---

## 7. Offline Deployment

### 7.1 Bundle Dependencies

For air-gapped environments, bundle all dependencies:

```bash
# Create tarball with all dependencies
npm pack
npm install -g @maidos/codeqc --global-style
tar -czf maidos-codeqc-offline.tar.gz ~/.npm-global/lib/node_modules/@maidos/

# Transfer tarball to offline machine
scp maidos-codeqc-offline.tar.gz user@offline-host:/tmp/

# Install on offline machine
tar -xzf maidos-codeqc-offline.tar.gz -C ~/.npm-global/lib/node_modules/
npm link @maidos/codeqc
```

### 7.2 Vendored Dependencies

Alternatively, commit `node_modules/` to version control (not recommended for npm packages, but acceptable for internal deployment):

```bash
# Remove .gitignore entry for node_modules
sed -i '/node_modules/d' .gitignore

# Commit dependencies
git add node_modules/
git commit -m "Vendor dependencies for offline deployment"
```

---

## 8. Version Management

### 8.1 Semantic Versioning

MAIDOS-CodeQC follows [Semantic Versioning](https://semver.org/):

- **Major (X.0.0)**: Breaking API changes (e.g., v1.0.0 → v2.0.0)
- **Minor (0.X.0)**: New features, backward-compatible (e.g., v0.3.0 → v0.4.0)
- **Patch (0.0.X)**: Bug fixes, backward-compatible (e.g., v0.3.4 → v0.3.5)

### 8.2 Version Bump Workflow

```bash
# Patch release (bug fix)
npm version patch  # 0.3.5 → 0.3.6

# Minor release (new feature)
npm version minor  # 0.3.5 → 0.4.0

# Major release (breaking change)
npm version major  # 0.3.5 → 1.0.0

# Custom version
npm version 1.0.0-beta.1

# Push tags
git push --follow-tags
```

---

## 9. Rollback Strategy

### 9.1 npm Deprecation

If a release has critical bugs:

```bash
# Deprecate broken version
npm deprecate @maidos/codeqc@0.3.5 "Critical bug, use 0.3.6 instead"

# Users will see warning on install
npm install @maidos/codeqc@0.3.5
# npm WARN deprecated @maidos/codeqc@0.3.5: Critical bug, use 0.3.6 instead
```

### 9.2 npm Unpublish (within 72 hours)

```bash
# Unpublish specific version (only if < 72 hours old)
npm unpublish @maidos/codeqc@0.3.5

# Force unpublish (use with extreme caution)
npm unpublish @maidos/codeqc@0.3.5 --force
```

### 9.3 Rollback in CI/CD

Pin to known-good version in CI configs:

```yaml
# GitHub Actions
- run: npm install @maidos/codeqc@0.3.4  # Pin to stable version
```

---

## 10. Monitoring and Health Checks

### 10.1 Serve Mode Health Endpoint

```bash
# Check API server health
curl http://localhost:3000/health

# Expected response:
{
  "status": "ok",
  "version": "0.3.5",
  "uptime": 3600,
  "timestamp": "2025-02-13T12:00:00Z"
}
```

### 10.2 Systemd Health Check

```bash
# Check service status
sudo systemctl status maidos-codeqc

# View logs
sudo journalctl -u maidos-codeqc -f
```

### 10.3 PM2 Health Check

```bash
# Check PM2 status
pm2 status codeqc-api

# View logs
pm2 logs codeqc-api

# Restart on failure
pm2 restart codeqc-api
```

---

## 11. Security Considerations

### 11.1 Dependency Audits

```bash
# Check for vulnerabilities
npm audit

# Auto-fix (if possible)
npm audit fix

# Force fix (may break compatibility)
npm audit fix --force
```

### 11.2 Lockfile Integrity

Always commit `package-lock.json` to ensure reproducible builds:

```bash
# Verify lockfile integrity
npm ci  # Clean install from lockfile
```

### 11.3 Serve Mode Security

- **Bind to localhost only** by default (no external access)
- **Authentication**: Add custom auth middleware for production
- **Rate limiting**: Use `express-rate-limit` for API endpoints
- **HTTPS**: Deploy behind Nginx with SSL certificates

---

## 12. Deployment Checklist

Before deploying a new version:

- [ ] All tests pass (`npm test`)
- [ ] Type check clean (`npm run typecheck`)
- [ ] Lint clean (`npm run lint`)
- [ ] Version bumped in `package.json`
- [ ] CHANGELOG.md updated with release notes
- [ ] README.md reflects new features
- [ ] Built successfully (`npm run build`)
- [ ] Tested in clean install (`npm pack` → install in /tmp)
- [ ] CI/CD pipeline passes on all platforms
- [ ] Documentation updated (if API changes)
- [ ] GitHub Release created (if major/minor)
- [ ] npm package published
- [ ] Docker image updated (if applicable)

---

## 13. Troubleshooting

### 13.1 npm Publish Fails

**Error**: `403 Forbidden - PUT https://registry.npmjs.org/@maidos/codeqc`

**Solution**:
```bash
# Verify login
npm whoami

# Re-login
npm logout
npm login

# Check organization access
npm access ls-collaborators @maidos/codeqc
```

### 13.2 CLI Not Found After Global Install

**Error**: `command not found: codeqc`

**Solution**:
```bash
# Verify global install
npm list -g @maidos/codeqc

# Check npm global bin path
npm config get prefix

# Add to PATH (Bash)
echo 'export PATH="$PATH:$(npm config get prefix)/bin"' >> ~/.bashrc
source ~/.bashrc
```

### 13.3 Serve Mode Port Already in Use

**Error**: `EADDRINUSE: address already in use :::3000`

**Solution**:
```bash
# Find process using port 3000
lsof -i :3000

# Kill process
kill -9 <PID>

# Or use different port
npx maidos-codeqc serve --port 3001
```

---

*MAIDOS-CodeQC Deployment Guide v0.3.5 -- CodeQC Gate C Compliant*
