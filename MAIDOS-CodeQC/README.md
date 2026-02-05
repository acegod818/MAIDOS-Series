# MAIDOS CodeQC

> Code Quality Control tool implementing Code-QC v3.3 standards

## Quick Start

### 1. Install

**Windows:**
```cmd
install.cmd
```

**Linux/Mac:**
```bash
chmod +x install.sh
./install.sh
```

### 2. Run

**Windows:**
```cmd
codeqc.cmd [target]
codeqc.cmd .\src
codeqc.cmd -l D .\project
```

**Linux/Mac:**
```bash
./codeqc.sh [target]
./codeqc.sh ./src
./codeqc.sh -l D ./project
```

## Requirements

- Node.js 18+

## Features

- Static analysis with AST-based code analysis
- Code-QC v3.3 rules (B: Work Discipline + C: Acceptance Criteria)
- 43 languages supported
- Console, JSON, HTML reporters
- CI/CD ready (GitHub Actions, GitLab CI)
- Web UI dashboard

## Usage

```
codeqc [options] [target]

Options:
  -l, --level <B|C|D>       Check level (default: D)
                            B = Work Discipline (41 rules)
                            C = Acceptance Criteria (~50 rules)
                            D = B + C (~91 rules)

  -C, --category <types>    Analysis types (comma-separated)
                            security,sec,s  = Security (R01-R12)
                            structure,t     = Structure (P03,P05-P07,P10)
                            quality,q       = Quality (P04,P09,P12-P14)
                            all,a           = All (default)

  -r, --reporter <type>     Output format (default: console)
                            console = Colored terminal
                            json    = JSON format
                            html    = HTML report

  -o, --output <file>       Output file (for json/html)
  --ci                      CI mode (exit 1 on errors)
  -h, --help                Show help

Examples:
  codeqc ./src                       # Scan ./src folder
  codeqc -C security ./src           # Security only
  codeqc -r html -o report.html ./   # HTML report
  codeqc --ci ./src                  # CI mode
```

## Package Structure

```
MAIDOS-CodeQC/
├── install.cmd / install.sh     # One-click install
├── codeqc.cmd / codeqc.sh       # Runner scripts
├── maidos-codeqc/               # Core package
│   ├── src/                     # TypeScript source
│   ├── dist/                    # Compiled JS
│   ├── tests/                   # Tests
│   └── web-ui/                  # Dashboard
└── maidos-codeqc-plugin-*/      # Language plugins
    ├── plugin-config            # JSON, YAML, TOML, XML
    ├── plugin-data              # SQL, R, Julia
    ├── plugin-dotnet            # C#, F#, VB.NET
    ├── plugin-enterprise        # COBOL, ABAP, PL/SQL, Fortran, VBA, RPG
    ├── plugin-functional        # Elixir, Erlang, Haskell, OCaml
    ├── plugin-jvm               # Java, Kotlin, Scala, Groovy, Clojure
    ├── plugin-mobile            # Swift, Objective-C, Dart
    ├── plugin-scripting         # Shell, PowerShell, Perl, Lua
    ├── plugin-systems           # C, C++, Zig, Nim
    └── plugin-web               # PHP, Ruby
```

## License

MIT
