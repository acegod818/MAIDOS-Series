# MAIDOS-CodeQC -- Dependency Manifest

| Field     | Value              |
|-----------|--------------------|
| Product   | MAIDOS-CodeQC      |
| Version   | v3.0               |
| Type      | Dependencies       |

## Runtime Dependencies

| Package              | Version   | Purpose                              |
|----------------------|-----------|--------------------------------------|
| typescript           | ^5.x      | TypeScript language                  |
| glob                 | ^10.x     | File pattern matching                |
| chalk                | ^5.x      | Terminal colored output              |
| commander            | ^12.x     | CLI argument parsing                 |
| fast-glob            | ^3.x      | High-performance file globbing       |
| express              | ^4.x      | Web UI server                        |
| ws                   | ^8.x      | WebSocket for dashboard updates      |

## Development Dependencies

| Package              | Version   | Purpose                              |
|----------------------|-----------|--------------------------------------|
| tsup                 | latest    | TypeScript bundler                   |
| vitest               | latest    | Test runner                          |
| @types/node          | ^20.x     | Node.js type definitions             |
| @types/express       | ^4.x      | Express type definitions             |
| eslint               | ^8.x      | Code linter                          |
| prettier             | ^3.x      | Code formatter                       |

## System Requirements

| Requirement          | Minimum   | Recommended                          |
|----------------------|-----------|--------------------------------------|
| Node.js              | 20.x      | 22.x (LTS)                          |
| npm                  | 10.x      | Latest                               |
| OS                   | Windows 10+ / macOS 12+ / Linux | Any modern OS       |
| Disk Space           | 100 MB    | 500 MB (with all plugins)           |
| RAM                  | 256 MB    | 512 MB                              |

## Plugin Dependencies

| Plugin                        | Extra Dependencies              |
|-------------------------------|---------------------------------|
| maidos-codeqc-plugin-systems  | None (uses system compilers)    |
| maidos-codeqc-plugin-web      | None (uses npm/node)            |
| maidos-codeqc-plugin-dotnet   | .NET SDK detection              |
| maidos-codeqc-plugin-jvm      | JDK detection                   |
| maidos-codeqc-plugin-mobile   | Android SDK / Xcode detection   |

## Dependency Update Policy

- Dependencies are pinned to major versions using `^` ranges
- Security updates are applied within 48 hours of advisory
- Major version upgrades are tested in staging before adoption

*MAIDOS-CodeQC DEPS v3.0 -- CodeQC Gate C Compliant*
