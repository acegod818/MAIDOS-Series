import { randomUUID, createHash } from 'crypto';

// src/languages.ts
var EXTENSION_TO_LANGUAGE = {
  // Core (5)
  ".ts": "typescript",
  ".tsx": "typescript",
  ".js": "javascript",
  ".jsx": "javascript",
  ".mjs": "javascript",
  ".cjs": "javascript",
  ".py": "python",
  ".rs": "rust",
  ".go": "go",
  // JVM (5)
  ".java": "java",
  ".kt": "kotlin",
  ".kts": "kotlin",
  ".scala": "scala",
  ".groovy": "groovy",
  ".gvy": "groovy",
  ".clj": "clojure",
  ".cljs": "clojure",
  ".cljc": "clojure",
  // .NET (3)
  ".cs": "csharp",
  ".fs": "fsharp",
  ".fsx": "fsharp",
  ".vb": "vbnet",
  // Mobile (3)
  ".swift": "swift",
  ".m": "objc",
  ".mm": "objc",
  ".dart": "dart",
  // Systems (4)
  ".c": "c",
  ".h": "cpp",
  ".cpp": "cpp",
  ".cxx": "cpp",
  ".cc": "cpp",
  ".hpp": "cpp",
  ".hxx": "cpp",
  ".zig": "zig",
  ".nim": "nim",
  // Web (2)
  ".php": "php",
  ".rb": "ruby",
  ".erb": "ruby",
  // Scripting (4)
  ".sh": "shell",
  ".bash": "shell",
  ".zsh": "shell",
  ".ps1": "powershell",
  ".psm1": "powershell",
  ".pl": "perl",
  ".pm": "perl",
  ".lua": "lua",
  // Data (3)
  ".sql": "sql",
  ".r": "r",
  ".jl": "julia",
  // Config (4)
  ".yaml": "yaml",
  ".yml": "yaml",
  ".json": "json",
  ".toml": "toml",
  ".xml": "xml",
  // Functional (4)
  ".ex": "elixir",
  ".exs": "elixir",
  ".hs": "haskell",
  ".lhs": "haskell",
  ".ml": "ocaml",
  ".mli": "ocaml",
  ".erl": "erlang",
  ".hrl": "erlang",
  // Enterprise (6)
  ".cob": "cobol",
  ".cbl": "cobol",
  ".cpy": "cobol",
  ".abap": "abap",
  ".abs": "abap",
  ".pls": "plsql",
  ".plb": "plsql",
  ".pks": "plsql",
  ".pkb": "plsql",
  ".f": "fortran",
  ".f90": "fortran",
  ".f95": "fortran",
  ".for": "fortran",
  ".bas": "vba",
  ".cls": "vba",
  ".frm": "vba",
  ".vbs": "vba",
  ".rpg": "rpg",
  ".rpgle": "rpg",
  ".sqlrpgle": "rpg"
};
var SUPPORTED_EXTENSIONS = Object.keys(EXTENSION_TO_LANGUAGE);
function detectLanguageFromPath(file) {
  const ext = "." + (file.split(".").pop() || "").toLowerCase();
  return EXTENSION_TO_LANGUAGE[ext] || null;
}
function isSupportedPath(file) {
  return detectLanguageFromPath(file) !== null;
}

// src/z-axis.ts
var IAV_DISQUALIFIERS = {
  q1_dataSource: ["\u786C\u7DE8\u78BC", "\u9ED8\u8A8D\u503C", "\u4E0D\u77E5\u9053", "hardcoded", "default", "unknown"],
  q2_callChain: ["\u6C92\u6709", "\u76F4\u63A5return", "none", "direct return"],
  q3_inputOutput: ["\u4E0D\u4F9D\u8CF4", "\u56FA\u5B9A\u503C", "not dependent", "fixed"],
  q4_errorHandling: ["\u5FFD\u7565", "return\u9ED8\u8A8D", "ignore", "return default"],
  q5_proof: ["\u7DE8\u8B6F\u901A\u904E", "\u6C92\u5831\u932F", "compiles", "no error"]
};
var BLDS_LEVELS = {
  0: "\u8A50\u6B3A \u2014 \u7A7A\u65B9\u6CD5/return\u9ED8\u8A8D/\u786C\u7DE8\u78BC",
  1: "\u5047\u8CA8 \u2014 \u6709\u4EE3\u78BC\u4F46\u4E0D\u8A2A\u554F\u771F\u5BE6\u6578\u64DA\u6E90",
  2: "\u534A\u6210\u54C1 \u2014 \u8A2A\u554F\u6578\u64DA\u6E90\u4F46\u7F3A\u932F\u8AA4\u8655\u7406",
  3: "\u5408\u683C \u2014 \u5B8C\u6574\u8ABF\u7528\u93C8+\u932F\u8AA4\u8655\u7406+\u6B63\u78BA\u8F49\u63DB",
  4: "\u512A\u79C0 \u2014 \u5408\u683C+\u908A\u754C\u8655\u7406+\u6027\u80FD\u8003\u91CF",
  5: "\u6B66\u5668\u7D1A \u2014 \u512A\u79C0+\u53EF\u6E2C+\u53EF\u91CD\u64AD+\u5BE9\u8A08\u65E5\u8A8C"
};
var BLDS_GATE_MINIMUM = 3;

// src/dod.ts
var DOD_DEFINITIONS = [
  { id: 1, name: "\u5BE6\u73FE\u8B49\u660E", verification: "redline.log = 0 (\u7121\u65B7\u8DEF/\u77ED\u8DEF)" },
  { id: 2, name: "\u88DC\u5B8C\u8B49\u660E", verification: "impl.log + mapping.log (\u8D70\u7DDA\u9023\u901A)" },
  { id: 3, name: "\u898F\u683C\u8B49\u660E", verification: "SPEC 100% + 0 MISSING (\u96FB\u8DEF\u5716\u5B8C\u6574)" },
  { id: 4, name: "\u540C\u6B65\u8B49\u660E", verification: "sync.log = 0 (\u8173\u4F4D\u63A5\u89F8\u826F\u597D)" },
  { id: 5, name: "\u7DE8\u8B6F\u8B49\u660E", verification: "build.log 0e/0w (\u710A\u63A5\u54C1\u8CEA)" },
  { id: 6, name: "\u4EA4\u4ED8\u8B49\u660E", verification: "package.log + run.log (\u53EF\u4E0A\u96FB)" },
  { id: 7, name: "\u771F\u5BE6\u6027\u8B49\u660E", verification: "iav.log PASS + BLDS \u2265 3 (\u4FE1\u865F\u771F\u5BE6)" },
  { id: 8, name: "\u53CD\u8A50\u6B3A\u8B49\u660E", verification: "fraud.log = 0 (ESD\u901A\u904E)" }
];

// src/hardwarization.ts
var HARDWARIZATION_PILLARS = {
  PINOUT: { id: 1, name: "\u8173\u4F4D\u5316", en: "Pinout", maps: "A", question: "\u8173\u4F4D\u5B9A\u7FA9\u5B8C\u6574\u55CE\uFF1F", faultIfMissing: "\u77ED\u8DEF" },
  WIRING: { id: 2, name: "\u8D70\u7DDA\u5316", en: "Wiring", maps: "D+A", question: "Pipeline\u56FA\u5B9A\u4E14\u4E0D\u53EF\u8DF3\u6B65\u55CE\uFF1F", faultIfMissing: "\u65B7\u8DEF" },
  LOGIC_GATE: { id: 3, name: "\u9598\u9580\u5316", en: "Logic Gate", maps: "C:G1-G4", question: "\u9598\u9580\u5168HIGH\u55CE\uFF1F", faultIfMissing: "\u9598\u9580\u4E0D\u958B" },
  INSTRUMENT: { id: 4, name: "\u91CF\u6E2C\u5316", en: "Instrumentation", maps: "C:YXZ", question: "\u6CE2\u5F62\u51FA\u4E86\u55CE\uFF1F", faultIfMissing: "\u7121\u6CE2\u5F62" },
  PROTECTION: { id: 5, name: "\u4FDD\u8B77\u5316", en: "Protection", maps: "B+Z", question: "\u4FDD\u8B77\u96FB\u8DEF\u5B8C\u597D\u55CE\uFF1F", faultIfMissing: "\u7194\u65B7" }
};
var CIRCUIT_WORLDVIEW = {
  A: { name: "\u898F\u683C\u6A19\u6E96", nameEn: "Schematic", circuit: "\u96FB\u8DEF\u5716", question: "\u96FB\u8DEF\u5716\u6709\u6C92\u6709\u756B\u597D\uFF1F", doc: "CodeQC_v3.3_A.md", pillar: "PINOUT" },
  B: { name: "\u5DE5\u4F5C\u7D00\u5F8B", nameEn: "Protection Circuit", circuit: "\u4FDD\u8B77\u96FB\u8DEF", question: "\u4FDD\u8B77\u96FB\u8DEF\u6709\u6C92\u6709\u88DD\uFF1F", doc: "CodeQC_v3.3_B.md", pillar: "PROTECTION" },
  C: { name: "\u8B49\u660E\u6A19\u6E96", nameEn: "Power-On Waveform", circuit: "\u4E0A\u96FB\u6CE2\u5F62", question: "\u4E0A\u96FB\u6CE2\u5F62\u6709\u6C92\u6709\u51FA\uFF1F", doc: "CodeQC_v3.3_C.md", pillar: "INSTRUMENT+LOGIC_GATE" },
  D: { name: "\u6E2C\u8A66\u53F0", nameEn: "Test Bench", circuit: "\u4E0A\u96FB\u6E2C\u8A66\u53F0", question: "\u6E2C\u8A66\u53F0\u80FD\u4E0D\u80FD\u91CD\u8DD1\uFF1F", doc: "CodeQC_v3.3_D.md", pillar: "WIRING+ALL" }
};
var GATE_CIRCUIT_LABELS = {
  G1: { circuit: "\u8173\u4F4D\u63A5\u89F8\u6E2C\u8A66", en: "Pin Contact Test", tool: "\u842C\u7528\u8868", phase: "\u63A5\u5F97\u4E0A\u55CE\uFF1F", logic: "AND Gate" },
  G2: { circuit: "\u8D70\u7DDA\u9023\u901A\u6E2C\u8A66", en: "Trace Continuity Test", tool: "\u8702\u9CF4\u6A94", phase: "\u901A\u5F97\u4E86\u55CE\uFF1F", logic: "AND Gate" },
  G3: { circuit: "\u4FDD\u8B77\u96FB\u8DEF\u6E2C\u8A66", en: "Protection Circuit Test", tool: "\u6545\u969C\u6CE8\u5165\u5668", phase: "\u6490\u5F97\u4F4F\u55CE\uFF1F", logic: "AND Gate" },
  G4: { circuit: "\u4E0A\u96FB\u91CF\u6E2C", en: "Power-On Measurement", tool: "\u793A\u6CE2\u5668", phase: "\u8DD1\u5F97\u52D5\u55CE\uFF1F", logic: "AND Gate" }
};
var PROTECTION_LAYERS = {
  L1_FUSE: { name: "\u4FDD\u96AA\u7D72", en: "Fuse", behavior: "\u904E\u6D41\u5373\u65B7", maps: "R01-R18" },
  L2_REGULATOR: { name: "\u7A69\u58D3\u5668", en: "Regulator", behavior: "\u9650\u5236\u7BC4\u570D", maps: "P01-P14" },
  L3_GROUND: { name: "\u63A5\u5730", en: "Ground", behavior: "\u57FA\u6E96\u53C3\u8003", maps: "A1-A8" },
  L4_ESD: { name: "ESD\u9632\u8B77", en: "ESD Protection", behavior: "\u9632\u975C\u96FB\u64CA\u7A7F", maps: "Z1-Z4" },
  L5_ANTI_REPLAY: { name: "\u9632\u56DE\u653E", en: "Anti-Replay", behavior: "\u5C01\u6B7B\u5047\u8DD1", maps: "Nonce+Hash+Verifier" }
};
var FAULT_MODES = {
  SHORT_CIRCUIT: { severity: "critical", label: "\u77ED\u8DEF\u2014\u5047\u5BE6\u73FE/Mock\u5192\u5145", rules: ["R13", "R16", "R17", "R18"] },
  OPEN_CIRCUIT: { severity: "critical", label: "\u65B7\u8DEF\u2014TODO/\u672A\u5BE6\u73FE", rules: ["R15"] },
  FAKE_SIGNAL: { severity: "critical", label: "\u507D\u4FE1\u865F\u2014\u62FC\u8CBC\u820A\u8B49\u64DA/\u5047\u8DD1", protection: "L5" },
  COLD_SOLDER_JOINT: { severity: "major", label: "\u865B\u710A\u2014\u7A7Acatch/\u975C\u9ED8\u5931\u6557", rules: ["R05", "R14"] },
  LEAKAGE: { severity: "major", label: "\u6F0F\u96FB\u2014\u7121\u9650\u5236\u8CC7\u6E90/\u6D29\u6F0F", rules: ["R09"] },
  CROSSTALK: { severity: "moderate", label: "\u4E32\u64FE\u2014\u5168\u5C40\u72C0\u614B/\u7DCA\u8026\u5408", rules: ["P07", "P08"] },
  OVERVOLTAGE: { severity: "moderate", label: "\u904E\u58D3\u2014\u8D85\u9577\u51FD\u6578/\u6DF1\u5D4C\u5957", rules: ["P05", "P06", "P10"] },
  NOISE: { severity: "minor", label: "\u566A\u8072\u2014\u9B54\u6CD5\u6578\u5B57/\u547D\u540D\u4E0D\u6E05", rules: ["P04", "P09"] }
};
var CIRCUIT_QUICK_CARD = [
  "Code-QC v3.3 \xB7 \u8EDF\u9AD4\u5DE5\u7A0B\u786C\u9AD4\u5316",
  "\u2776\u8173\u4F4D\u5316: SPEC+type\u5B9A\u7FA9+\u4FE1\u865F\u6D41 (\u6C92\u8173\u4F4D=\u77ED\u8DEF)",
  "\u2777\u8D70\u7DDA\u5316: Pipeline\u56FA\u5B9A\u4E0D\u53EF\u8DF3\u6B65 (\u4E0D\u901A=\u65B7\u8DEF)",
  "\u2778\u9598\u9580\u5316: G1-G4 AND\u5168HIGH\u624D\u958B (\u4E00LOW=\u65B7\u96FB)",
  "\u2779\u91CF\u6E2C\u5316: YXZ\u4E09\u8EF8+evidence/LOG (\u6C92\u6CE2\u5F62=\u6C92\u8DD1)",
  "\u277A\u4FDD\u8B77\u5316: L1\u4FDD\u96AA\u7D72+L2\u7A69\u58D3+L3\u63A5\u5730+L4ESD+L5\u9632\u56DE\u653E (\u5077\u61F6=\u7194\u65B7)",
  "\u4E94\u6BB5\u5168\u904E=MISSION COMPLETE | \u4EFB\u4E00\u7194\u65B7=REJECTED"
].join("\n");
var PROTECTION_LEVELS = {
  LV1: { name: "\u4FDD\u96AA\u7D72\u4FDD\u8B77", nameEn: "Redline Fuse", scope: "All projects", tier: "basic" },
  LV2: { name: "\u7A69\u58D3\u5668\u9650\u5236", nameEn: "Prohibition Regulator", scope: "All projects", tier: "basic" },
  LV3: { name: "\u9632\u8A50\u6B3A\u6383\u63CF", nameEn: "Anti-Fraud ESD", scope: "All projects", tier: "basic" },
  LV4: { name: "\u9632\u56DE\u653E\u9396", nameEn: "Nonce/Challenge", scope: "Multi-model collaboration", tier: "enhanced" },
  LV5: { name: "\u9632\u7BE1\u6539\u5C01\u5370", nameEn: "Hash/Merkle", scope: "Outsourcing acceptance", tier: "enhanced" },
  LV6: { name: "\u7368\u7ACB\u6AA2\u6E2C\u7AD9", nameEn: "Verifier Replay", scope: "High-reliability projects", tier: "independent" },
  LV7: { name: "\u53EF\u4FE1\u6A21\u7D44", nameEn: "Attestation/TEE", scope: "Finance/Medical", tier: "independent" },
  LV8: { name: "\u4EA4\u53C9\u5C0D\u6297", nameEn: "Cross-Model Adversarial", scope: "Deep-tech", tier: "formal" },
  LV9: { name: "\u5F62\u5F0F\u5316\u8B49\u660E", nameEn: "Formal Verification", scope: "Military/Aerospace", tier: "formal" }
};
var PRODUCT_GRADES = {
  E: { name: "\u5546\u7528\u7D1A", nameEn: "Commercial Grade", protection: "LV1-5", gates: "G1-G4", description: "\u4E00\u822C\u5546\u7528\u7522\u54C1" },
  F: { name: "\u6DF1\u79D1\u6280\u7D1A", nameEn: "Deep-Tech Grade", protection: "LV1-9", gates: "G1-G4+Formal", description: "\u91D1\u878D/\u91AB\u7642/\u8ECD\u898F/\u822A\u592A" }
};
var PROTECTION_COMPONENTS = {
  FUSE: { name: "\u4FDD\u96AA\u7D72", nameEn: "Fuse", behavior: "\u904E\u6D41\u5373\u65B7", maps: "Redlines R01-R18", tier: "LV1" },
  REGULATOR: { name: "\u7A69\u58D3\u5668", nameEn: "Regulator", behavior: "\u9650\u5236\u7BC4\u570D", maps: "Prohibitions P01-P14", tier: "LV2" },
  GROUND: { name: "\u63A5\u5730", nameEn: "Ground", behavior: "\u57FA\u6E96\u53C3\u8003", maps: "Axioms A1-A8", tier: "LV1" },
  ESD: { name: "ESD\u9632\u8B77", nameEn: "ESD Protection", behavior: "\u9632\u975C\u96FB\u64CA\u7A7F", maps: "Anti-Fraud Z1-Z5", tier: "LV3" },
  THERMAL: { name: "\u6EAB\u5EA6\u4FDD\u8B77", nameEn: "Thermal Shutdown", behavior: "\u904E\u71B1\u95DC\u65B7", maps: "Handover Tags", tier: "LV1" }
};
var HARDWARE_QUICK_CARD = `
\u2554\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2557
\u2551         Code-QC v3.3  \u8EDF\u9AD4\u5DE5\u7A0B\u786C\u9AD4\u5316  \u901F\u67E5\u5361               \u2551
\u2551         \u4F60\u662F\u65BD\u5DE5\u968A\uFF0C\u4E0D\u662F\u5BEB\u624B\u3002\u63A5\u96FB\u8DEF\uFF0C\u51FA\u6CE2\u5F62\u3002              \u2551
\u2560\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2563
\u2551  \u2776 \u8173\u4F4D\u5316 (A): \u63A5\u53E3\u5B9A\u7FA9+\u985E\u578B\u7C3D\u540D+\u932F\u8AA4\u8173\u4F4D+\u72C0\u614B\u6A5F         \u2551
\u2551  \u2777 \u8D70\u7DDA\u5316 (D): build\u2192lint\u2192test\u2192proof\u2192gate\u2192ship            \u2551
\u2551  \u2778 \u9598\u9580\u5316 (B): R01-18=0 + P01-14\u5728\u9650 + A1-8\u7121\u9055\u53CD        \u2551
\u2551  \u2779 \u91CF\u6E2C\u5316 (C): G1\u842C\u7528\u8868+G2\u8702\u9CF4+G3\u6545\u969C\u6CE8\u5165+G4\u793A\u6CE2\u5668       \u2551
\u2551  \u277A \u904E\u8F09\u4FDD\u8B77: LV1-3\u57FA\u790E + LV4-5\u9632\u507D + LV6-9\u7368\u7ACB\u9A57\u8B49       \u2551
\u2551  DoD: (1)\u5BE6\u73FE(2)\u88DC\u5B8C(3)\u898F\u683C(4)\u540C\u6B65(5)\u7DE8\u8B6F(6)\u4EA4\u4ED8(7)\u771F\u5BE6(8)\u9632\u8A50 \u2551
\u2551  \u5168\u904E=MISSION COMPLETE  \u4EFB\u4E00\u4E0D\u904E=REJECTED                  \u2551
\u255A\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u255D
`;

// src/types.ts
var CODEQC_VERSION = "3.3";
var DEFAULT_CONFIG = {
  level: "D",
  include: SUPPORTED_EXTENSIONS.map((ext) => `**/*${ext}`),
  exclude: ["**/node_modules/**", "**/dist/**", "**/build/**", "**/.git/**", "**/vendor/**"],
  reporter: "console",
  ci: false
};

// src/rules/b-axioms.ts
var AXIOMS = [
  {
    id: "A1",
    category: "axiom",
    name: "\u5B8C\u6574\u4EA4\u4ED8",
    nameEn: "Complete Delivery",
    description: "\u6BCF\u500B\u4EFB\u52D9\u5FC5\u9808\u5B8C\u6574\u5B8C\u6210\uFF0C\u4E0D\u7559\u534A\u6210\u54C1",
    severity: "error",
    action: "\u56DE\u6EFE\u91CD\u505A",
    autoDetectable: false,
    detectMethod: "manual",
    priority: 2
  },
  {
    id: "A2",
    category: "axiom",
    name: "\u96F6\u6280\u8853\u50B5",
    nameEn: "Zero Tech Debt",
    description: "\u4E0D\u5F15\u5165\u5DF2\u77E5\u7684\u6280\u8853\u50B5\u52D9",
    severity: "error",
    action: "\u7ACB\u5373\u4FEE\u5FA9",
    autoDetectable: false,
    detectMethod: "manual",
    priority: 3
  },
  {
    id: "A3",
    category: "axiom",
    name: "\u53EF\u8FFD\u6EAF",
    nameEn: "Traceability",
    description: "\u6240\u6709\u8B8A\u66F4\u5FC5\u9808\u53EF\u8FFD\u6EAF\u5230\u9700\u6C42",
    severity: "warning",
    action: "\u88DC\u5145\u8A18\u9304",
    autoDetectable: false,
    detectMethod: "manual",
    priority: 6
  },
  {
    id: "A4",
    category: "axiom",
    name: "\u53EF\u6E2C\u8A66",
    nameEn: "Testability",
    description: "\u6240\u6709\u4EE3\u78BC\u5FC5\u9808\u53EF\u88AB\u6E2C\u8A66",
    severity: "error",
    action: "\u91CD\u69CB",
    autoDetectable: false,
    detectMethod: "manual",
    priority: 4
  },
  {
    id: "A5",
    category: "axiom",
    name: "\u53EF\u7DAD\u8B77",
    nameEn: "Maintainability",
    description: "\u4EE3\u78BC\u5FC5\u9808\u6613\u65BC\u7406\u89E3\u548C\u4FEE\u6539",
    severity: "warning",
    action: "\u91CD\u69CB",
    autoDetectable: false,
    detectMethod: "manual",
    priority: 5
  },
  {
    id: "A6",
    category: "axiom",
    name: "\u5B89\u5168\u512A\u5148",
    nameEn: "Security First",
    description: "\u5B89\u5168\u8003\u91CF\u512A\u5148\u65BC\u529F\u80FD\u548C\u6027\u80FD",
    severity: "error",
    action: "\u7ACB\u5373\u4FEE\u5FA9",
    autoDetectable: false,
    detectMethod: "manual",
    priority: 1
    // 最高優先
  },
  {
    id: "A7",
    category: "axiom",
    name: "\u6587\u6A94\u540C\u6B65",
    nameEn: "Doc Sync",
    description: "\u6587\u6A94\u8207\u4EE3\u78BC\u4FDD\u6301\u540C\u6B65",
    severity: "warning",
    action: "\u7ACB\u5373\u66F4\u65B0",
    autoDetectable: false,
    detectMethod: "manual",
    priority: 7
  },
  {
    id: "A8",
    category: "axiom",
    name: "\u6301\u7E8C\u6539\u9032",
    nameEn: "Continuous Improvement",
    description: "\u6BCF\u6B21\u8FED\u4EE3\u90FD\u8981\u6709\u6539\u9032",
    severity: "info",
    action: "\u56DE\u9867\u5206\u6790",
    autoDetectable: false,
    detectMethod: "manual",
    priority: 8
  }
];
var AXIOMS_BY_PRIORITY = [...AXIOMS].sort((a, b) => a.priority - b.priority);
function getAxiom(id) {
  return AXIOMS.find((a) => a.id === id);
}
function formatAxiomsPrompt() {
  const lines = [
    "# Code-QC v2.4 \u516B\u516C\u7406",
    "",
    "\u512A\u5148\u7D1A\uFF1AA6 > A1 > A2 > A4 > A5 > A3 > A7 > A8",
    ""
  ];
  for (const axiom of AXIOMS_BY_PRIORITY) {
    lines.push(`## ${axiom.id} ${axiom.name} (${axiom.nameEn})`);
    lines.push(`- \u8AAA\u660E\uFF1A${axiom.description}`);
    lines.push(`- \u9055\u53CD\u5F8C\u679C\uFF1A${axiom.action}`);
    lines.push("");
  }
  return lines.join("\n");
}

// src/rules/b-redlines-defs.ts
var REDLINES = [
  { id: "R01", category: "redline", name: "\u786C\u7DE8\u78BC\u6191\u8B49", nameEn: "Hardcoded Credentials", description: "\u4EE3\u78BC\u4E2D\u786C\u7DE8\u78BC\u5BC6\u78BC\u3001\u5BC6\u9470\u3001Token", severity: "error", action: "\u{1F534} \u7ACB\u5373\u522A\u9664\uFF0C\u8F2A\u63DB", autoDetectable: true, detectMethod: "regex", implemented: true },
  { id: "R02", category: "redline", name: "\u8DF3\u904E\u5B89\u5168\u6AA2\u67E5", nameEn: "Bypass Security", description: "\u7E5E\u904E\u8A8D\u8B49\u3001\u6388\u6B0A\u3001\u8F38\u5165\u9A57\u8B49", severity: "error", action: "\u{1F534} \u56DE\u6EFE\uFF0C\u5BE9\u67E5", autoDetectable: true, detectMethod: "regex", implemented: true },
  { id: "R03", category: "redline", name: "\u522A\u9664\u5BE9\u8A08\u65E5\u8A8C", nameEn: "Delete Audit Logs", description: "\u522A\u9664\u6216\u7BE1\u6539\u5BE9\u8A08\u8A18\u9304", severity: "error", action: "\u{1F534} \u7ACB\u5373\u6062\u5FA9", autoDetectable: true, detectMethod: "regex", implemented: true },
  { id: "R04", category: "redline", name: "\u672A\u6388\u6B0A\u6578\u64DA\u8A2A\u554F", nameEn: "Unauthorized Data Access", description: "\u8A2A\u554F\u8D85\u51FA\u6B0A\u9650\u7684\u6578\u64DA", severity: "error", action: "\u{1F534} \u64A4\u92B7\uFF0C\u5BE9\u67E5", autoDetectable: false, detectMethod: "llm", implemented: false, requiresIntegration: "LLM \u8A9E\u7FA9\u5206\u6790" },
  { id: "R05", category: "redline", name: "\u5FFD\u7565\u932F\u8AA4\u8655\u7406", nameEn: "Ignore Error Handling", description: "\u7A7A catch\u3001\u541E\u7570\u5E38", severity: "error", action: "\u{1F534} \u7ACB\u5373\u4FEE\u5FA9", autoDetectable: true, detectMethod: "regex", implemented: true },
  { id: "R06", category: "redline", name: "\u76F4\u63A5\u64CD\u4F5C\u751F\u7522", nameEn: "Direct Production Access", description: "\u672A\u7D93\u5BE9\u6279\u4FEE\u6539\u751F\u7522\u74B0\u5883", severity: "error", action: "\u{1F534} \u56DE\u6EFE", autoDetectable: false, detectMethod: "integration", implemented: false, requiresIntegration: "CI/CD \u7CFB\u7D71" },
  { id: "R07", category: "redline", name: "\u95DC\u9589\u5B89\u5168\u529F\u80FD", nameEn: "Disable Security", description: "\u95DC\u9589\u9632\u706B\u7246\u3001TLS\u3001\u52A0\u5BC6", severity: "error", action: "\u{1F534} \u7ACB\u5373\u6062\u5FA9", autoDetectable: true, detectMethod: "regex", implemented: true },
  { id: "R08", category: "redline", name: "\u4F7F\u7528\u5DF2\u77E5\u6F0F\u6D1E", nameEn: "Known Vulnerabilities", description: "\u4F7F\u7528\u6709\u6F0F\u6D1E\u7684\u4F9D\u8CF4\u7248\u672C", severity: "error", action: "\u{1F534} \u7ACB\u5373\u5347\u7D1A", autoDetectable: true, detectMethod: "regex", implemented: true },
  { id: "R09", category: "redline", name: "\u7121\u9650\u5236\u8CC7\u6E90", nameEn: "Unlimited Resources", description: "\u7121\u9650\u5236 API \u8ABF\u7528\u3001\u67E5\u8A62\u3001\u5FAA\u74B0", severity: "error", action: "\u{1F534} \u6DFB\u52A0\u9650\u5236", autoDetectable: true, detectMethod: "regex", implemented: true },
  { id: "R10", category: "redline", name: "\u660E\u6587\u50B3\u8F38\u654F\u611F", nameEn: "Plaintext Sensitive Data", description: "\u660E\u6587\u50B3\u8F38\u5BC6\u78BC\u3001PII\u3001\u8CA1\u52D9", severity: "error", action: "\u{1F534} \u52A0\u5BC6\u50B3\u8F38", autoDetectable: true, detectMethod: "regex", implemented: true },
  { id: "R11", category: "redline", name: "\u8DF3\u904E\u4EE3\u78BC\u5BE9\u67E5", nameEn: "Skip Code Review", description: "\u672A\u5BE9\u67E5\u4EE3\u78BC\u9032\u5165\u4E3B\u5206\u652F", severity: "error", action: "\u{1F534} \u56DE\u6EFE\uFF0C\u88DC\u5BE9\u67E5", autoDetectable: false, detectMethod: "integration", implemented: false, requiresIntegration: "Git/VCS \u7CFB\u7D71" },
  { id: "R12", category: "redline", name: "\u507D\u9020\u6E2C\u8A66\u7D50\u679C", nameEn: "Fake Test Results", description: "\u507D\u9020\u3001\u8DF3\u904E\u3001\u786C\u7DE8\u78BC\u6E2C\u8A66", severity: "error", action: "\u{1F534} \u91CD\u65B0\u6E2C\u8A66", autoDetectable: true, detectMethod: "regex", implemented: true },
  { id: "R13", category: "redline", name: "\u5047\u5BE6\u73FE", nameEn: "Fake Implementation", description: "return true/null/\u7A7A\u7269\u4EF6\u7121\u5BE6\u969B\u908F\u8F2F", severity: "error", action: "\u{1F534} \u7ACB\u5373\u91CD\u5BEB", autoDetectable: true, detectMethod: "regex", implemented: true },
  { id: "R14", category: "redline", name: "\u975C\u9ED8\u5931\u6557", nameEn: "Silent Failure", description: "catch \u4E0D log \u4E5F\u4E0D re-throw", severity: "error", action: "\u{1F534} \u7ACB\u5373\u4FEE\u5FA9", autoDetectable: true, detectMethod: "regex", implemented: true },
  { id: "R15", category: "redline", name: "TODO\u6B98\u7559", nameEn: "TODO Residue", description: "todo!/unimplemented!/TODO \u9032\u5165\u63D0\u4EA4", severity: "error", action: "\u{1F534} \u7ACB\u5373\u6E05\u9664", autoDetectable: true, detectMethod: "regex", implemented: true },
  { id: "R16", category: "redline", name: "\u7A7A\u65B9\u6CD5", nameEn: "Empty Method", description: "\u65B9\u6CD5\u7C3D\u540D\u6B63\u78BA\u4F46\u65B9\u6CD5\u9AD4\u7A7A\u6216\u50C5return\u9ED8\u8A8D\u503C", severity: "error", action: "\u{1F534} \u780D\u6389\u91CD\u5BEB", autoDetectable: true, detectMethod: "regex", implemented: true },
  { id: "R17", category: "redline", name: "\u8A50\u6B3A\u7269\u4EF6", nameEn: "Fraud Object", description: "\u7269\u4EF6\u7D50\u69CB\u6B63\u78BA\u4F46\u6578\u64DA\u786C\u7DE8\u78BC/\u4E0D\u4F86\u81EA\u771F\u5BE6\u6578\u64DA\u6E90", severity: "error", action: "\u{1F534} \u780D\u6389\u91CD\u5BEB", autoDetectable: true, detectMethod: "regex", implemented: true },
  { id: "R18", category: "redline", name: "\u7E5E\u9053\u5BE6\u4F5C", nameEn: "Bypass Implementation", description: "\u8DF3\u904E\u61C9\u4F7F\u7528\u7684API/DB/Config\u7528\u5047\u8CC7\u6599\u66FF\u4EE3", severity: "error", action: "\u{1F534} \u780D\u6389\u91CD\u5BEB", autoDetectable: true, detectMethod: "regex", implemented: true }
];
function getRedline(id) {
  return REDLINES.find((r) => r.id === id);
}

// src/rules/b-redlines-utils.ts
function maskJsStringsAndComments(source) {
  const chars = source.split("");
  let state = "normal";
  for (let i = 0; i < chars.length; i++) {
    const c = chars[i];
    const n = chars[i + 1];
    if (state === "normal") {
      if (c === "/" && n === "/") {
        chars[i] = " ";
        chars[i + 1] = " ";
        i++;
        state = "line_comment";
        continue;
      }
      if (c === "/" && n === "*") {
        chars[i] = " ";
        chars[i + 1] = " ";
        i++;
        state = "block_comment";
        continue;
      }
      if (c === "'") {
        state = "single";
        continue;
      }
      if (c === '"') {
        state = "double";
        continue;
      }
      if (c === "`") {
        state = "template";
        continue;
      }
      continue;
    }
    if (state === "line_comment") {
      if (c === "\n") {
        state = "normal";
        continue;
      }
      if (c !== "\r") chars[i] = " ";
      continue;
    }
    if (state === "block_comment") {
      if (c === "*" && n === "/") {
        chars[i] = " ";
        chars[i + 1] = " ";
        i++;
        state = "normal";
        continue;
      }
      if (c !== "\n") chars[i] = " ";
      continue;
    }
    if (state === "single") {
      if (c === "\\") {
        chars[i] = " ";
        if (n && n !== "\n") chars[i + 1] = " ";
        i++;
        continue;
      }
      if (c === "\n") {
        state = "normal";
        continue;
      }
      if (c === "'") {
        state = "normal";
        continue;
      }
      chars[i] = " ";
      continue;
    }
    if (state === "double") {
      if (c === "\\") {
        chars[i] = " ";
        if (n && n !== "\n") chars[i + 1] = " ";
        i++;
        continue;
      }
      if (c === "\n") {
        state = "normal";
        continue;
      }
      if (c === '"') {
        state = "normal";
        continue;
      }
      chars[i] = " ";
      continue;
    }
    if (c === "\\") {
      chars[i] = " ";
      if (n && n !== "\n") chars[i + 1] = " ";
      i++;
      continue;
    }
    if (c === "`") {
      state = "normal";
      continue;
    }
    if (c !== "\n") chars[i] = " ";
  }
  return chars.join("");
}
function maskRustStringsAndComments(source) {
  const chars = source.split("");
  let state = "normal";
  let blockDepth = 0;
  let rawHashes = 0;
  for (let i = 0; i < chars.length; i++) {
    const c = chars[i];
    const n = chars[i + 1];
    if (state === "normal") {
      if (c === "/" && n === "/") {
        chars[i] = " ";
        chars[i + 1] = " ";
        i++;
        state = "line_comment";
        continue;
      }
      if (c === "/" && n === "*") {
        chars[i] = " ";
        chars[i + 1] = " ";
        i++;
        blockDepth = 1;
        state = "block_comment";
        continue;
      }
      if (c === "r" || c === "b" && n === "r") {
        let j = i;
        if (c === "b") j += 2;
        else j += 1;
        let hashes = 0;
        while (chars[j] === "#") {
          hashes++;
          j++;
        }
        if (chars[j] === '"') {
          rawHashes = hashes;
          state = "raw_string";
          i = j;
          continue;
        }
      }
      if (c === "b" && n === '"') {
        state = "string";
        i = i + 1;
        continue;
      }
      if (c === '"') {
        state = "string";
        continue;
      }
      continue;
    }
    if (state === "line_comment") {
      if (c === "\n") {
        state = "normal";
        continue;
      }
      if (c !== "\r") chars[i] = " ";
      continue;
    }
    if (state === "block_comment") {
      if (c === "/" && n === "*") {
        chars[i] = " ";
        chars[i + 1] = " ";
        i++;
        blockDepth++;
        continue;
      }
      if (c === "*" && n === "/") {
        chars[i] = " ";
        chars[i + 1] = " ";
        i++;
        blockDepth--;
        if (blockDepth === 0) state = "normal";
        continue;
      }
      if (c !== "\n") chars[i] = " ";
      continue;
    }
    if (state === "string") {
      if (c === "\\") {
        chars[i] = " ";
        if (n && n !== "\n") chars[i + 1] = " ";
        i++;
        continue;
      }
      if (c === '"') {
        state = "normal";
        continue;
      }
      if (c !== "\n") chars[i] = " ";
      continue;
    }
    if (c === '"') {
      let ok = true;
      for (let k = 0; k < rawHashes; k++) {
        if (chars[i + 1 + k] !== "#") {
          ok = false;
          break;
        }
      }
      if (ok) {
        state = "normal";
        i = i + rawHashes;
        continue;
      }
    }
    if (c !== "\n") chars[i] = " ";
  }
  return chars.join("");
}
function stripRustCfgTestBlocks(source) {
  const normalized = source.replace(/\r\n/g, "\n");
  const masked = maskRustStringsAndComments(normalized);
  const out = normalized.split("");
  const cfgRe = /#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]/g;
  let match;
  while ((match = cfgRe.exec(masked)) !== null) {
    const start = match.index;
    const openBrace = masked.indexOf("{", cfgRe.lastIndex);
    if (openBrace === -1) continue;
    let depth = 0;
    let end = -1;
    for (let i = openBrace; i < masked.length; i++) {
      const ch = masked[i];
      if (ch === "{") depth++;
      else if (ch === "}") {
        depth--;
        if (depth === 0) {
          end = i + 1;
          break;
        }
      }
    }
    if (end === -1) continue;
    for (let i = start; i < end; i++) {
      if (out[i] !== "\n") out[i] = " ";
    }
    cfgRe.lastIndex = end;
  }
  return out.join("");
}

// src/rules/b-redlines-r01-r05.ts
var CREDENTIAL_PATTERNS = [
  /(?:password|passwd|pwd)\s*[=:]\s*['"`](?![\s'"`])[^'"`]{3,}/gi,
  /(?:api[_-]?key|apikey)\s*[=:]\s*['"`](?![\s'"`])[^'"`]{8,}/gi,
  /(?:secret|token|auth)\s*[=:]\s*['"`](?![\s'"`])[^'"`]{8,}/gi,
  /(?:aws[_-]?(?:access[_-]?key|secret))\s*[=:]\s*['"`][A-Za-z0-9+/]{20,}/gi,
  /-----BEGIN\s+(?:RSA\s+)?PRIVATE\s+KEY-----/gi,
  /bearer\s+[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_]+/gi
];
var CREDENTIAL_WHITELIST = [/process\.env\./, /os\.environ/, /env\s*\(/, /getenv\s*\(/, /\$\{[A-Z_]+\}/, /\$[A-Z_]+/];
var R01_CHECKER = {
  rule: getRedline("R01"),
  checkSource(source, file) {
    const violations = [];
    const lines = source.split("\n");
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      if (/^\s*(?:\/\/|#|\/\*|\*|--|;)/.test(line)) continue;
      if (CREDENTIAL_WHITELIST.some((p) => p.test(line))) continue;
      for (const pattern of CREDENTIAL_PATTERNS) {
        pattern.lastIndex = 0;
        const match = pattern.exec(line);
        if (match) {
          violations.push({ ruleId: "R01", ruleName: "\u786C\u7DE8\u78BC\u6191\u8B49", severity: "error", file, line: i + 1, column: match.index + 1, message: "\u6AA2\u6E2C\u5230\u786C\u7DE8\u78BC\u6191\u8B49", snippet: line.trim(), suggestion: "\u4F7F\u7528\u74B0\u5883\u8B8A\u6578\u6216\u5BC6\u9470\u7BA1\u7406\u670D\u52D9" });
          break;
        }
      }
    }
    return violations;
  }
};
var BYPASS_SECURITY_PATTERNS = [
  /(?:execute|query|raw)\s*\(\s*[`'"].*\$\{/gi,
  /(?:execute|query)\s*\(\s*['"`].*\+\s*\w+/gi,
  /f['"]SELECT.*\{/gi,
  /(?:skip|bypass|disable)[_-]?(?:auth|validation|verification|check)\s*[=:]\s*true/gi,
  /isAdmin\s*[=:]\s*true/gi,
  /\beval\s*\(/gi,
  /new\s+Function\s*\(/gi,
  /pickle\.loads?\s*\(/gi,
  /yaml\.(?:unsafe_)?load\s*\(/gi,
  /ObjectInputStream/gi,
  /BinaryFormatter/gi
];
var R02_CHECKER = {
  rule: getRedline("R02"),
  checkSource(source, file) {
    const violations = [];
    const lines = source.split("\n");
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      if (/^\s*(?:\/\/|#|\/\*|\*|--|;)/.test(line)) continue;
      if (/^\s*\/.*\/[gimsuy]*\s*,?\s*$/.test(line)) continue;
      for (const pattern of BYPASS_SECURITY_PATTERNS) {
        pattern.lastIndex = 0;
        const match = pattern.exec(line);
        if (match) {
          violations.push({ ruleId: "R02", ruleName: "\u8DF3\u904E\u5B89\u5168\u6AA2\u67E5", severity: "error", file, line: i + 1, column: match.index + 1, message: `\u6AA2\u6E2C\u5230\u5B89\u5168\u7E5E\u904E: ${match[0].substring(0, 30)}`, snippet: line.trim(), suggestion: "\u4F7F\u7528\u53C3\u6578\u5316\u67E5\u8A62\u3001\u6B63\u78BA\u9A57\u8B49" });
          break;
        }
      }
    }
    return violations;
  }
};
var DELETE_AUDIT_PATTERNS = [
  // Use word boundaries to avoid false positives like "ViewModel ... Dialog..." (del + log substrings).
  /\b(?:rm|del|remove|unlink)\b\s+.*\b(?:log|audit|trace)\b/gi,
  /\.(?:delete|remove|unlink)\s*\([^)]*\b(?:log|audit)\b/gi,
  /os\.(?:remove|unlink)\s*\([^)]*\b(?:log|audit)\b/gi,
  /fs\.(?:unlink|rm)Sync?\s*\([^)]*\b(?:log|audit)\b/gi,
  /truncate.*\b(?:log|audit)\b/gi,
  />\s*\/(?:var\/)?log\//gi,
  /UPDATE\s+.*\b(?:audit|log)(?:[_-]\w+)?\b.*SET/gi,
  /DELETE\s+FROM\s+.*\b(?:audit|log)(?:[_-]\w+)?\b/gi
];
var R03_CHECKER = {
  rule: getRedline("R03"),
  checkSource(source, file) {
    const violations = [];
    const lines = source.split("\n");
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      if (/^\s*(?:\/\/|#|\/\*|\*|--|;)/.test(line)) continue;
      if (/^\s*\/.*\/[gimsuy]*\s*,?\s*$/.test(line)) continue;
      for (const pattern of DELETE_AUDIT_PATTERNS) {
        pattern.lastIndex = 0;
        const match = pattern.exec(line);
        if (match) {
          violations.push({ ruleId: "R03", ruleName: "\u522A\u9664\u5BE9\u8A08\u65E5\u8A8C", severity: "error", file, line: i + 1, column: match.index + 1, message: "\u6AA2\u6E2C\u5230\u5BE9\u8A08\u65E5\u8A8C\u64CD\u4F5C", snippet: line.trim(), suggestion: "\u5BE9\u8A08\u65E5\u8A8C\u61C9\u53EA\u8B80" });
          break;
        }
      }
    }
    return violations;
  }
};
var EMPTY_CATCH_PATTERNS = {
  typescript: [/catch\s*\([^)]*\)\s*\{\s*\}/g, /\.catch\s*\(\s*\(\s*\w*\s*\)\s*=>\s*\{\s*\}\s*\)/g],
  javascript: [/catch\s*\([^)]*\)\s*\{\s*\}/g, /\.catch\s*\(\s*\(\s*\)\s*=>\s*\{\s*\}\s*\)/g],
  python: [/except\s*:\s*pass/g, /except\s+\w+\s*:\s*pass/g, /except\s+\w+\s+as\s+\w+\s*:\s*pass/g],
  rust: [/\.unwrap\(\)/g],
  go: [/if\s+err\s*!=\s*nil\s*\{\s*\}/g, /_\s*,?\s*=\s*\w+\([^)]*\)/g]
};
var R05_CHECKER = {
  rule: getRedline("R05"),
  checkSource(source, file) {
    const violations = [];
    const ext = file.split(".").pop()?.toLowerCase();
    const langMap = { ts: "typescript", tsx: "typescript", js: "javascript", jsx: "javascript", py: "python", rs: "rust", go: "go" };
    const lang = langMap[ext || ""];
    if (!lang) return violations;
    const patterns = EMPTY_CATCH_PATTERNS[lang] || [];
    const normalized = source.replace(/\r\n/g, "\n");
    for (const pattern of patterns) {
      pattern.lastIndex = 0;
      let match;
      while ((match = pattern.exec(normalized)) !== null) {
        const before = normalized.substring(0, match.index);
        const lineNum = (before.match(/\n/g) || []).length + 1;
        violations.push({ ruleId: "R05", ruleName: "\u5FFD\u7565\u932F\u8AA4\u8655\u7406", severity: "error", file, line: lineNum, column: 1, message: "\u6AA2\u6E2C\u5230\u5FFD\u7565\u932F\u8AA4\u8655\u7406", snippet: match[0].substring(0, 40), suggestion: "\u6DFB\u52A0\u932F\u8AA4\u8655\u7406\u908F\u8F2F" });
      }
    }
    return violations;
  }
};

// src/rules/b-redlines-r07-r12.ts
var SECURITY_DISABLE_PATTERNS = [
  /(?:verify|ssl|tls)[_-]?(?:ssl|verify|cert)\s*[=:]\s*(?:false|False|FALSE|0)/gi,
  /NODE_TLS_REJECT_UNAUTHORIZED\s*=\s*['"]?0/gi,
  /rejectUnauthorized\s*:\s*false/gi,
  /Access-Control-Allow-Origin['"]\s*:\s*['"]\*/gi,
  /Content-Security-Policy['"]\s*:\s*['"]?none/gi,
  /DEBUG\s*=\s*(?:true|True|TRUE|1)/gi
];
var R07_CHECKER = {
  rule: getRedline("R07"),
  checkSource(source, file) {
    const violations = [];
    const lines = source.split("\n");
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      for (const pattern of SECURITY_DISABLE_PATTERNS) {
        pattern.lastIndex = 0;
        const match = pattern.exec(line);
        if (match) {
          violations.push({ ruleId: "R07", ruleName: "\u95DC\u9589\u5B89\u5168\u529F\u80FD", severity: "error", file, line: i + 1, column: match.index + 1, message: `\u95DC\u9589\u5B89\u5168\u529F\u80FD: ${match[0]}`, snippet: line.trim(), suggestion: "\u78BA\u4FDD\u751F\u7522\u555F\u7528\u5B89\u5168\u529F\u80FD" });
          break;
        }
      }
    }
    return violations;
  }
};
var VULNERABLE_PATTERNS = [
  /\bgets\s*\(/g,
  /\bstrcpy\s*\(/g,
  /\bsprintf\s*\(/g,
  /\bstrcat\s*\(/g,
  /log4j.*2\.(?:0|1[0-4])\./gi,
  /(?:MD5|SHA1)\s*\(/gi,
  /\.createHash\s*\(\s*['"](?:md5|sha1)['"]\s*\)/gi,
  /\bDES\b|\bRC4\b|\b3DES\b/g
  // 完整單詞匹配
];
var R08_CHECKER = {
  rule: getRedline("R08"),
  checkSource(source, file) {
    const violations = [];
    const lines = source.split("\n");
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      if (/^\s*(?:\/\/|#|\/\*|\*|--|;)/.test(line)) continue;
      for (const pattern of VULNERABLE_PATTERNS) {
        pattern.lastIndex = 0;
        const match = pattern.exec(line);
        if (match) {
          violations.push({ ruleId: "R08", ruleName: "\u4F7F\u7528\u5DF2\u77E5\u6F0F\u6D1E", severity: "error", file, line: i + 1, column: match.index + 1, message: `\u5DF2\u77E5\u6F0F\u6D1E: ${match[0]}`, snippet: line.trim(), suggestion: "\u5347\u7D1A\u6216\u4F7F\u7528\u5B89\u5168\u66FF\u4EE3" });
          break;
        }
      }
    }
    return violations;
  }
};
var UNLIMITED_RESOURCE_PATTERNS = [
  /while\s*\(\s*true\s*\)/gi,
  /while\s*\(\s*1\s*\)/gi,
  /for\s*\(\s*;\s*;\s*\)/gi,
  /loop\s*\{/gi,
  /SELECT\s+\*\s+FROM(?!.*LIMIT)/gi,
  /\.find\s*\(\s*\{\s*\}\s*\)(?!.*limit)/gi,
  /rate[_-]?limit\s*[=:]\s*(?:0|false|none|null|nil)/gi
];
var R09_CHECKER = {
  rule: getRedline("R09"),
  checkSource(source, file) {
    const violations = [];
    const lines = source.split("\n");
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      if (/^\s*(?:\/\/|#|\/\*|\*|--|;)/.test(line)) continue;
      for (const pattern of UNLIMITED_RESOURCE_PATTERNS) {
        pattern.lastIndex = 0;
        const match = pattern.exec(line);
        if (match) {
          violations.push({ ruleId: "R09", ruleName: "\u7121\u9650\u5236\u8CC7\u6E90", severity: "error", file, line: i + 1, column: match.index + 1, message: `\u7121\u9650\u5236\u8CC7\u6E90: ${match[0]}`, snippet: line.trim(), suggestion: "\u6DFB\u52A0\u9650\u5236/\u8D85\u6642" });
          break;
        }
      }
    }
    return violations;
  }
};
var PLAINTEXT_PATTERNS = [
  /['"`]http:\/\/[^'"`]+(?:login|auth|password|token|secret|api|user)[^'"`]*['"`]/gi,
  /['"`]ftp:\/\//gi,
  /['"`]telnet:\/\//gi
];
var R10_CHECKER = {
  rule: getRedline("R10"),
  checkSource(source, file) {
    const violations = [];
    if (/(?:^|[\\/])tests?(?:$|[\\/])/i.test(file) || /(?:^|[\\/])[^\\/]*\.tests(?:$|[\\/])/i.test(file)) {
      return violations;
    }
    const lines = source.split("\n");
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      if (/^\s*(?:\/\/|#|\/\*|\*|--|;)/.test(line)) continue;
      for (const pattern of PLAINTEXT_PATTERNS) {
        pattern.lastIndex = 0;
        const match = pattern.exec(line);
        if (match) {
          violations.push({ ruleId: "R10", ruleName: "\u660E\u6587\u50B3\u8F38\u654F\u611F", severity: "error", file, line: i + 1, column: match.index + 1, message: "\u660E\u6587\u50B3\u8F38", snippet: line.trim(), suggestion: "\u4F7F\u7528 HTTPS/TLS" });
          break;
        }
      }
    }
    return violations;
  }
};
var FAKE_TEST_PATTERNS = [
  /(?:assert|expect|should)\s*\(\s*true\s*\)/gi,
  /(?:assert|expect)\s*\.\s*(?:equal|toBe)\s*\(\s*true\s*,\s*true\s*\)/gi,
  /(?:it|test|describe)\s*\.\s*skip\s*\(/gi,
  /(?:@skip|@ignore|@disabled)/gi,
  /pytest\.skip\s*\(/gi,
  /@pytest\.mark\.skip/gi
];
var R12_CHECKER = {
  rule: getRedline("R12"),
  checkSource(source, file) {
    if (!/(?:^test_|_test|\.test|\.spec|test\.|spec\.)[^/]+$/i.test(file)) return [];
    const violations = [];
    const lines = source.split("\n");
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      for (const pattern of FAKE_TEST_PATTERNS) {
        pattern.lastIndex = 0;
        const match = pattern.exec(line);
        if (match) {
          violations.push({ ruleId: "R12", ruleName: "\u507D\u9020\u6E2C\u8A66\u7D50\u679C", severity: "error", file, line: i + 1, column: match.index + 1, message: `\u507D\u9020\u6E2C\u8A66: ${match[0]}`, snippet: line.trim(), suggestion: "\u79FB\u9664\u8DF3\u904E\u6216\u4FEE\u6B63\u65B7\u8A00" });
          break;
        }
      }
    }
    return violations;
  }
};

// src/rules/b-redlines-r13-r18.ts
var FAKE_IMPL_PATTERNS = [
  /\btodo!\b/g,
  /\bunimplemented!\b/g,
  /throw\s+new\s+NotImplementedException/gi,
  /raise\s+NotImplementedError/gi
];
var R13_CHECKER = {
  rule: getRedline("R13"),
  checkSource(source, file) {
    const violations = [];
    const lines = source.split("\n");
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      if (/^\s*(?:\/\/|#|\/\*|\*|--|;)/.test(line)) continue;
      for (const pattern of FAKE_IMPL_PATTERNS) {
        pattern.lastIndex = 0;
        const match = pattern.exec(line);
        if (match) {
          violations.push({ ruleId: "R13", ruleName: "\u5047\u5BE6\u73FE", severity: "error", file, line: i + 1, column: match.index + 1, message: `\u5047\u5BE6\u73FE: ${match[0]}`, snippet: line.trim(), suggestion: "\u66FF\u63DB\u70BA\u771F\u5BE6\u696D\u52D9\u908F\u8F2F" });
          break;
        }
      }
    }
    return violations;
  }
};
var SILENT_FAIL_PATTERNS = [
  /catch\s*\([^)]*\)\s*\{\s*\}/g,
  /\.catch\s*\(\s*\(\s*\w*\s*\)\s*=>\s*\{\s*\}\s*\)/g,
  /except\s*:\s*pass/g,
  /except\s+\w+\s*:\s*pass/g,
  /if\s+err\s*!=\s*nil\s*\{\s*\}/g
];
var R14_CHECKER = {
  rule: getRedline("R14"),
  checkSource(source, file) {
    const violations = [];
    const normalized = source.replace(/\r\n/g, "\n");
    for (const pattern of SILENT_FAIL_PATTERNS) {
      pattern.lastIndex = 0;
      let match;
      while ((match = pattern.exec(normalized)) !== null) {
        const before = normalized.substring(0, match.index);
        const lineNum = (before.match(/\n/g) || []).length + 1;
        violations.push({ ruleId: "R14", ruleName: "\u975C\u9ED8\u5931\u6557", severity: "error", file, line: lineNum, column: 1, message: "\u975C\u9ED8\u5931\u6557: catch \u4E0D log \u4E0D rethrow", snippet: match[0].substring(0, 50), suggestion: "\u6DFB\u52A0 log \u6216 re-throw" });
      }
    }
    return violations;
  }
};
var TODO_PATTERNS = [
  /\/\/\s*TODO\b/gi,
  /\/\/\s*FIXME\b/gi,
  /#\s*TODO\b/gi,
  /#\s*FIXME\b/gi,
  /\/\*\s*TODO\b/gi
];
var R15_CHECKER = {
  rule: getRedline("R15"),
  checkSource(source, file) {
    const violations = [];
    const lines = source.split("\n");
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      for (const pattern of TODO_PATTERNS) {
        pattern.lastIndex = 0;
        const match = pattern.exec(line);
        if (match) {
          violations.push({ ruleId: "R15", ruleName: "TODO\u6B98\u7559", severity: "error", file, line: i + 1, column: match.index + 1, message: `TODO\u6B98\u7559: ${match[0]}`, snippet: line.trim(), suggestion: "\u5BE6\u4F5C\u6216\u79FB\u9664 TODO" });
          break;
        }
      }
    }
    return violations;
  }
};
var EMPTY_METHOD_PATTERNS = {
  typescript: [
    /\b(?:async\s+)?(?:function\s+\w+|(?:get|set)\s+\w+|(?!(?:if|for|while|switch|catch|with)\b)\w+)\s*\([^)]*\)\s*(?::\s*\w+[<[\]>|]*\s*)?\{\s*\}/g,
    /\b(?:async\s+)?(?!(?:if|for|while|switch|catch|with)\b)\w+\s*\([^)]*\)\s*(?::\s*\w+[<[\]>|]*\s*)?\{\s*return\s+(?:null|undefined|false|true|0|''|""|``|\[\]|\{\})\s*;?\s*\}/g
  ],
  python: [
    /def\s+\w+\s*\([^)]*\)\s*(?:->\s*\w+\s*)?:\s*(?:\n\s+)?pass\b/g,
    /def\s+\w+\s*\([^)]*\)\s*(?:->\s*\w+\s*)?:\s*(?:\n\s+)?return\s+(?:None|False|True|0|''|""|\[\]|\{\})\s*$/gm
  ],
  rust: [
    /fn\s+\w+\s*(?:<[^>]*>)?\s*\([^)]*\)\s*(?:->\s*\w+[<[\]>]*\s*)?\{\s*\}/g,
    /fn\s+\w+\s*(?:<[^>]*>)?\s*\([^)]*\)\s*(?:->\s*\w+[<[\]>]*\s*)?\{\s*(?:Ok\(\(\)\)|None|Default::default\(\)|0|true|false|String::new\(\)|Vec::new\(\))\s*\}/g
  ],
  csharp: [
    /(?:public|private|protected|internal)\s+(?:static\s+)?(?:async\s+)?\w+[<[\]>]*\s+\w+\s*\([^)]*\)\s*\{\s*\}/g,
    /(?:public|private|protected|internal)\s+(?:static\s+)?(?:async\s+)?\w+[<[\]>]*\s+\w+\s*\([^)]*\)\s*\{\s*return\s+(?:null|false|true|0|"")\s*;\s*\}/g
  ],
  cpp: [
    /\w+\s+\w+::\w+\s*\([^)]*\)\s*\{\s*\}/g,
    /\w+\s+\w+::\w+\s*\([^)]*\)\s*\{\s*return\s*;\s*\}/g
  ]
};
var R16_CHECKER = {
  rule: getRedline("R16"),
  checkSource(source, file) {
    const violations = [];
    const ext = file.split(".").pop()?.toLowerCase() || "";
    const langMap = { ts: "typescript", tsx: "typescript", js: "typescript", jsx: "typescript", py: "python", rs: "rust", cs: "csharp", cpp: "cpp", h: "cpp" };
    const lang = langMap[ext];
    if (!lang || !EMPTY_METHOD_PATTERNS[lang]) return violations;
    const normalized = source.replace(/\r\n/g, "\n");
    const scanSource = lang === "typescript" ? maskJsStringsAndComments(normalized) : normalized;
    for (const pattern of EMPTY_METHOD_PATTERNS[lang]) {
      pattern.lastIndex = 0;
      let match;
      while ((match = pattern.exec(scanSource)) !== null) {
        const before = scanSource.substring(0, match.index);
        const lineNum = (before.match(/\n/g) || []).length + 1;
        if (/(?:test|spec|mock|fixture)/i.test(file)) continue;
        violations.push({ ruleId: "R16", ruleName: "\u7A7A\u65B9\u6CD5", severity: "error", file, line: lineNum, column: 1, message: "\u7A7A\u65B9\u6CD5 (R16): \u65B9\u6CD5\u9AD4\u70BA\u7A7A\u6216\u50C5return\u9ED8\u8A8D\u503C", snippet: match[0].substring(0, 60), suggestion: "\u5BE6\u4F5C\u771F\u5BE6\u696D\u52D9\u908F\u8F2F\uFF0C\u56DE\u7B54 IAV \u4E94\u554F" });
      }
    }
    return violations;
  }
};
var FRAUD_OBJECT_PATTERNS = [
  /(?:const|let|var)\s+\w+\s*=\s*\{[^}]*(?:name|title|label)\s*:\s*['"`](?:Test|test|Placeholder|placeholder|Dummy|dummy|Fake|fake|Sample|sample|Mock|mock|Default|default|Example|example|Lorem|TODO)[^}]*\}/g,
  /stub\.\w+\s*=/gi,
  /mock\.\w+\s*=/gi,
  /dummy\.\w+\s*=/gi,
  /fake\.\w+\s*=/gi
];
var R17_CHECKER = {
  rule: getRedline("R17"),
  checkSource(source, file) {
    const violations = [];
    if (/(?:test|spec|mock|fixture|__test__|\.test\.|\.spec\.)/i.test(file)) return violations;
    const lines = source.split("\n");
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      if (/^\s*(?:\/\/|#|\/\*|\*|--|;)/.test(line)) continue;
      for (const pattern of FRAUD_OBJECT_PATTERNS) {
        pattern.lastIndex = 0;
        const match = pattern.exec(line);
        if (match) {
          violations.push({ ruleId: "R17", ruleName: "\u8A50\u6B3A\u7269\u4EF6", severity: "error", file, line: i + 1, column: match.index + 1, message: "\u8A50\u6B3A\u7269\u4EF6 (R17): \u786C\u7DE8\u78BC\u5047\u6578\u64DA", snippet: match[0].substring(0, 60), suggestion: "\u4F7F\u7528\u771F\u5BE6\u6578\u64DA\u6E90\uFF0C\u6A19\u6CE8 @DATASOURCE" });
          break;
        }
      }
    }
    return violations;
  }
};
var BYPASS_IMPL_PATTERNS = [
  /\/\/\s*(?:TODO|FIXME|HACK|TEMP)\s*.*(?:從|from|fetch|load|get|query|API|api|service|db|database)/gi,
  /\/\/\s*(?:HARDCODED|BYPASSED|SHORTCUT|STUBBED)/gi,
  /#\s*(?:HARDCODED|BYPASSED|SHORTCUT|STUBBED)/gi,
  /\/\/\s*(?:應該|should)\s*(?:從|from|call|use)/gi
];
var R18_CHECKER = {
  rule: getRedline("R18"),
  checkSource(source, file) {
    const violations = [];
    if (/(?:test|spec|mock|fixture)/i.test(file)) return violations;
    const lines = source.split("\n");
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      for (const pattern of BYPASS_IMPL_PATTERNS) {
        pattern.lastIndex = 0;
        const match = pattern.exec(line);
        if (match) {
          violations.push({ ruleId: "R18", ruleName: "\u7E5E\u9053\u5BE6\u4F5C", severity: "error", file, line: i + 1, column: match.index + 1, message: `\u7E5E\u9053\u5BE6\u4F5C (R18): ${match[0].substring(0, 40)}`, snippet: line.trim(), suggestion: "\u4F7F\u7528\u8A2D\u8A08\u4E0A\u7684\u6578\u64DA\u6E90/API/Service" });
          break;
        }
      }
    }
    return violations;
  }
};

// src/rules/b-redlines.ts
var REDLINE_CHECKERS = [
  R01_CHECKER,
  R02_CHECKER,
  R03_CHECKER,
  R05_CHECKER,
  R07_CHECKER,
  R08_CHECKER,
  R09_CHECKER,
  R10_CHECKER,
  R12_CHECKER,
  R13_CHECKER,
  R14_CHECKER,
  R15_CHECKER,
  R16_CHECKER,
  R17_CHECKER,
  R18_CHECKER
];
var ANTI_FRAUD_CHECKERS = [
  R16_CHECKER,
  R17_CHECKER,
  R18_CHECKER
];
function checkAntifraud(source, file) {
  const ext = file.split(".").pop()?.toLowerCase() || "";
  const scanSource = ext === "rs" ? stripRustCfgTestBlocks(source) : source;
  const violations = [];
  for (const checker of ANTI_FRAUD_CHECKERS) {
    if (checker.checkSource) violations.push(...checker.checkSource(scanSource, file));
  }
  return violations;
}
function checkRedlines(source, file) {
  const ext = file.split(".").pop()?.toLowerCase() || "";
  const scanSource = ext === "rs" ? stripRustCfgTestBlocks(source) : source;
  const violations = [];
  for (const checker of REDLINE_CHECKERS) {
    if (checker.checkSource) violations.push(...checker.checkSource(scanSource, file));
  }
  return violations;
}

// src/rules/b-prohibitions.ts
var THRESHOLDS = {
  /** P03: 重複代碼組數 */
  DUPLICATE_GROUPS: 3,
  /** P03: 最小重複次數 */
  DUPLICATE_MIN_COUNT: 3,
  /** P04: 魔法數字報告上限 */
  MAGIC_NUMBER_MAX_REPORT: 10,
  /** P05: 函數最大行數 */
  FUNCTION_MAX_LINES: 50,
  /** P06: 嵌套最大層數 */
  NESTING_MAX_DEPTH: 3,
  /** P09: 無意義命名報告上限 */
  MEANINGLESS_NAME_MAX_REPORT: 10,
  /** P10: 參數最大數量 */
  PARAM_MAX_COUNT: 5,
  /** P12: 註釋代碼行數閾值 */
  COMMENTED_CODE_THRESHOLD: 10,
  /** P13: TODO 最大數量 */
  TODO_MAX_COUNT: 10,
  /** P14: 生產依賴最大數量 */
  DEPS_MAX_COUNT: 50,
  /** P14: 開發依賴最大數量 */
  DEV_DEPS_MAX_COUNT: 80
};
var PROHIBITIONS = [
  { id: "P01", category: "prohibition", name: "\u904E\u5EA6\u5DE5\u7A0B", nameEn: "Over-Engineering", description: "\u70BA\u4E0D\u5B58\u5728\u7684\u9700\u6C42\u505A\u8A2D\u8A08", severity: "warning", action: "\u7C21\u5316", autoDetectable: false, detectMethod: "llm", implemented: false, requiresIntegration: "LLM \u8A9E\u7FA9\u5206\u6790" },
  { id: "P02", category: "prohibition", name: "\u904E\u65E9\u512A\u5316", nameEn: "Premature Optimization", description: "\u5728\u8B49\u660E\u9700\u8981\u524D\u512A\u5316", severity: "warning", action: "\u79FB\u9664", autoDetectable: false, detectMethod: "llm", implemented: false, requiresIntegration: "LLM \u8A9E\u7FA9\u5206\u6790" },
  { id: "P03", category: "prohibition", name: "\u8907\u88FD\u7C98\u8CBC", nameEn: "Copy-Paste", description: "\u5927\u91CF\u91CD\u8907\u4EE3\u78BC", severity: "warning", action: "DRY \u91CD\u69CB", autoDetectable: true, detectMethod: "heuristic", implemented: true },
  { id: "P04", category: "prohibition", name: "\u9B54\u6CD5\u6578\u5B57", nameEn: "Magic Numbers", description: "\u786C\u7DE8\u78BC\u6578\u503C\u7121\u8AAA\u660E", severity: "warning", action: "\u63D0\u53D6\u5E38\u91CF", autoDetectable: true, detectMethod: "regex", implemented: true },
  { id: "P05", category: "prohibition", name: "\u8D85\u9577\u51FD\u6578", nameEn: "Long Function", description: "\u51FD\u6578\u884C\u6578\u904E\u591A", severity: "warning", action: "\u62C6\u5206", autoDetectable: true, detectMethod: "ast", implemented: true, threshold: THRESHOLDS.FUNCTION_MAX_LINES },
  { id: "P06", category: "prohibition", name: "\u6DF1\u5C64\u5D4C\u5957", nameEn: "Deep Nesting", description: "\u5D4C\u5957\u5C64\u6578\u904E\u6DF1", severity: "warning", action: "\u63D0\u53D6/\u65E9\u8FD4\u56DE", autoDetectable: true, detectMethod: "ast", implemented: true, threshold: THRESHOLDS.NESTING_MAX_DEPTH },
  { id: "P07", category: "prohibition", name: "\u5168\u5C40\u72C0\u614B", nameEn: "Global State", description: "\u904E\u5EA6\u4F7F\u7528\u5168\u5C40\u8B8A\u91CF", severity: "warning", action: "\u5C01\u88DD", autoDetectable: true, detectMethod: "regex", implemented: true },
  { id: "P08", category: "prohibition", name: "\u7DCA\u8026\u5408", nameEn: "Tight Coupling", description: "\u6A21\u7D44\u9593\u76F4\u63A5\u4F9D\u8CF4", severity: "warning", action: "\u4F9D\u8CF4\u6CE8\u5165", autoDetectable: false, detectMethod: "llm", implemented: false, requiresIntegration: "LLM \u8A9E\u7FA9\u5206\u6790" },
  { id: "P09", category: "prohibition", name: "\u7121\u610F\u7FA9\u547D\u540D", nameEn: "Meaningless Names", description: "temp, data, info \u7B49", severity: "warning", action: "\u91CD\u547D\u540D", autoDetectable: true, detectMethod: "regex", implemented: true },
  { id: "P10", category: "prohibition", name: "\u904E\u9577\u53C3\u6578", nameEn: "Long Parameter List", description: "\u51FD\u6578\u53C3\u6578\u904E\u591A", severity: "warning", action: "\u63D0\u53D6\u7269\u4EF6", autoDetectable: true, detectMethod: "regex", implemented: true, threshold: THRESHOLDS.PARAM_MAX_COUNT },
  { id: "P11", category: "prohibition", name: "\u6DF7\u5408\u62BD\u8C61", nameEn: "Mixed Abstraction", description: "\u9AD8\u4F4E\u5C64\u908F\u8F2F\u6DF7\u96DC", severity: "warning", action: "\u5206\u5C64", autoDetectable: false, detectMethod: "llm", implemented: false, requiresIntegration: "LLM \u8A9E\u7FA9\u5206\u6790" },
  { id: "P12", category: "prohibition", name: "\u8A3B\u91CB\u4EE3\u78BC", nameEn: "Commented Code", description: "\u5927\u91CF\u88AB\u8A3B\u91CB\u7684\u4EE3\u78BC", severity: "info", action: "\u522A\u9664", autoDetectable: true, detectMethod: "regex", implemented: true },
  { id: "P13", category: "prohibition", name: "TODO \u5806\u7A4D", nameEn: "TODO Accumulation", description: "\u672A\u8655\u7406\u7684 TODO", severity: "info", action: "\u8655\u7406\u6216\u79FB\u9664", autoDetectable: true, detectMethod: "regex", implemented: true, threshold: THRESHOLDS.TODO_MAX_COUNT },
  { id: "P14", category: "prohibition", name: "\u4F9D\u8CF4\u81A8\u8139", nameEn: "Dependency Bloat", description: "\u4E0D\u5FC5\u8981\u7684\u4F9D\u8CF4", severity: "info", action: "\u79FB\u9664", autoDetectable: true, detectMethod: "heuristic", implemented: true }
];
function getProhibition(id) {
  return PROHIBITIONS.find((p) => p.id === id);
}
var P03_CHECKER = {
  rule: getProhibition("P03"),
  checkSource(source, file) {
    const violations = [];
    const lines = source.split("\n").map((l) => l.trim()).filter((l) => l.length > 10 && !/^\s*(?:\/\/|#|\*|\/\*)/.test(l));
    const counts = /* @__PURE__ */ new Map();
    lines.forEach((line, i) => {
      const existing = counts.get(line) || [];
      existing.push(i + 1);
      counts.set(line, existing);
    });
    const duplicates = Array.from(counts.entries()).filter(([_, locs]) => locs.length >= THRESHOLDS.DUPLICATE_MIN_COUNT);
    if (duplicates.length > THRESHOLDS.DUPLICATE_GROUPS) {
      violations.push({ ruleId: "P03", ruleName: "\u8907\u88FD\u7C98\u8CBC", severity: "warning", file, line: 1, column: 1, message: `\u6AA2\u6E2C\u5230 ${duplicates.length} \u7D44\u91CD\u8907\u4EE3\u78BC`, suggestion: "\u63D0\u53D6\u70BA\u5171\u7528\u51FD\u6578\u6216\u5E38\u91CF" });
    }
    return violations;
  }
};
var MAGIC_NUMBER_PATTERN = /(?<![\w.])\b(\d{2,})\b(?!\s*[,\]:}])/g;
var ALLOWED_NUMBERS = /* @__PURE__ */ new Set(["0", "1", "2", "10", "100", "1000", "60", "24", "365", "1024", "4096"]);
var P04_CHECKER = {
  rule: getProhibition("P04"),
  checkSource(source, file) {
    if (/(?:test|spec|config)\.[^/]+$/i.test(file)) return [];
    const violations = [];
    const lines = source.split("\n");
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      if (/^\s*(?:\/\/|#|\/\*|\*)/.test(line)) continue;
      if (/^\s*(?:const|let|var|def|final|static)\s+\w+\s*=\s*\d+\s*;?\s*$/.test(line)) continue;
      MAGIC_NUMBER_PATTERN.lastIndex = 0;
      let match;
      while ((match = MAGIC_NUMBER_PATTERN.exec(line)) !== null) {
        if (!ALLOWED_NUMBERS.has(match[1])) {
          violations.push({ ruleId: "P04", ruleName: "\u9B54\u6CD5\u6578\u5B57", severity: "warning", file, line: i + 1, column: match.index + 1, message: `\u9B54\u6CD5\u6578\u5B57: ${match[1]}`, snippet: line.trim(), suggestion: "\u63D0\u53D6\u70BA\u6709\u610F\u7FA9\u7684\u5E38\u91CF" });
        }
      }
    }
    return violations.slice(0, THRESHOLDS.MAGIC_NUMBER_MAX_REPORT);
  }
};
var P05_CHECKER = {
  rule: getProhibition("P05"),
  checkSource(source, file) {
    const violations = [];
    const lines = source.split("\n");
    const threshold = THRESHOLDS.FUNCTION_MAX_LINES;
    const ext = file.split(".").pop()?.toLowerCase();
    let inFunction = false, funcName = "", funcStart = 0, braceCount = 0;
    const funcPattern = ext === "py" ? /^\s*(?:async\s+)?def\s+(\w+)/ : /(?:function|fn|func)\s+(\w+)|(\w+)\s*[=:]\s*(?:async\s+)?(?:function|\([^)]*\)\s*=>)/;
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      if (!inFunction) {
        const match = line.match(funcPattern);
        if (match) {
          inFunction = true;
          funcName = match[1] || match[2] || "anonymous";
          funcStart = i;
          braceCount = (line.match(/\{/g) || []).length - (line.match(/\}/g) || []).length;
        }
      } else if (ext !== "py") {
        braceCount += (line.match(/\{/g) || []).length - (line.match(/\}/g) || []).length;
        if (braceCount <= 0) {
          const len = i - funcStart + 1;
          if (len > threshold) {
            violations.push({ ruleId: "P05", ruleName: "\u8D85\u9577\u51FD\u6578", severity: "warning", file, line: funcStart + 1, column: 1, message: `\u51FD\u6578 "${funcName}" \u9577\u5EA6 ${len} \u884C > ${threshold}`, suggestion: "\u62C6\u5206\u70BA\u591A\u500B\u5C0F\u51FD\u6578" });
          }
          inFunction = false;
        }
      }
    }
    return violations;
  }
};
var P06_CHECKER = {
  rule: getProhibition("P06"),
  checkSource(source, file) {
    const violations = [];
    const lines = source.split("\n");
    const threshold = THRESHOLDS.NESTING_MAX_DEPTH;
    let maxNesting = 0, maxLine = 0, current = 0;
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      current += (line.match(/\{/g) || []).length;
      if (current > maxNesting) {
        maxNesting = current;
        maxLine = i + 1;
      }
      current -= (line.match(/\}/g) || []).length;
      if (current < 0) current = 0;
    }
    if (maxNesting > threshold) {
      violations.push({ ruleId: "P06", ruleName: "\u6DF1\u5C64\u5D4C\u5957", severity: "warning", file, line: maxLine, column: 1, message: `\u6700\u5927\u5D4C\u5957 ${maxNesting} \u5C64 > ${threshold}`, suggestion: "\u4F7F\u7528\u65E9\u8FD4\u56DE\u6216\u63D0\u53D6\u5B50\u51FD\u6578" });
    }
    return violations;
  }
};
var GLOBAL_STATE_PATTERNS = [
  /^(?:var|let)\s+\w+\s*=/gm,
  // 頂層 var/let
  /^window\.\w+\s*=/gm,
  /^global\.\w+\s*=/gm,
  /^globalThis\.\w+\s*=/gm
];
var P07_CHECKER = {
  rule: getProhibition("P07"),
  checkSource(source, file) {
    const violations = [];
    const lines = source.split("\n");
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      for (const pattern of GLOBAL_STATE_PATTERNS) {
        pattern.lastIndex = 0;
        if (pattern.test(line)) {
          violations.push({ ruleId: "P07", ruleName: "\u5168\u5C40\u72C0\u614B", severity: "warning", file, line: i + 1, column: 1, message: "\u6AA2\u6E2C\u5230\u5168\u5C40\u72C0\u614B", snippet: line.trim(), suggestion: "\u5C01\u88DD\u5230\u6A21\u7D44\u6216\u985E\u5225\u4E2D" });
          break;
        }
      }
    }
    return violations.slice(0, 5);
  }
};
var MEANINGLESS = ["temp", "tmp", "data", "info", "val", "value", "result", "res", "obj", "item", "foo", "bar", "baz", "x", "y", "z", "a", "b", "c"];
var P09_CHECKER = {
  rule: getProhibition("P09"),
  checkSource(source, file) {
    if (/(?:test|spec)\.[^/]+$/i.test(file)) return [];
    const violations = [];
    const pattern = new RegExp(`\\b(?:const|let|var|def|fn)\\s+(${MEANINGLESS.join("|")})\\b`, "gi");
    source.split("\n").forEach((line, i) => {
      pattern.lastIndex = 0;
      const match = pattern.exec(line);
      if (match) {
        violations.push({ ruleId: "P09", ruleName: "\u7121\u610F\u7FA9\u547D\u540D", severity: "warning", file, line: i + 1, column: match.index + 1, message: `\u7121\u610F\u7FA9\u547D\u540D: "${match[1]}"`, suggestion: "\u4F7F\u7528\u63CF\u8FF0\u6027\u547D\u540D" });
      }
    });
    return violations.slice(0, THRESHOLDS.MAGIC_NUMBER_MAX_REPORT);
  }
};
var P10_CHECKER = {
  rule: getProhibition("P10"),
  checkSource(source, file) {
    const violations = [];
    const threshold = THRESHOLDS.PARAM_MAX_COUNT;
    const pattern = /(?:function|def|fn|func)\s+(\w+)\s*\(([^)]*)\)/g;
    source.split("\n").forEach((line, i) => {
      pattern.lastIndex = 0;
      let match;
      while ((match = pattern.exec(line)) !== null) {
        const params = match[2]?.trim();
        const count = params ? params.split(",").filter((p) => p.trim()).length : 0;
        if (count > threshold) {
          violations.push({ ruleId: "P10", ruleName: "\u904E\u9577\u53C3\u6578", severity: "warning", file, line: i + 1, column: 1, message: `\u51FD\u6578 "${match[1]}" \u6709 ${count} \u500B\u53C3\u6578 > ${threshold}`, suggestion: "\u4F7F\u7528\u53C3\u6578\u7269\u4EF6" });
        }
      }
    });
    return violations;
  }
};
var COMMENTED_CODE_PATTERNS = [
  /^\s*\/\/\s*(?:const|let|var|function|if|for|while|return|import|export)\s/,
  /^\s*#\s*(?:def|class|if|for|while|return|import|from)\s/,
  /^\s*\/\*[\s\S]*(?:function|if|for|while)[\s\S]*\*\/$/
];
var P12_CHECKER = {
  rule: getProhibition("P12"),
  checkSource(source, file) {
    const violations = [];
    const lines = source.split("\n");
    let commentedCodeCount = 0;
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      if (COMMENTED_CODE_PATTERNS.some((p) => p.test(line))) {
        commentedCodeCount++;
      }
    }
    if (commentedCodeCount > THRESHOLDS.COMMENTED_CODE_THRESHOLD) {
      violations.push({ ruleId: "P12", ruleName: "\u8A3B\u91CB\u4EE3\u78BC", severity: "info", file, line: 1, column: 1, message: `${commentedCodeCount} \u884C\u88AB\u8A3B\u91CB\u7684\u4EE3\u78BC`, suggestion: "\u522A\u9664\u7121\u7528\u8A3B\u91CB\u4EE3\u78BC\uFF0C\u4F7F\u7528 VCS \u4FDD\u5B58\u6B77\u53F2" });
    }
    return violations;
  }
};
var P13_CHECKER = {
  rule: getProhibition("P13"),
  checkSource(source, file) {
    const threshold = THRESHOLDS.TODO_MAX_COUNT;
    const count = (source.match(/\b(TODO|FIXME|HACK|XXX)\b/gi) || []).length;
    if (count > threshold) {
      return [{ ruleId: "P13", ruleName: "TODO \u5806\u7A4D", severity: "info", file, line: 1, column: 1, message: `${count} \u500B TODO/FIXME > ${threshold}`, suggestion: "\u8655\u7406\u6216\u6E05\u7406 TODO" }];
    }
    return [];
  }
};
var P14_CHECKER = {
  rule: getProhibition("P14"),
  checkSource(source, file) {
    if (!file.endsWith("package.json")) return [];
    const violations = [];
    try {
      const pkg = JSON.parse(source);
      const deps = Object.keys(pkg.dependencies || {}).length;
      const devDeps = Object.keys(pkg.devDependencies || {}).length;
      if (deps > THRESHOLDS.DEPS_MAX_COUNT) {
        violations.push({ ruleId: "P14", ruleName: "\u4F9D\u8CF4\u81A8\u8139", severity: "info", file, line: 1, column: 1, message: `${deps} \u500B\u751F\u7522\u4F9D\u8CF4\u53EF\u80FD\u904E\u591A`, suggestion: "\u5BE9\u67E5\u4E26\u79FB\u9664\u4E0D\u5FC5\u8981\u4F9D\u8CF4" });
      }
      if (devDeps > THRESHOLDS.DEV_DEPS_MAX_COUNT) {
        violations.push({ ruleId: "P14", ruleName: "\u4F9D\u8CF4\u81A8\u8139", severity: "info", file, line: 1, column: 1, message: `${devDeps} \u500B\u958B\u767C\u4F9D\u8CF4\u53EF\u80FD\u904E\u591A`, suggestion: "\u6574\u5408\u6216\u79FB\u9664\u91CD\u8907\u5DE5\u5177" });
      }
    } catch {
    }
    return violations;
  }
};
var PROHIBITION_CHECKERS = [
  P03_CHECKER,
  P04_CHECKER,
  P05_CHECKER,
  P06_CHECKER,
  P07_CHECKER,
  P09_CHECKER,
  P10_CHECKER,
  P12_CHECKER,
  P13_CHECKER,
  P14_CHECKER
];
function checkProhibitions(source, file) {
  return PROHIBITION_CHECKERS.flatMap((c) => c.checkSource?.(source, file) || []);
}

// src/rules/c-gates.ts
var GATES = [
  {
    id: "Gate-In",
    name: "\u5165\u53E3\u95DC\u5361",
    nameEn: "Gate-In",
    description: "\u9700\u6C42\u78BA\u8A8D\u968E\u6BB5",
    items: [
      { name: "\u9700\u6C42\u660E\u78BA", required: true },
      { name: "\u9A57\u6536\u6A19\u6E96\u5B9A\u7FA9", required: true },
      { name: "\u6280\u8853\u9078\u578B\u78BA\u5B9A", required: true },
      { name: "\u4F9D\u8CF4\u78BA\u8A8D\u53EF\u7528", required: true },
      { name: "\u8CC7\u6E90\u5DF2\u8A55\u4F30", required: true }
    ]
  },
  {
    id: "Gate-Mid",
    name: "\u4E2D\u671F\u95DC\u5361",
    nameEn: "Gate-Mid",
    description: "50% \u9032\u5EA6\u6AA2\u67E5",
    items: [
      { name: "\u9032\u5EA6\u5728 \xB120% \u5167", required: true },
      { name: "\u6838\u5FC3\u67B6\u69CB\u7A69\u5B9A", required: true },
      { name: "\u6838\u5FC3\u908F\u8F2F\u6709\u6E2C\u8A66", required: true },
      { name: "\u963B\u585E\u9805\u6709\u65B9\u6848", required: true }
    ]
  },
  {
    id: "Gate-Out",
    name: "\u51FA\u53E3\u95DC\u5361",
    nameEn: "Gate-Out",
    description: "\u5B8C\u6210\u6AA2\u67E5",
    items: [
      { name: "\u6240\u6709\u529F\u80FD\u5B8C\u6210", required: true },
      { name: "\u6240\u6709\u6E2C\u8A66\u901A\u904E", required: true },
      { name: "\u8986\u84CB\u7387 \u2265 80%", required: true },
      { name: "\u7121\u7D05\u7DDA\u9055\u898F", required: true },
      { name: "\u6587\u6A94\u5B8C\u6210", required: true },
      { name: "\u4EE3\u78BC\u5BE9\u67E5\u901A\u904E", required: true },
      { name: "\u5B89\u5168\u6383\u63CF\u901A\u904E", required: true }
    ]
  },
  {
    id: "Gate-Accept",
    name: "\u9A57\u6536\u95DC\u5361",
    nameEn: "Gate-Accept",
    description: "\u9A57\u6536\u78BA\u8A8D",
    items: [
      { name: "\u7528\u6236\u9A57\u6536\u6E2C\u8A66\u901A\u904E", required: true },
      { name: "\u6027\u80FD\u6307\u6A19\u9054\u6A19", required: true },
      { name: "\u53EF\u90E8\u7F72\u5230\u76EE\u6A19\u74B0\u5883", required: true },
      { name: "\u56DE\u6EFE\u65B9\u6848\u5C31\u7DD2", required: true },
      { name: "\u76E3\u63A7\u544A\u8B66\u914D\u7F6E", required: true }
    ]
  }
];
function evaluateGate(gateId, results) {
  const definition = GATES.find((g) => g.id === gateId);
  if (!definition) {
    throw new Error(`Unknown gate: ${gateId}`);
  }
  const items = definition.items.map((item) => ({
    ...item,
    passed: results[item.name] ?? false
  }));
  const passed = items.filter((item) => item.required).every((item) => item.passed);
  return {
    id: gateId,
    name: definition.name,
    passed,
    items
  };
}
function createGateStatus(gateIn, gateMid, gateOut, gateAccept) {
  return {
    gateIn: evaluateGate("Gate-In", gateIn),
    gateMid: evaluateGate("Gate-Mid", gateMid),
    gateOut: evaluateGate("Gate-Out", gateOut),
    gateAccept: evaluateGate("Gate-Accept", gateAccept)
  };
}
var X_WEIGHTS = {
  codeStandard: 0.15,
  architecture: 0.2,
  security: 0.25,
  testing: 0.2,
  documentation: 0.1,
  process: 0.1
};
var Y_WEIGHTS = {
  functionality: 0.3,
  quality: 0.2,
  performance: 0.2,
  usability: 0.15,
  satisfaction: 0.15
};
function calculateComplianceScore(scores) {
  const total = scores.codeStandard * X_WEIGHTS.codeStandard + scores.architecture * X_WEIGHTS.architecture + scores.security * X_WEIGHTS.security + scores.testing * X_WEIGHTS.testing + scores.documentation * X_WEIGHTS.documentation + scores.process * X_WEIGHTS.process;
  return { ...scores, total: Math.round(total * 100) / 100 };
}
function calculateOutcomeScore(scores) {
  const total = scores.functionality * Y_WEIGHTS.functionality + scores.quality * Y_WEIGHTS.quality + scores.performance * Y_WEIGHTS.performance + scores.usability * Y_WEIGHTS.usability + scores.satisfaction * Y_WEIGHTS.satisfaction;
  return { ...scores, total: Math.round(total * 100) / 100 };
}
function calculateDualAxisScore(xScores, yScores) {
  const x = calculateComplianceScore(xScores);
  const y = calculateOutcomeScore(yScores);
  let grade;
  if (x.total >= 80 && y.total >= 80) {
    grade = "A";
  } else if (x.total < 80 && y.total >= 80) {
    grade = "B";
  } else if (x.total >= 80 && y.total < 80) {
    grade = "C";
  } else {
    grade = "D";
  }
  return { x, y, grade };
}
function generateGateChecklist(gateId) {
  const definition = GATES.find((g) => g.id === gateId);
  if (!definition) {
    throw new Error(`Unknown gate: ${gateId}`);
  }
  const lines = [
    `# ${definition.name} (${definition.nameEn})`,
    "",
    definition.description,
    "",
    "## \u6AA2\u67E5\u9805",
    ""
  ];
  for (const item of definition.items) {
    const marker = item.required ? "\u2705" : "\u26AA";
    lines.push(`- [ ] ${marker} ${item.name}`);
  }
  return lines.join("\n");
}
function generateAllGatesChecklist() {
  return GATES.map((g) => generateGateChecklist(g.id)).join("\n\n---\n\n");
}
var HANDOVER_TEMPLATE = `# \u4EA4\u63A5\u6587\u6A94

## \u6458\u8981
- **\u9805\u76EE**: <\u9805\u76EE\u540D>
- **\u65E5\u671F**: <\u65E5\u671F>
- **\u72C0\u614B**: \u{1F7E2}/\u{1F7E1}/\u{1F534}

## \u9032\u5EA6
- [x] \u5DF2\u5B8C\u6210\u9805\u76EE
- [ ] \u9032\u884C\u4E2D - XX%

## \u963B\u585E\u9805
| \u963B\u585E | \u9700\u8981 |
|:-----|:-----|
| <\u554F\u984C> | <\u8CC7\u6E90> |

## \u4E0B\u4E00\u6B65
1. **P0**: <\u6700\u9AD8\u512A\u5148>
2. **P1**: <\u6B21\u512A\u5148>

## \u6CE8\u610F\u4E8B\u9805
\u26A0\uFE0F <\u91CD\u8981\u63D0\u9192>
`;
var HANDOVER_TAGS = [
  { tag: "@HANDOVER", description: "\u4EA4\u63A5\u9EDE", format: "// @HANDOVER: <\u8AAA\u660E>" },
  { tag: "@WIP", description: "\u9032\u884C\u4E2D", format: "// @WIP: <\u5269\u9918\u5DE5\u4F5C>" },
  { tag: "@BLOCKED", description: "\u963B\u585E\u9EDE", format: "// @BLOCKED: <\u539F\u56E0>" },
  { tag: "@DECISION", description: "\u9700\u6C7A\u7B56", format: "// @DECISION: <\u9078\u9805>" },
  { tag: "@REVIEW", description: "\u9700\u5BE9\u67E5", format: "// @REVIEW: <\u95DC\u6CE8\u9EDE>" },
  { tag: "@FIXME", description: "\u9700\u4FEE\u5FA9", format: "// @FIXME: <\u554F\u984C>" },
  { tag: "@HACK", description: "\u81E8\u6642\u65B9\u6848", format: "// @HACK: <\u539F\u56E0>" }
];
function extractHandoverTags(source) {
  const results = [];
  const lines = source.split("\n");
  const tagPattern = /@(HANDOVER|WIP|BLOCKED|DECISION|REVIEW|FIXME|HACK)\s*:\s*(.+)/gi;
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    tagPattern.lastIndex = 0;
    const match = tagPattern.exec(line);
    if (match) {
      results.push({
        tag: match[1].toUpperCase(),
        line: i + 1,
        content: match[2].trim()
      });
    }
  }
  return results;
}

// src/rules/index.ts
function checkRules(source, file, level = "D") {
  const violations = [];
  if (level === "B" || level === "D") {
    violations.push(...checkRedlines(source, file));
    violations.push(...checkProhibitions(source, file));
  }
  return violations;
}
function checkFraud(source, file) {
  return checkAntifraud(source, file);
}

// src/analyzer.ts
function detectLanguage(file) {
  return detectLanguageFromPath(file);
}
function isSupported(file) {
  return isSupportedPath(file);
}
var C_LIKE_LANGUAGES = /* @__PURE__ */ new Set([
  // Core
  "typescript",
  "javascript",
  "rust",
  "go",
  // JVM
  "java",
  "kotlin",
  "scala",
  "groovy",
  // .NET
  "csharp",
  // Mobile
  "swift",
  "objc",
  "dart",
  // Systems
  "c",
  "cpp",
  "zig"
]);
function getCommentSyntax(language) {
  if (C_LIKE_LANGUAGES.has(language)) {
    return {
      line: [/^\s*\/\//],
      blocks: [{ start: /^\s*\/\*/, end: /\*\/\s*$/ }]
    };
  }
  switch (language) {
    case "python":
      return {
        line: [/^\s*#/],
        blocks: [{ start: /^\s*['"]{3}/, end: /['"]{3}\s*$/ }]
      };
    case "clojure":
      return { line: [/^\s*;/, /^\s*#_/], blocks: [] };
    case "fsharp":
      return {
        line: [/^\s*\/\//],
        blocks: [{ start: /^\s*\(\*/, end: /\*\)\s*$/ }]
      };
    case "vbnet":
      return { line: [/^\s*'/, /^\s*rem\b/i], blocks: [] };
    case "nim":
      return {
        line: [/^\s*#/],
        blocks: [{ start: /^\s*#\[/, end: /\]#\s*$/ }]
      };
    // Web
    case "php":
      return {
        line: [/^\s*\/\//, /^\s*#/],
        blocks: [{ start: /^\s*\/\*/, end: /\*\/\s*$/ }]
      };
    case "ruby":
      return {
        line: [/^\s*#/],
        blocks: [{ start: /^\s*=begin\b/, end: /^\s*=end\b/ }]
      };
    // Scripting
    case "shell":
      return { line: [/^\s*#/], blocks: [] };
    case "powershell":
      return {
        line: [/^\s*#/],
        blocks: [{ start: /^\s*<#/, end: /#>\s*$/ }]
      };
    case "perl":
      return { line: [/^\s*#/], blocks: [] };
    case "lua":
      return {
        line: [/^\s*--/],
        blocks: [{ start: /^\s*--\[\[/, end: /\]\]\s*$/ }]
      };
    // Data
    case "sql":
    case "plsql":
      return {
        line: [/^\s*--/],
        blocks: [{ start: /^\s*\/\*/, end: /\*\/\s*$/ }]
      };
    case "r":
      return { line: [/^\s*#/], blocks: [] };
    case "julia":
      return {
        line: [/^\s*#/],
        blocks: [{ start: /^\s*#=/, end: /=#\s*$/ }]
      };
    // Config
    case "yaml":
    case "toml":
      return { line: [/^\s*#/], blocks: [] };
    case "json":
      return { line: [], blocks: [] };
    case "xml":
      return { line: [], blocks: [{ start: /^\s*<!--/, end: /-->\s*$/ }] };
    // Functional
    case "elixir":
      return { line: [/^\s*#/], blocks: [] };
    case "haskell":
      return {
        line: [/^\s*--/],
        blocks: [{ start: /^\s*\{-/, end: /-\}\s*$/ }]
      };
    case "ocaml":
      return { line: [], blocks: [{ start: /^\s*\(\*/, end: /\*\)\s*$/ }] };
    case "erlang":
      return { line: [/^\s*%/], blocks: [] };
    // Enterprise
    case "cobol":
      return { line: [/^\s*\*>/, /^\s*\*/], blocks: [] };
    case "abap":
      return { line: [/^\s*\*/, /^\s*"/], blocks: [] };
    case "fortran":
      return { line: [/^\s*!/, /^[cC*]/], blocks: [] };
    case "vba":
      return { line: [/^\s*'/, /^\s*rem\b/i], blocks: [] };
    case "rpg":
      return { line: [/^\s*\/\//, /^\s*\*/], blocks: [] };
    default:
      return { line: [], blocks: [] };
  }
}
function countLines(source, language) {
  const lines = source.split("\n");
  let codeLines = 0;
  let commentLines = 0;
  let blankLines = 0;
  let inBlockComment = null;
  const syntax = getCommentSyntax(language);
  for (const line of lines) {
    const trimmed = line.trim();
    if (trimmed === "") {
      blankLines++;
      continue;
    }
    if (inBlockComment) {
      commentLines++;
      if (inBlockComment.end.test(trimmed)) {
        inBlockComment = null;
      }
      continue;
    }
    let matched = false;
    for (const block of syntax.blocks) {
      if (block.start.test(trimmed)) {
        commentLines++;
        matched = true;
        if (!block.end.test(trimmed)) {
          inBlockComment = block;
        }
        break;
      }
    }
    if (matched) continue;
    for (const lc of syntax.line) {
      if (lc.test(trimmed)) {
        commentLines++;
        matched = true;
        break;
      }
    }
    if (matched) continue;
    codeLines++;
  }
  return {
    totalLines: lines.length,
    codeLines,
    commentLines,
    blankLines
  };
}
function analyzeFile(source, file, level) {
  const startTime = performance.now();
  const language = detectLanguage(file);
  if (!language) {
    return {
      file,
      language: "typescript",
      // fallback
      violations: [],
      duration: 0,
      stats: { totalLines: 0, codeLines: 0, commentLines: 0, blankLines: 0 }
    };
  }
  const violations = checkRules(source, file, level);
  const stats = countLines(source, language);
  const endTime = performance.now();
  return {
    file,
    language,
    violations,
    duration: Math.round(endTime - startTime),
    stats
  };
}
function analyze(options) {
  const startTime = performance.now();
  const { files, level, targetPath } = options;
  const fileResults = [];
  const byRule = {};
  let errorCount = 0;
  let warningCount = 0;
  let infoCount = 0;
  for (const { path, content } of files) {
    const result = analyzeFile(content, path, level);
    fileResults.push(result);
    for (const v of result.violations) {
      byRule[v.ruleId] = (byRule[v.ruleId] || 0) + 1;
      switch (v.severity) {
        case "error":
          errorCount++;
          break;
        case "warning":
          warningCount++;
          break;
        case "info":
          infoCount++;
          break;
      }
    }
  }
  const endTime = performance.now();
  return {
    timestamp: (/* @__PURE__ */ new Date()).toISOString(),
    targetPath,
    level,
    files: fileResults,
    summary: {
      totalFiles: files.length,
      totalViolations: errorCount + warningCount + infoCount,
      errorCount,
      warningCount,
      infoCount,
      byRule
    },
    duration: Math.round(endTime - startTime)
  };
}
function quickCheck(source, file) {
  return checkRules(source, file, "D");
}

// src/reporter/console.ts
var colors = {
  reset: "\x1B[0m",
  bold: "\x1B[1m",
  dim: "\x1B[2m",
  red: "\x1B[31m",
  green: "\x1B[32m",
  yellow: "\x1B[33m",
  blue: "\x1B[34m",
  cyan: "\x1B[36m",
  white: "\x1B[37m"};
function colorize(text, ...codes) {
  return `${codes.join("")}${text}${colors.reset}`;
}
function severityIcon(severity) {
  switch (severity) {
    case "error":
      return colorize("\u{1F534}", colors.red);
    case "warning":
      return colorize("\u{1F7E1}", colors.yellow);
    case "info":
      return colorize("\u{1F535}", colors.blue);
  }
}
function severityColor(severity) {
  switch (severity) {
    case "error":
      return colors.red;
    case "warning":
      return colors.yellow;
    case "info":
      return colors.blue;
  }
}
function formatViolation(v) {
  const icon = severityIcon(v.severity);
  const ruleId = colorize(v.ruleId, colors.bold, severityColor(v.severity));
  const location = colorize(`${v.file}:${v.line}:${v.column}`, colors.dim);
  let output = `${icon} ${ruleId} ${location}
`;
  output += `   ${v.message}
`;
  if (v.snippet) {
    output += colorize(`   > ${v.snippet}
`, colors.dim);
  }
  if (v.suggestion) {
    output += colorize(`   \u{1F4A1} ${v.suggestion}
`, colors.cyan);
  }
  return output;
}
var consoleReporter = {
  name: "console",
  report(result) {
    const lines = [];
    lines.push("");
    lines.push(colorize("\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550", colors.dim));
    lines.push(colorize("  MAIDOS CodeQC v2.4 Analysis Report", colors.bold, colors.cyan));
    lines.push(colorize("\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550", colors.dim));
    lines.push("");
    lines.push(`\u{1F4C2} Target: ${colorize(result.targetPath, colors.bold)}`);
    lines.push(`\u{1F4CA} Level: ${colorize(result.level, colors.bold)}`);
    lines.push(`\u23F1\uFE0F  Duration: ${colorize(`${result.duration}ms`, colors.dim)}`);
    lines.push(`\u{1F4C1} Files: ${colorize(String(result.summary.totalFiles), colors.bold)}`);
    lines.push("");
    if (result.summary.totalViolations > 0) {
      lines.push(colorize("\u2500\u2500\u2500 Violations \u2500\u2500\u2500", colors.dim));
      lines.push("");
      for (const fileResult of result.files) {
        if (fileResult.violations.length === 0) continue;
        lines.push(colorize(`\u{1F4C4} ${fileResult.file}`, colors.bold));
        lines.push("");
        for (const violation of fileResult.violations) {
          lines.push(formatViolation(violation));
        }
      }
    } else {
      lines.push(colorize("\u2705 No violations found!", colors.green, colors.bold));
      lines.push("");
    }
    lines.push(colorize("\u2500\u2500\u2500 Summary \u2500\u2500\u2500", colors.dim));
    lines.push("");
    const { errorCount, warningCount, infoCount } = result.summary;
    lines.push(`${severityIcon("error")} Errors:   ${colorize(String(errorCount), errorCount > 0 ? colors.red : colors.green)}`);
    lines.push(`${severityIcon("warning")} Warnings: ${colorize(String(warningCount), warningCount > 0 ? colors.yellow : colors.green)}`);
    lines.push(`${severityIcon("info")} Info:     ${colorize(String(infoCount), colors.blue)}`);
    lines.push("");
    if (result.gates) {
      lines.push(colorize("\u2500\u2500\u2500 Gate Status \u2500\u2500\u2500", colors.dim));
      lines.push("");
      const gateStatus = (passed2) => passed2 ? colorize("\u2705 PASS", colors.green, colors.bold) : colorize("\u274C FAIL", colors.red, colors.bold);
      lines.push(`Gate-In:     ${gateStatus(result.gates.gateIn.passed)}`);
      lines.push(`Gate-Mid:    ${gateStatus(result.gates.gateMid.passed)}`);
      lines.push(`Gate-Out:    ${gateStatus(result.gates.gateOut.passed)}`);
      lines.push(`Gate-Accept: ${gateStatus(result.gates.gateAccept.passed)}`);
      lines.push("");
    }
    if (result.score) {
      lines.push(colorize("\u2500\u2500\u2500 Dual-Axis Score \u2500\u2500\u2500", colors.dim));
      lines.push("");
      const gradeColor = (grade) => {
        switch (grade) {
          case "A":
            return colors.green;
          case "B":
            return colors.blue;
          case "C":
            return colors.yellow;
          case "D":
            return colors.red;
          default:
            return colors.white;
        }
      };
      lines.push(`X-Axis (Compliance): ${colorize(`${result.score.x.total}%`, colors.bold)}`);
      lines.push(`Y-Axis (Outcome):    ${colorize(`${result.score.y.total}%`, colors.bold)}`);
      lines.push(`Grade: ${colorize(result.score.grade, colors.bold, gradeColor(result.score.grade))}`);
      lines.push("");
    }
    const passed = result.summary.errorCount === 0;
    lines.push(colorize("\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550", colors.dim));
    if (passed) {
      lines.push(colorize("  \u2705 Gate-Out: PASS", colors.green, colors.bold));
    } else {
      lines.push(colorize("  \u274C Gate-Out: FAIL", colors.red, colors.bold));
    }
    lines.push(colorize("\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550", colors.dim));
    lines.push("");
    return lines.join("\n");
  }
};

// src/reporter/json.ts
var jsonReporter = {
  name: "json",
  report(result) {
    return JSON.stringify(result, null, 2);
  }
};

// src/reporter/html.ts
function escapeHtml(text) {
  return text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;").replace(/'/g, "&#039;");
}
function renderCategories(categories) {
  const names = {
    security: "\u{1F512} \u5B89\u5168\u6027",
    structure: "\u{1F3D7}\uFE0F \u7D50\u69CB\u6027",
    quality: "\u2728 \u4EE3\u78BC\u8CEA\u91CF"
  };
  return categories.map((c) => names[c]).join(" + ");
}
function severityClass(severity) {
  switch (severity) {
    case "error":
      return "severity-error";
    case "warning":
      return "severity-warning";
    case "info":
      return "severity-info";
  }
}
function severityIcon2(severity) {
  switch (severity) {
    case "error":
      return "\u{1F534}";
    case "warning":
      return "\u{1F7E1}";
    case "info":
      return "\u{1F535}";
  }
}
function renderViolation(v) {
  return `
    <div class="violation ${severityClass(v.severity)}">
      <div class="violation-header">
        <span class="severity-icon">${severityIcon2(v.severity)}</span>
        <span class="rule-id">${escapeHtml(v.ruleId)}</span>
        <span class="rule-name">${escapeHtml(v.ruleName)}</span>
        <span class="location">${escapeHtml(v.file)}:${v.line}:${v.column}</span>
      </div>
      <div class="violation-message">${escapeHtml(v.message)}</div>
      ${v.snippet ? `<pre class="snippet">${escapeHtml(v.snippet)}</pre>` : ""}
      ${v.suggestion ? `<div class="suggestion">\u{1F4A1} ${escapeHtml(v.suggestion)}</div>` : ""}
    </div>
  `;
}
var htmlReporter = {
  name: "html",
  report(result) {
    const { summary, score, gates } = result;
    const passed = summary.errorCount === 0;
    return `<!DOCTYPE html>
<html lang="zh-TW">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>MAIDOS CodeQC Report</title>
  <style>
    :root {
      --color-error: #ef4444;
      --color-warning: #f59e0b;
      --color-info: #3b82f6;
      --color-success: #22c55e;
      --color-bg: #0f172a;
      --color-card: #1e293b;
      --color-text: #f8fafc;
      --color-dim: #94a3b8;
    }
    
    * { box-sizing: border-box; margin: 0; padding: 0; }
    
    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      background: var(--color-bg);
      color: var(--color-text);
      line-height: 1.6;
      padding: 2rem;
    }
    
    .container { max-width: 1200px; margin: 0 auto; }
    
    h1 {
      font-size: 2rem;
      margin-bottom: 0.5rem;
      background: linear-gradient(135deg, #06b6d4, #3b82f6);
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
    }
    
    .meta {
      color: var(--color-dim);
      margin-bottom: 2rem;
    }
    
    .summary-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
      gap: 1rem;
      margin-bottom: 2rem;
    }
    
    .card {
      background: var(--color-card);
      border-radius: 12px;
      padding: 1.5rem;
    }
    
    .card-title {
      color: var(--color-dim);
      font-size: 0.875rem;
      text-transform: uppercase;
      letter-spacing: 0.05em;
      margin-bottom: 0.5rem;
    }
    
    .card-value {
      font-size: 2rem;
      font-weight: bold;
    }
    
    .card-value.error { color: var(--color-error); }
    .card-value.warning { color: var(--color-warning); }
    .card-value.info { color: var(--color-info); }
    .card-value.success { color: var(--color-success); }
    
    .analysis-categories {
      font-size: 1.1rem;
      color: var(--color-text);
      margin-top: 0.5rem;
    }
    
    .category-badges {
      display: flex;
      gap: 0.75rem;
      margin: 1.5rem 0;
      flex-wrap: wrap;
    }
    
    .badge {
      padding: 0.5rem 1rem;
      border-radius: 20px;
      font-size: 0.9rem;
      font-weight: 500;
      transition: all 0.2s;
    }
    
    .badge-security {
      background: linear-gradient(135deg, #ef4444, #dc2626);
      color: white;
      box-shadow: 0 2px 8px rgba(239, 68, 68, 0.3);
    }
    
    .badge-structure {
      background: linear-gradient(135deg, #3b82f6, #2563eb);
      color: white;
      box-shadow: 0 2px 8px rgba(59, 130, 246, 0.3);
    }
    
    .badge-quality {
      background: linear-gradient(135deg, #22c55e, #16a34a);
      color: white;
      box-shadow: 0 2px 8px rgba(34, 197, 94, 0.3);
    }
    
    .badge-disabled {
      background: var(--color-card);
      color: var(--color-dim);
      opacity: 0.5;
    }
    
    .verdict {
      text-align: center;
      padding: 2rem;
      border-radius: 12px;
      margin-bottom: 2rem;
    }
    
    .verdict.pass { background: rgba(34, 197, 94, 0.2); border: 2px solid var(--color-success); }
    .verdict.fail { background: rgba(239, 68, 68, 0.2); border: 2px solid var(--color-error); }
    
    .verdict-text {
      font-size: 1.5rem;
      font-weight: bold;
    }
    
    .violations-section {
      margin-bottom: 2rem;
    }
    
    .section-title {
      font-size: 1.25rem;
      margin-bottom: 1rem;
      padding-bottom: 0.5rem;
      border-bottom: 1px solid var(--color-card);
    }
    
    .file-group {
      margin-bottom: 1.5rem;
    }
    
    .file-name {
      font-weight: bold;
      color: var(--color-dim);
      margin-bottom: 0.5rem;
    }
    
    .violation {
      background: var(--color-card);
      border-radius: 8px;
      padding: 1rem;
      margin-bottom: 0.75rem;
      border-left: 4px solid;
    }
    
    .violation.severity-error { border-color: var(--color-error); }
    .violation.severity-warning { border-color: var(--color-warning); }
    .violation.severity-info { border-color: var(--color-info); }
    
    .violation-header {
      display: flex;
      flex-wrap: wrap;
      gap: 0.5rem;
      align-items: center;
      margin-bottom: 0.5rem;
    }
    
    .rule-id {
      font-weight: bold;
      font-family: monospace;
    }
    
    .rule-name { color: var(--color-dim); }
    
    .location {
      font-family: monospace;
      font-size: 0.875rem;
      color: var(--color-dim);
      margin-left: auto;
    }
    
    .violation-message { margin-bottom: 0.5rem; }
    
    .snippet {
      background: rgba(0, 0, 0, 0.3);
      padding: 0.5rem;
      border-radius: 4px;
      font-family: monospace;
      font-size: 0.875rem;
      overflow-x: auto;
      margin-bottom: 0.5rem;
    }
    
    .suggestion {
      color: #06b6d4;
      font-size: 0.875rem;
    }
    
    .score-section {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
      gap: 1rem;
      margin-bottom: 2rem;
    }
    
    .score-bar {
      height: 8px;
      background: rgba(255, 255, 255, 0.1);
      border-radius: 4px;
      overflow: hidden;
      margin-top: 0.5rem;
    }
    
    .score-fill {
      height: 100%;
      background: linear-gradient(90deg, #06b6d4, #3b82f6);
      transition: width 0.3s;
    }
    
    .grade {
      font-size: 4rem;
      font-weight: bold;
      text-align: center;
    }
    
    .grade-A { color: var(--color-success); }
    .grade-B { color: var(--color-info); }
    .grade-C { color: var(--color-warning); }
    .grade-D { color: var(--color-error); }
    
    .gate-status {
      display: grid;
      grid-template-columns: repeat(4, 1fr);
      gap: 0.5rem;
    }
    
    .gate {
      padding: 1rem;
      text-align: center;
      border-radius: 8px;
    }
    
    .gate.pass { background: rgba(34, 197, 94, 0.2); }
    .gate.fail { background: rgba(239, 68, 68, 0.2); }
    
    @media (max-width: 768px) {
      body { padding: 1rem; }
      .gate-status { grid-template-columns: repeat(2, 1fr); }
    }
  </style>
</head>
<body>
  <div class="container">
    <h1>\u{1F4CA} MAIDOS CodeQC Report</h1>
    <div class="meta">
      <div>\u{1F4C2} ${escapeHtml(result.targetPath)}</div>
      <div>\u{1F4CA} Level: ${result.level} | \u23F1\uFE0F ${result.duration}ms | \u{1F4C1} ${summary.totalFiles} files</div>
      <div>\u{1F550} ${result.timestamp}</div>
      ${result.categories ? `<div class="analysis-categories">\u{1F50D} \u5206\u6790\u985E\u578B: ${renderCategories(result.categories)}</div>` : ""}
    </div>
    
    ${result.categories ? `
    <div class="category-badges">
      ${result.categories.includes("security") ? '<span class="badge badge-security">\u{1F512} \u5B89\u5168\u6027 Security</span>' : '<span class="badge badge-disabled">\u{1F512} \u5B89\u5168\u6027</span>'}
      ${result.categories.includes("structure") ? '<span class="badge badge-structure">\u{1F3D7}\uFE0F \u7D50\u69CB\u6027 Structure</span>' : '<span class="badge badge-disabled">\u{1F3D7}\uFE0F \u7D50\u69CB\u6027</span>'}
      ${result.categories.includes("quality") ? '<span class="badge badge-quality">\u2728 \u4EE3\u78BC\u8CEA\u91CF Quality</span>' : '<span class="badge badge-disabled">\u2728 \u4EE3\u78BC\u8CEA\u91CF</span>'}
    </div>
    ` : ""}
    
    <div class="verdict ${passed ? "pass" : "fail"}">
      <div class="verdict-text">${passed ? "\u2705 Gate-Out: PASS" : "\u274C Gate-Out: FAIL"}</div>
    </div>
    
    <div class="summary-grid">
      <div class="card">
        <div class="card-title">Total Violations</div>
        <div class="card-value ${summary.totalViolations > 0 ? "error" : "success"}">${summary.totalViolations}</div>
      </div>
      <div class="card">
        <div class="card-title">Errors</div>
        <div class="card-value ${summary.errorCount > 0 ? "error" : "success"}">${summary.errorCount}</div>
      </div>
      <div class="card">
        <div class="card-title">Warnings</div>
        <div class="card-value ${summary.warningCount > 0 ? "warning" : "success"}">${summary.warningCount}</div>
      </div>
      <div class="card">
        <div class="card-title">Info</div>
        <div class="card-value info">${summary.infoCount}</div>
      </div>
    </div>
    
    ${score ? `
    <div class="section-title">\u{1F4C8} Dual-Axis Score</div>
    <div class="score-section">
      <div class="card">
        <div class="card-title">X-Axis: Compliance</div>
        <div class="card-value">${score.x.total}%</div>
        <div class="score-bar"><div class="score-fill" style="width: ${score.x.total}%"></div></div>
      </div>
      <div class="card">
        <div class="card-title">Y-Axis: Outcome</div>
        <div class="card-value">${score.y.total}%</div>
        <div class="score-bar"><div class="score-fill" style="width: ${score.y.total}%"></div></div>
      </div>
      <div class="card">
        <div class="card-title">Grade</div>
        <div class="grade grade-${score.grade}">${score.grade}</div>
      </div>
    </div>
    ` : ""}
    
    ${gates ? `
    <div class="section-title">\u{1F6AA} Gate Status</div>
    <div class="card">
      <div class="gate-status">
        <div class="gate ${gates.gateIn.passed ? "pass" : "fail"}">
          <div>${gates.gateIn.passed ? "\u2705" : "\u274C"}</div>
          <div>Gate-In</div>
        </div>
        <div class="gate ${gates.gateMid.passed ? "pass" : "fail"}">
          <div>${gates.gateMid.passed ? "\u2705" : "\u274C"}</div>
          <div>Gate-Mid</div>
        </div>
        <div class="gate ${gates.gateOut.passed ? "pass" : "fail"}">
          <div>${gates.gateOut.passed ? "\u2705" : "\u274C"}</div>
          <div>Gate-Out</div>
        </div>
        <div class="gate ${gates.gateAccept.passed ? "pass" : "fail"}">
          <div>${gates.gateAccept.passed ? "\u2705" : "\u274C"}</div>
          <div>Gate-Accept</div>
        </div>
      </div>
    </div>
    ` : ""}
    
    ${summary.totalViolations > 0 ? `
    <div class="violations-section">
      <div class="section-title">\u26A0\uFE0F Violations</div>
      ${result.files.filter((f) => f.violations.length > 0).map((f) => `
          <div class="file-group">
            <div class="file-name">\u{1F4C4} ${escapeHtml(f.file)}</div>
            ${f.violations.map((v) => renderViolation(v)).join("")}
          </div>
        `).join("")}
    </div>
    ` : ""}
    
    <div class="meta" style="text-align: center; margin-top: 3rem;">
      Generated by MAIDOS CodeQC v2.4
    </div>
  </div>
</body>
</html>`;
  }
};

// src/reporter/index.ts
var reporters = {
  console: consoleReporter,
  json: jsonReporter,
  html: htmlReporter
};
function getReporter(name) {
  const reporter = reporters[name];
  if (!reporter) {
    throw new Error(`Unknown reporter: ${name}. Available: ${Object.keys(reporters).join(", ")}`);
  }
  return reporter;
}

// src/engine/gates-v33.ts
function runG1(input) {
  const { step8Result } = input;
  return {
    passed: step8Result.passed,
    tool: GATE_CIRCUIT_LABELS.G1.tool,
    details: step8Result.passed ? "G1 PASS: \u63A5\u53E3\u6B63\u5E38\uFF0C\u6C92\u6709\u65B7\u958B\u7684\u529F\u80FD" : `G1 FAIL: ${step8Result.details}`
  };
}
function runG2(input) {
  const { step9Result } = input;
  return {
    passed: step9Result.passed,
    tool: GATE_CIRCUIT_LABELS.G2.tool,
    details: step9Result.passed ? "G2 PASS: \u898F\u683C\u5168\u8986\u84CB\uFF0CSPEC 100% \u5DF2\u5BE6\u4F5C" : `G2 FAIL: ${step9Result.details}`
  };
}
function runG3(input) {
  const { allSteps, files } = input;
  const step1 = allSteps[0];
  const step2 = allSteps[1];
  const step7 = allSteps[6];
  const protectionPassed = (step1?.passed ?? false) && (step2?.passed ?? false) && (step7?.passed ?? false);
  let prohibitionErrors = 0;
  for (const f of files) {
    const pViolations = checkProhibitions(f.content, f.path);
    prohibitionErrors += pViolations.filter((v) => v.severity === "error").length;
  }
  const passed = protectionPassed && prohibitionErrors === 0;
  const failReasons = [];
  if (!step1?.passed) failReasons.push("\u9632\u5047(R13-18)");
  if (!step2?.passed) failReasons.push("\u9632\u8A50");
  if (!step7?.passed) failReasons.push("\u7D05\u7DDA");
  if (prohibitionErrors > 0) failReasons.push(`\u7981\u6B62\u898F\u5247(${prohibitionErrors}e)`);
  return {
    passed,
    tool: GATE_CIRCUIT_LABELS.G3.tool,
    details: passed ? "G3 PASS: \u9632\u8B77\u5B8C\u6574\uFF0C\u6240\u6709\u9632\u7DDA\u90FD\u5B88\u4F4F" : `G3 FAIL: ${failReasons.join(" + ")} \u6C92\u904E`
  };
}
function runG4(input, g1, g2, g3) {
  const { allSteps, step10Result } = input;
  const buildPass = allSteps[2]?.passed ?? false;
  const testPass = allSteps[4]?.passed ?? false;
  const passed = g1.passed && g2.passed && g3.passed && buildPass && testPass && step10Result.passed;
  const failReasons = [];
  if (!g1.passed) failReasons.push("G1(\u63A5\u53E3)");
  if (!g2.passed) failReasons.push("G2(\u898F\u683C)");
  if (!g3.passed) failReasons.push("G3(\u9632\u8B77)");
  if (!buildPass) failReasons.push("\u7DE8\u8B6F");
  if (!testPass) failReasons.push("\u6E2C\u8A66");
  if (!step10Result.passed) failReasons.push("\u4EA4\u4ED8\u8B49\u64DA");
  return {
    passed,
    tool: GATE_CIRCUIT_LABELS.G4.tool,
    details: passed ? "G4 PASS: \u9A57\u6536\u901A\u904E\uFF0C\u5168\u90E8\u6B63\u5E38\uFF0C\u53EF\u4EE5\u51FA\u8CA8" : `G4 FAIL: ${failReasons.join(" + ")} \u6709\u554F\u984C\uFF0C\u9A57\u6536\u4E0D\u901A\u904E`
  };
}
function runGatesV33(input) {
  const g1 = runG1(input);
  const g2 = runG2(input);
  const g3 = runG3(input);
  const g4 = runG4(input, g1, g2, g3);
  return {
    G1: g1,
    G2: g2,
    G3: g3,
    G4: g4,
    allPassed: g1.passed && g2.passed && g3.passed && g4.passed
  };
}

// src/engine/evidence.ts
function collectEvidence(steps, gates, input) {
  const dir = input.evidenceDir;
  const logs = {};
  logs["scan.log"] = {
    path: `${dir}/scan.log`,
    exists: steps[0]?.evidencePath !== void 0,
    lineCount: steps[0]?.stats?.violations ?? (steps[0]?.passed ? 0 : 1),
    zeroViolations: steps[0]?.passed ?? false,
    summary: steps[0]?.details ?? "N/A"
  };
  logs["fraud.log"] = {
    path: `${dir}/fraud.log`,
    exists: steps[1]?.evidencePath !== void 0,
    lineCount: steps[1]?.stats?.violations ?? (steps[1]?.passed ? 0 : 1),
    zeroViolations: steps[1]?.passed ?? false,
    summary: steps[1]?.details ?? "N/A"
  };
  logs["build.log"] = {
    path: `${dir}/build.log`,
    exists: steps[2]?.evidencePath !== void 0,
    lineCount: 0,
    zeroViolations: steps[2]?.passed ?? false,
    summary: steps[2]?.details ?? "N/A"
  };
  logs["lint.log"] = {
    path: `${dir}/lint.log`,
    exists: steps[3]?.evidencePath !== void 0,
    lineCount: 0,
    zeroViolations: steps[3]?.passed ?? false,
    summary: steps[3]?.details ?? "N/A"
  };
  logs["test.log"] = {
    path: `${dir}/test.log`,
    exists: steps[4]?.evidencePath !== void 0,
    lineCount: 0,
    zeroViolations: steps[4]?.passed ?? false,
    summary: steps[4]?.details ?? "N/A"
  };
  logs["coverage.log"] = {
    path: `${dir}/coverage.log`,
    exists: steps[5]?.evidencePath !== void 0,
    lineCount: 0,
    zeroViolations: steps[5]?.passed ?? false,
    summary: steps[5]?.details ?? "N/A"
  };
  logs["redline.log"] = {
    path: `${dir}/redline.log`,
    exists: steps[6]?.evidencePath !== void 0,
    lineCount: steps[6]?.stats?.violations ?? (steps[6]?.passed ? 0 : 1),
    zeroViolations: steps[6]?.passed ?? false,
    summary: steps[6]?.details ?? "N/A"
  };
  logs["sync.log"] = {
    path: `${dir}/sync.log`,
    exists: steps[7]?.evidencePath !== void 0,
    lineCount: steps[7]?.stats?.disconnects ?? (steps[7]?.passed ? 0 : 1),
    zeroViolations: steps[7]?.passed ?? false,
    summary: steps[7]?.details ?? "N/A"
  };
  logs["mapping.log"] = {
    path: `${dir}/mapping.log`,
    exists: steps[8]?.evidencePath !== void 0,
    lineCount: 0,
    zeroViolations: steps[8]?.passed ?? false,
    summary: steps[8]?.details ?? "N/A"
  };
  logs["impl.log"] = {
    path: `${dir}/impl.log`,
    exists: true,
    lineCount: 0,
    // Treat "no MISSING" as "impl complete" at v3.3 baseline.
    zeroViolations: steps[8]?.passed ?? false,
    summary: steps[8]?.passed ? "\u88DC\u5B8C\u8B49\u660E: \u898F\u683C\u51FD\u6578\u5747\u5DF2\u843D\u5730" : "\u88DC\u5B8C\u8B49\u660E: \u4ECD\u6709\u7F3A\u53E3 (\u898B mapping.log)"
  };
  const iav = input.proof?.iav;
  logs["iav.log"] = {
    path: `${dir}/iav.log`,
    exists: iav !== void 0,
    lineCount: iav ? iav.failedCount > 0 ? iav.failedCount : iav.passedCount : 0,
    zeroViolations: iav?.passed ?? false,
    summary: iav?.details ?? "\u26A0\uFE0F \u672A\u63D0\u4F9B IAV \u8B49\u64DA (evidence/iav.log)"
  };
  const blds = input.proof?.blds;
  logs["blds.log"] = {
    path: `${dir}/blds.log`,
    exists: blds !== void 0,
    lineCount: 0,
    zeroViolations: blds?.passed ?? false,
    summary: blds?.details ?? "\u26A0\uFE0F \u672A\u63D0\u4F9B BLDS \u8B49\u64DA (evidence/blds.log)"
  };
  const ds = input.proof?.datasource;
  logs["datasource.log"] = {
    path: `${dir}/datasource.log`,
    exists: ds !== void 0,
    lineCount: ds?.untraced ?? 0,
    zeroViolations: ds?.passed ?? false,
    summary: ds?.details ?? "\u26A0\uFE0F \u672A\u63D0\u4F9B datasource \u8B49\u64DA (evidence/datasource.log)"
  };
  logs["package.log"] = {
    path: `${dir}/package.log`,
    exists: input.externalResults?.package !== void 0,
    lineCount: 0,
    zeroViolations: input.externalResults?.package?.exitCode === 0,
    summary: input.externalResults?.package ? `package exit=${input.externalResults.package.exitCode}` : "\u26A0\uFE0F \u672A\u63D0\u4F9B package \u8B49\u64DA"
  };
  logs["run.log"] = {
    path: `${dir}/run.log`,
    exists: input.externalResults?.run !== void 0,
    lineCount: 0,
    zeroViolations: input.externalResults?.run?.exitCode === 0,
    summary: input.externalResults?.run ? `run exit=${input.externalResults.run.exitCode}` : "\u26A0\uFE0F \u672A\u63D0\u4F9B run \u8B49\u64DA"
  };
  logs["audit.log"] = {
    path: `${dir}/audit.log`,
    exists: input.externalResults?.audit !== void 0,
    lineCount: 0,
    zeroViolations: (input.externalResults?.audit?.critical ?? 1) === 0 && (input.externalResults?.audit?.high ?? 0) === 0,
    summary: input.externalResults?.audit ? `audit critical=${input.externalResults.audit.critical} high=${input.externalResults.audit.high}` : "\u26A0\uFE0F \u672A\u63D0\u4F9B audit \u8B49\u64DA"
  };
  return { logs, steps, gates, dir };
}
function judgeDod(evidence) {
  const { logs, gates } = evidence;
  const items = DOD_DEFINITIONS.map((def) => {
    let passed = false;
    let evidencePath;
    switch (def.id) {
      case 1:
        passed = logs["redline.log"]?.zeroViolations ?? false;
        evidencePath = logs["redline.log"]?.path;
        break;
      case 2:
        passed = (logs["mapping.log"]?.zeroViolations ?? false) && (logs["impl.log"]?.exists ?? false);
        evidencePath = logs["impl.log"]?.path;
        break;
      case 3:
        passed = gates.G2.passed;
        evidencePath = logs["mapping.log"]?.path;
        break;
      case 4:
        passed = logs["sync.log"]?.zeroViolations ?? false;
        evidencePath = logs["sync.log"]?.path;
        break;
      case 5:
        passed = logs["build.log"]?.zeroViolations ?? false;
        evidencePath = logs["build.log"]?.path;
        break;
      case 6:
        passed = (logs["package.log"]?.zeroViolations ?? false) && (logs["run.log"]?.zeroViolations ?? false) && gates.G4.passed;
        evidencePath = logs["package.log"]?.path;
        break;
      case 7:
        passed = (logs["iav.log"]?.zeroViolations ?? false) && (logs["blds.log"]?.zeroViolations ?? false);
        evidencePath = logs["iav.log"]?.path;
        break;
      case 8:
        passed = logs["fraud.log"]?.zeroViolations ?? false;
        evidencePath = logs["fraud.log"]?.path;
        break;
    }
    return {
      ...def,
      passed,
      evidencePath
    };
  });
  return {
    items,
    missionComplete: items.every((i) => i.passed)
  };
}
function generateProofPackManifest(evidence) {
  const lines = [];
  lines.push("# Proof Pack \u2014 Code-QC v3.3");
  lines.push(`# Generated: ${(/* @__PURE__ */ new Date()).toISOString()}`);
  lines.push(`# Directory: ${evidence.dir}`);
  lines.push("");
  lines.push("## Evidence Files");
  lines.push("");
  for (const [name, log] of Object.entries(evidence.logs)) {
    const icon = log.zeroViolations ? "\u2705" : "\u274C";
    lines.push(`${icon} ${name} \u2014 ${log.summary}`);
  }
  lines.push("");
  lines.push("## DoD Status");
  lines.push("");
  const dod = judgeDod(evidence);
  for (const item of dod.items) {
    const icon = item.passed ? "\u2705" : "\u274C";
    lines.push(`${icon} [${item.id}] ${item.name}: ${item.verification}`);
  }
  lines.push("");
  lines.push(`## Verdict: ${dod.missionComplete ? "MISSION COMPLETE \u2705" : "REJECTED \u274C"}`);
  return lines.join("\n");
}

// src/engine/protection.ts
function checkLV1(steps, _ctx) {
  const step7 = steps[6];
  const passed = step7?.passed ?? false;
  return {
    level: 1,
    name: PROTECTION_LEVELS.LV1.name,
    passed,
    details: passed ? "LV1 PASS: R01-R18 \u7D05\u7DDA = 0" : "LV1 FAIL: \u7D05\u7DDA\u9055\u898F\u672A\u6E05\u96F6"
  };
}
function checkLV2(steps, _ctx) {
  const step4 = steps[3];
  const passed = step4?.passed ?? false;
  return {
    level: 2,
    name: PROTECTION_LEVELS.LV2.name,
    passed,
    details: passed ? "LV2 PASS: P01-P14 \u7981\u6B62\u898F\u5247\u5728\u9650" : "LV2 FAIL: \u7981\u6B62\u898F\u5247\u8D85\u9650"
  };
}
function checkLV3(steps, _ctx) {
  const step1 = steps[0];
  const step2 = steps[1];
  const passed = (step1?.passed ?? false) && (step2?.passed ?? false);
  return {
    level: 3,
    name: PROTECTION_LEVELS.LV3.name,
    passed,
    details: passed ? "LV3 PASS: Z\u8EF8\u53CD\u8A50\u6B3A = 0" : "LV3 FAIL: \u8A50\u6B3A\u4FE1\u865F\u672A\u6E05\u96F6"
  };
}
function checkLV4(_steps, ctx) {
  const nonce = ctx?.nonce;
  const passed = typeof nonce === "string" && nonce.trim().length > 0;
  return {
    level: 4,
    name: PROTECTION_LEVELS.LV4.name,
    passed,
    details: passed ? "LV4 PASS: nonce \u5DF2\u7522\u51FA\u4E26\u8A18\u9304 (\u57FA\u790E\u6A21\u5F0F)" : "LV4 FAIL: \u7F3A\u5C11 nonce (\u9700 pipeline \u7522\u51FA anti-replay token)"
  };
}
function checkLV5(_steps, ctx) {
  const h = ctx?.evidenceHash;
  const passed = typeof h === "string" && /^[a-f0-9]{64}$/i.test(h);
  return {
    level: 5,
    name: PROTECTION_LEVELS.LV5.name,
    passed,
    details: passed ? "LV5 PASS: evidence hash \u5DF2\u7522\u51FA\u4E26\u8A18\u9304 (sha256)" : "LV5 FAIL: \u7F3A\u5C11 evidence hash (\u9700\u8F38\u51FA sha256\uFF0C\u9632\u62FC\u8CBC/\u7BE1\u6539)"
  };
}
function checkLV6(_steps, _ctx) {
  return { level: 6, name: PROTECTION_LEVELS.LV6.name, passed: false, details: "LV6 SKIP: \u7368\u7ACB\u6AA2\u6E2C\u7AD9\u9700\u5916\u90E8 Verifier \u6574\u5408" };
}
function checkLV7(_steps, _ctx) {
  return { level: 7, name: PROTECTION_LEVELS.LV7.name, passed: false, details: "LV7 SKIP: \u53EF\u4FE1\u6A21\u7D44\u9700 TEE/Attestation \u6574\u5408" };
}
function checkLV8(_steps, _ctx) {
  return { level: 8, name: PROTECTION_LEVELS.LV8.name, passed: false, details: "LV8 SKIP: \u4EA4\u53C9\u5C0D\u6297\u9700\u591A\u6A21\u578B Adversarial \u6574\u5408" };
}
function checkLV9(_steps, _ctx) {
  return { level: 9, name: PROTECTION_LEVELS.LV9.name, passed: false, details: "LV9 SKIP: \u5F62\u5F0F\u5316\u8B49\u660E\u9700\u5916\u90E8\u5DE5\u5177 (TLA+/Coq)" };
}
var LV_CHECKERS = [checkLV1, checkLV2, checkLV3, checkLV4, checkLV5, checkLV6, checkLV7, checkLV8, checkLV9];
function checkProtection(grade, steps, ctx) {
  const targetLevel = grade === "E" ? 5 : 9;
  const checks = LV_CHECKERS.map((fn) => fn(steps, ctx));
  let achievedLevel = 0;
  for (const check of checks) {
    if (check.passed) {
      achievedLevel = check.level;
    } else {
      break;
    }
  }
  const relevantChecks = checks.slice(0, targetLevel);
  const allPassed = relevantChecks.every((c) => c.passed);
  return {
    grade,
    targetLevel,
    achievedLevel,
    checks: relevantChecks,
    allPassed
  };
}
function resolveProtectionLevel(grade, steps, ctx) {
  const report = checkProtection(grade, steps, ctx);
  return report.achievedLevel;
}

// src/engine/waveform.ts
function buildYChannel(input) {
  const readings = [
    {
      id: "Y1",
      name: "\u898F\u683C\u6709\u5C0D\u61C9",
      amplitude: input.specMapped ? 100 : Math.max(0, 100 - input.specMissingCount * 20),
      status: input.specMapped ? "PASS" : "FAIL",
      evidence: "mapping.log"
    },
    {
      id: "Y2",
      name: "\u6E2C\u8A66\u6709\u904E",
      amplitude: input.testsTotal > 0 ? (input.testsTotal - input.testsFailed) / input.testsTotal * 100 : 0,
      status: input.testsPass ? "PASS" : "FAIL",
      evidence: "test.log"
    },
    {
      id: "Y3",
      name: "\u6E2C\u4E86\u591A\u5C11",
      amplitude: input.coveragePercent,
      status: input.coveragePercent >= input.coverageThreshold ? "PASS" : "WARN",
      evidence: "coverage.log"
    },
    {
      id: "Y4",
      name: "\u529F\u80FD\u5B8C\u6574",
      amplitude: input.implComplete ? 100 : 50,
      status: input.implComplete ? "PASS" : "WARN",
      evidence: "impl.log"
    }
  ];
  return finalizeChannel("CH1_Y", "\u529F\u80FD\u5B8C\u6574\u5EA6", readings);
}
function buildXChannel(input) {
  const readings = [
    {
      id: "X1",
      name: "\u7DE8\u8B6F\u4E7E\u6DE8",
      amplitude: input.buildErrors === 0 ? 100  : 0,
      status: input.buildErrors === 0 ? "PASS"  : "FAIL",
      evidence: "build.log"
    },
    {
      id: "X2",
      name: "\u98A8\u683C\u4E7E\u6DE8",
      amplitude: input.lintErrors === 0 ? 100  : 0,
      status: input.lintErrors === 0 ? "PASS"  : "FAIL",
      evidence: "lint.log"
    },
    {
      id: "X3",
      name: "\u6C92\u8E29\u7D05\u7DDA",
      amplitude: input.redlineViolations === 0 ? 100 : 0,
      status: input.redlineViolations === 0 ? "PASS" : "FAIL",
      evidence: "redline.log"
    },
    {
      id: "X4",
      name: "\u5B89\u5168\u7121\u865E",
      amplitude: input.securityCritical === 0 && input.securityHigh === 0 ? 100 : 0,
      status: input.securityCritical === 0 && input.securityHigh === 0 ? "PASS" : "FAIL",
      evidence: "audit.log"
    }
  ];
  return finalizeChannel("CH2_X", "\u7A0B\u5F0F\u78BC\u54C1\u8CEA", readings);
}
function buildZChannel(input) {
  const readings = [
    {
      id: "Z1",
      name: "\u6709\u6C92\u6709\u9020\u5047",
      amplitude: input.fraudCount === 0 ? 100 : 0,
      status: input.fraudCount === 0 ? "PASS" : "FAIL",
      evidence: "fraud.log"
    },
    {
      id: "Z2",
      name: "\u771F\u5BE6\u6027\u4E94\u554F",
      amplitude: input.iavPass ? 100 : 0,
      status: input.iavPass ? "PASS" : "FAIL",
      evidence: "iav.log"
    },
    {
      id: "Z3",
      name: "\u771F\u5BE6\u5EA6\u8A55\u5206",
      amplitude: input.bldsScore / 5 * 100,
      status: input.bldsScore >= input.bldsMinimum ? "PASS" : "FAIL",
      evidence: "blds.log",
      detail: `BLDS=${input.bldsScore}/5 (min=${input.bldsMinimum})`
    },
    {
      id: "Z4",
      name: "\u8CC7\u6599\u4F86\u6E90",
      amplitude: input.traceability ? 100 : 0,
      status: input.traceability ? "PASS" : "WARN",
      evidence: "datasource.log"
    }
  ];
  return finalizeChannel("CH3_Z", "\u771F\u5BE6\u5EA6", readings);
}
function finalizeChannel(channel, name, readings) {
  const hasAnyFail = readings.some((r) => r.status === "FAIL");
  const hasAnyWarn = readings.some((r) => r.status === "WARN");
  const score = readings.length > 0 ? readings.reduce((sum, r) => sum + r.amplitude, 0) / readings.length : 0;
  return {
    channel,
    name,
    readings,
    overall: hasAnyFail ? "FAIL" : hasAnyWarn ? "WARN" : "PASS",
    score: Math.round(score * 100) / 100
  };
}
function buildWaveformReport(y, x, z) {
  const allPass = y.overall === "PASS" && x.overall === "PASS" && z.overall === "PASS";
  const compositeScore = Math.round((y.score * 0.4 + x.score * 0.3 + z.score * 0.3) * 100) / 100;
  return {
    timestamp: (/* @__PURE__ */ new Date()).toISOString(),
    channels: [y, x, z],
    allPass,
    compositeScore
  };
}
var STEP_DEFS = [
  { id: 1, name: "\u9632\u507D\u6AA2\u67E5 (\u6709\u6C92\u6709\u5047\u7A0B\u5F0F\u78BC)", circuitTerm: "\u2778 \u9632\u507D\u6383\u63CF", pillar: "PROTECTION", fatalOnFail: true },
  { id: 2, name: "\u8A50\u6B3A\u6383\u63CF (\u6709\u6C92\u6709\u9020\u5047)", circuitTerm: "\u277A \u8A50\u6B3A\u6383\u63CF", pillar: "PROTECTION", fatalOnFail: true },
  { id: 3, name: "\u7DE8\u8B6F\u6AA2\u67E5 (\u80FD\u4E0D\u80FD\u7DE8\u8B6F)", circuitTerm: "\u2778 \u7DE8\u8B6F", pillar: "LOGIC_GATE", fatalOnFail: true },
  { id: 4, name: "\u98A8\u683C\u6AA2\u67E5 (\u6709\u6C92\u6709\u58DE\u7FD2\u6163)", circuitTerm: "\u2778 \u98A8\u683C", pillar: "LOGIC_GATE", fatalOnFail: false },
  { id: 5, name: "\u6E2C\u8A66\u6AA2\u67E5 (\u6E2C\u8A66\u6709\u6C92\u6709\u5168\u904E)", circuitTerm: "\u2779 \u6E2C\u8A66", pillar: "INSTRUMENT", fatalOnFail: true },
  { id: 6, name: "\u8986\u84CB\u7387 (\u6E2C\u8A66\u6DB5\u84CB\u591A\u5C11)", circuitTerm: "\u2779 \u8986\u84CB\u7387", pillar: "INSTRUMENT", fatalOnFail: false },
  { id: 7, name: "\u7D05\u7DDA\u5168\u6AA2 (\u6709\u6C92\u6709\u9055\u53CD\u7D05\u7DDA)", circuitTerm: "\u2778 \u7D05\u7DDA\u5168\u6AA2", pillar: "PROTECTION", fatalOnFail: true },
  { id: 8, name: "G1 \u63A5\u53E3\u540C\u6B65 (\u6709\u6C92\u6709\u5C0D\u4E0A)", circuitTerm: "\u2779 G1\u540C\u6B65", pillar: "LOGIC_GATE", fatalOnFail: true },
  { id: 9, name: "G2 \u898F\u683C\u8986\u84CB (\u529F\u80FD\u6709\u6C92\u6709\u505A\u5B8C)", circuitTerm: "\u2779 G2\u898F\u683C", pillar: "LOGIC_GATE", fatalOnFail: true },
  { id: 10, name: "G4 \u6700\u7D42\u9A57\u6536", circuitTerm: "\u2779 G4\u9A57\u6536", pillar: "INSTRUMENT", fatalOnFail: true }
];
function isTestPath(path) {
  const norm = path.replace(/\\/g, "/");
  return /(^|\/)(tests?|specs?|__tests__|__test__|__specs__|__spec__|fixtures|__mocks__|mocks|mock|__snapshots__)(\/|$)/i.test(norm) || /\.(?:test|spec)\.[^/]+$/i.test(norm);
}
function step01_fuseCheck(input) {
  const t0 = performance.now();
  const fraudViolations = [];
  for (const f of input.files) {
    if (isTestPath(f.path)) continue;
    if (/web-ui[/\\]/i.test(f.path)) continue;
    const subset = checkRedlines(f.content, f.path).filter((v) => v.ruleId === "R13" || v.ruleId === "R14" || v.ruleId === "R15" || v.ruleId === "R16" || v.ruleId === "R17" || v.ruleId === "R18");
    fraudViolations.push(...subset);
  }
  const passed = fraudViolations.length === 0;
  const log = passed ? "" : fraudViolations.map((v) => `${v.ruleId} ${v.file}:${v.line}:${v.column ?? 1} ${v.message}`).join("\n");
  return {
    step: 1,
    name: STEP_DEFS[0].name,
    circuitTerm: STEP_DEFS[0].circuitTerm,
    passed,
    duration: Math.round(performance.now() - t0),
    details: passed ? "\u9632\u507D\u901A\u904E: \u6C92\u6709\u5047\u7A0B\u5F0F\u78BC" : `\u{1F534} \u9632\u507D\u5931\u6557! ${fraudViolations.length} \u500B\u5047\u5BE6\u73FE \u2014 ${fraudViolations.map((v) => `${v.ruleId}@${v.file}:${v.line}`).join(", ")}`,
    faultMode: passed ? void 0 : "SHORT_CIRCUIT",
    evidencePath: "evidence/scan.log",
    log,
    violations: fraudViolations,
    stats: { violations: fraudViolations.length }
  };
}
function step02_esdProtection(input) {
  const t0 = performance.now();
  const fraudViolations = [];
  for (const f of input.files) {
    if (isTestPath(f.path)) continue;
    if (/web-ui[/\\]/i.test(f.path)) continue;
    fraudViolations.push(...checkAntifraud(f.content, f.path));
  }
  const passed = fraudViolations.length === 0;
  const log = passed ? "" : fraudViolations.map((v) => `${v.ruleId} ${v.file}:${v.line}:${v.column ?? 1} ${v.message}`).join("\n");
  return {
    step: 2,
    name: STEP_DEFS[1].name,
    circuitTerm: STEP_DEFS[1].circuitTerm,
    passed,
    duration: Math.round(performance.now() - t0),
    details: passed ? "\u8A50\u6B3A\u6383\u63CF\u901A\u904E: \u6C92\u6709\u9020\u5047" : `\u{1F534} \u767C\u73FE\u9020\u5047! ${fraudViolations.length} \u500B\u8A50\u6B3A\u554F\u984C (R16/R17/R18)`,
    faultMode: passed ? void 0 : "SHORT_CIRCUIT",
    evidencePath: "evidence/fraud.log",
    log,
    violations: fraudViolations,
    stats: { violations: fraudViolations.length }
  };
}
function step03_solder(input) {
  const t0 = performance.now();
  const ext = input.externalResults?.build;
  if (!ext) {
    return {
      step: 3,
      name: STEP_DEFS[2].name,
      circuitTerm: STEP_DEFS[2].circuitTerm,
      passed: false,
      duration: Math.round(performance.now() - t0),
      details: "\u26A0\uFE0F \u6C92\u6709\u7DE8\u8B6F\u7D50\u679C (\u7528 --build \u6307\u5B9A\u7DE8\u8B6F\u6307\u4EE4\uFF0C\u6216\u8B93 CLI \u81EA\u52D5\u5075\u6E2C)",
      faultMode: "OPEN_CIRCUIT"
    };
  }
  const passed = ext.exitCode === 0;
  return {
    step: 3,
    name: STEP_DEFS[2].name,
    circuitTerm: STEP_DEFS[2].circuitTerm,
    passed,
    duration: Math.round(performance.now() - t0),
    details: passed ? "\u7DE8\u8B6F\u901A\u904E: 0 \u932F\u8AA4 0 \u8B66\u544A" : `\u{1F534} \u7DE8\u8B6F\u5931\u6557! (exit ${ext.exitCode})`,
    faultMode: passed ? void 0 : "COLD_SOLDER_JOINT",
    evidencePath: "evidence/build.log"
  };
}
function step04_wash(input) {
  const t0 = performance.now();
  const ext = input.externalResults?.lint;
  if (!ext) {
    const violations = [];
    for (const f of input.files) {
      violations.push(...checkProhibitions(f.content, f.path));
    }
    const passed2 = violations.filter((v) => v.severity === "error").length === 0;
    return {
      step: 4,
      name: STEP_DEFS[3].name,
      circuitTerm: STEP_DEFS[3].circuitTerm,
      passed: passed2,
      duration: Math.round(performance.now() - t0),
      details: passed2 ? `\u98A8\u683C\u6AA2\u67E5\u901A\u904E: \u7981\u6B62\u898F\u5247\u6383\u63CF ${violations.length} warnings` : `\u{1F7E1} \u98A8\u683C\u6709\u554F\u984C: ${violations.length} \u500B\u9055\u898F (\u7981\u6B62\u898F\u5247)`
    };
  }
  const passed = ext.exitCode === 0;
  return {
    step: 4,
    name: STEP_DEFS[3].name,
    circuitTerm: STEP_DEFS[3].circuitTerm,
    passed,
    duration: Math.round(performance.now() - t0),
    details: passed ? "\u98A8\u683C\u6AA2\u67E5\u901A\u904E: Lint 0 \u932F\u8AA4 0 \u8B66\u544A" : `\u{1F7E1} \u98A8\u683C\u6709\u554F\u984C: Lint \u6709\u932F (exit ${ext.exitCode})`,
    evidencePath: "evidence/lint.log"
  };
}
function step05_jointTest(input) {
  const t0 = performance.now();
  const ext = input.externalResults?.test;
  if (!ext) {
    return {
      step: 5,
      name: STEP_DEFS[4].name,
      circuitTerm: STEP_DEFS[4].circuitTerm,
      passed: false,
      duration: Math.round(performance.now() - t0),
      details: "\u26A0\uFE0F \u6C92\u6709\u6E2C\u8A66\u7D50\u679C (\u7528 --test \u6307\u5B9A\u6E2C\u8A66\u6307\u4EE4\uFF0C\u6216\u8B93 CLI \u81EA\u52D5\u5075\u6E2C)",
      faultMode: "OPEN_CIRCUIT"
    };
  }
  const passed = ext.exitCode === 0 && ext.failed === 0 || ext.failed === 0 && ext.passed > 0;
  return {
    step: 5,
    name: STEP_DEFS[4].name,
    circuitTerm: STEP_DEFS[4].circuitTerm,
    passed,
    duration: Math.round(performance.now() - t0),
    details: passed ? `\u6E2C\u8A66\u901A\u904E: ${ext.passed} passed, 0 failed` : `\u{1F534} \u6E2C\u8A66\u5931\u6557! ${ext.failed} tests failed`,
    faultMode: passed ? void 0 : "COLD_SOLDER_JOINT",
    evidencePath: "evidence/test.log"
  };
}
function step06_spectrum(input) {
  const t0 = performance.now();
  const ext = input.externalResults?.coverage;
  const threshold = 80;
  if (!ext) {
    return {
      step: 6,
      name: STEP_DEFS[5].name,
      circuitTerm: STEP_DEFS[5].circuitTerm,
      passed: false,
      duration: Math.round(performance.now() - t0),
      details: "\u26A0\uFE0F \u6C92\u6709\u8986\u84CB\u7387\u7D50\u679C"
    };
  }
  const passed = ext.percentage >= threshold;
  return {
    step: 6,
    name: STEP_DEFS[5].name,
    circuitTerm: STEP_DEFS[5].circuitTerm,
    passed,
    duration: Math.round(performance.now() - t0),
    details: passed ? `\u8986\u84CB\u7387\u9054\u6A19: ${ext.percentage}% \u2265 ${threshold}%` : `\u{1F7E1} \u8986\u84CB\u7387\u4E0D\u8DB3: ${ext.percentage}% < ${threshold}%`,
    evidencePath: "evidence/coverage.log"
  };
}
function step07_fuseFullCheck(input) {
  const t0 = performance.now();
  const violations = [];
  for (const f of input.files) {
    if (/(?:b-redlines|b-prohibitions|c-gates)[/\\.]/.test(f.path)) continue;
    if (isTestPath(f.path)) continue;
    if (/web-ui[/\\]/i.test(f.path)) continue;
    violations.push(...checkRedlines(f.content, f.path));
  }
  const errors = violations.filter((v) => v.severity === "error");
  const passed = errors.length === 0;
  const log = passed ? "" : errors.map((v) => `${v.ruleId} ${v.file}:${v.line}:${v.column ?? 1} ${v.message}`).join("\n");
  return {
    step: 7,
    name: STEP_DEFS[6].name,
    circuitTerm: STEP_DEFS[6].circuitTerm,
    passed,
    duration: Math.round(performance.now() - t0),
    details: passed ? `\u7D05\u7DDA\u5168\u6AA2\u901A\u904E: R01-R18 = 0 \u9055\u898F` : `\u{1F534} \u7D05\u7DDA\u9055\u898F! ${errors.length} \u689D \u2014 ${errors.slice(0, 5).map((v) => v.ruleId).join(",")}`,
    faultMode: passed ? void 0 : "SHORT_CIRCUIT",
    evidencePath: "evidence/redline.log",
    log,
    violations: errors,
    stats: { violations: errors.length }
  };
}
function step08_g1Contact(input) {
  const t0 = performance.now();
  const disconnects = [];
  for (const f of input.files) {
    if (/(?:engine|rules)[/\\]/i.test(f.path)) continue;
    if (isTestPath(f.path)) continue;
    const lines = f.content.split("\n");
    lines.forEach((line, i) => {
      if (/\bDISCONNECTED\b/.test(line)) {
        disconnects.push(`${f.path}:${i + 1}`);
      } else if (/\bTODO\b.*\bconnect\b/i.test(line) && !/regex|pattern|test|detect/i.test(line)) {
        disconnects.push(`${f.path}:${i + 1}`);
      } else if (/\bFIXME\b.*\bwire\b/i.test(line) && !/regex|pattern|test|detect/i.test(line)) {
        disconnects.push(`${f.path}:${i + 1}`);
      }
    });
  }
  const passed = disconnects.length === 0;
  const log = passed ? "" : disconnects.join("\n");
  return {
    step: 8,
    name: STEP_DEFS[7].name,
    circuitTerm: STEP_DEFS[7].circuitTerm,
    passed,
    duration: Math.round(performance.now() - t0),
    details: passed ? "G1 \u63A5\u53E3\u6B63\u5E38: \u6C92\u6709\u65B7\u958B\u7684\u529F\u80FD" : `\u{1F534} G1 \u6709\u529F\u80FD\u65B7\u958B! ${disconnects.length} \u500B DISCONNECTED \u2014 ${disconnects.slice(0, 3).join(", ")}`,
    faultMode: passed ? void 0 : "OPEN_CIRCUIT",
    evidencePath: "evidence/sync.log",
    log,
    stats: { disconnects: disconnects.length }
  };
}
function step09_g2Continuity(input) {
  const t0 = performance.now();
  const specTotal = input.specChecklist?.total ?? 0;
  const specDone = input.specChecklist?.done ?? 0;
  const specPct = specTotal > 0 ? Math.round(specDone / specTotal * 100) : 0;
  const specChecklistOk = specTotal > 0 && specDone === specTotal;
  if (!input.specFunctions || input.specFunctions.length === 0) {
    return {
      step: 9,
      name: STEP_DEFS[8].name,
      circuitTerm: STEP_DEFS[8].circuitTerm,
      passed: false,
      duration: Math.round(performance.now() - t0),
      details: "\u26A0\uFE0F G2 \u898F\u683C\u6838\u5C0D: \u627E\u4E0D\u5230 SPEC \u51FD\u6578\u6E05\u55AE (\u7528 --spec \u6307\u5411 SPEC.md\uFF0C\u4E26\u7528 `\u2192 `\u6A19\u8A3B\u51FD\u6578)",
      faultMode: "OPEN_CIRCUIT",
      evidencePath: "evidence/mapping.log",
      log: "MISSING: <no spec functions extracted>",
      stats: { specTotal, specDone, specPct, missing: 0 }
    };
  }
  const allSource = input.files.map((f) => f.content).join("\n");
  const missing = input.specFunctions.filter((fn) => !allSource.includes(fn));
  const passed = specChecklistOk && missing.length === 0;
  const log = missing.length === 0 ? "" : missing.map((fn) => `MISSING: ${fn}`).join("\n");
  return {
    step: 9,
    name: STEP_DEFS[8].name,
    circuitTerm: STEP_DEFS[8].circuitTerm,
    passed,
    duration: Math.round(performance.now() - t0),
    details: passed ? `G2 \u898F\u683C\u6838\u5C0D: SPEC=${specPct}% \xB7 ${input.specFunctions.length}/${input.specFunctions.length} \u51FD\u6578\u90FD\u6709\u5BE6\u4F5C` : `\u{1F534} G2 FAIL: SPEC=${specPct}% (${specDone}/${specTotal}) \xB7 \u7F3A ${missing.length} \u500B\u51FD\u6578 \u2014 ${missing.slice(0, 5).join(", ")}`,
    faultMode: passed ? void 0 : "OPEN_CIRCUIT",
    evidencePath: "evidence/mapping.log",
    log,
    stats: { specTotal, specDone, specPct, missing: missing.length }
  };
}
function step10_g4PowerOn(_input, prevSteps) {
  const t0 = performance.now();
  const fatalFails = prevSteps.filter((s, i) => STEP_DEFS[i].fatalOnFail && !s.passed);
  const pkg = _input.externalResults?.package;
  const run = _input.externalResults?.run;
  const deliveryProvided = Boolean(pkg) && Boolean(run);
  const deliveryPassed = deliveryProvided && pkg.exitCode === 0 && run.exitCode === 0;
  const passed = fatalFails.length === 0 && deliveryPassed;
  return {
    step: 10,
    name: STEP_DEFS[9].name,
    circuitTerm: STEP_DEFS[9].circuitTerm,
    passed,
    duration: Math.round(performance.now() - t0),
    details: passed ? "\u2705 G4 \u9A57\u6536\u901A\u904E: \u5168\u90E8\u5408\u683C\uFF0C\u53EF\u4EE5\u51FA\u8CA8" : !deliveryProvided ? "\u26A0\uFE0F G4 \u7F3A\u4EA4\u4ED8\u8B49\u64DA (\u9700\u8981 --package-cmd + --run-cmd)" : `\u{1F534} G4 \u9A57\u6536\u4E0D\u901A\u904E! \u95DC\u9375\u6B65\u9A5F\u5931\u6557=${fatalFails.length} \xB7 package=${pkg.exitCode} run=${run.exitCode}`,
    faultMode: passed ? void 0 : "OPEN_CIRCUIT",
    evidencePath: "evidence/g4_package.log",
    log: deliveryProvided ? `package.exit=${pkg.exitCode}
run.exit=${run.exitCode}` : "MISSING: package/run",
    stats: {
      fatalFails: fatalFails.length,
      deliveryProvided: deliveryProvided ? 1 : 0,
      packageExit: pkg?.exitCode ?? 999,
      runExit: run?.exitCode ?? 999
    }
  };
}
var STEP_RUNNERS = [
  (input) => step01_fuseCheck(input),
  (input) => step02_esdProtection(input),
  (input) => step03_solder(input),
  (input) => step04_wash(input),
  (input) => step05_jointTest(input),
  (input) => step06_spectrum(input),
  (input) => step07_fuseFullCheck(input),
  (input) => step08_g1Contact(input),
  (input) => step09_g2Continuity(input),
  (input, prev) => step10_g4PowerOn(input, prev)
];
function runPipeline(input) {
  const t0 = performance.now();
  const steps = [];
  const timestamp = (/* @__PURE__ */ new Date()).toISOString();
  const nonce = input.nonce && input.nonce.trim().length > 0 ? input.nonce.trim() : randomUUID();
  for (let i = 0; i < STEP_RUNNERS.length; i++) {
    const runner = STEP_RUNNERS[i];
    const result = runner(input, steps);
    steps.push(result);
  }
  const gateInput = {
    step8Result: steps[7],
    // G1
    step9Result: steps[8],
    // G2
    step10Result: steps[9],
    // G4
    allSteps: steps,
    files: input.files
  };
  const gates = runGatesV33(gateInput);
  const evidence = collectEvidence(steps, gates, input);
  const dod = judgeDod(evidence);
  const ext = input.externalResults;
  const hashPayload = JSON.stringify({
    version: "3.3",
    timestamp,
    targetPath: input.targetPath,
    grade: input.grade,
    nonce,
    external: {
      build: ext?.build?.log ?? "",
      lint: ext?.lint?.log ?? "",
      test: ext?.test?.log ?? "",
      coverage: ext?.coverage?.log ?? "",
      audit: ext?.audit?.log ?? "",
      package: ext?.package?.log ?? "",
      run: ext?.run?.log ?? ""
    },
    internal: steps.map((s) => ({
      step: s.step,
      passed: s.passed,
      evidencePath: s.evidencePath ?? "",
      log: s.log ?? ""
    })),
    proof: {
      iav: input.proofContent?.iavLog ?? "",
      blds: input.proofContent?.bldsLog ?? "",
      datasource: input.proofContent?.datasourceLog ?? ""
    }
  });
  const evidenceHash = createHash("sha256").update(hashPayload, "utf8").digest("hex");
  const protectionTarget = input.grade === "E" ? 5 : 9;
  const protectionLevel = resolveProtectionLevel(input.grade, steps, { nonce, evidenceHash });
  const passed = steps[9].passed && gates.allPassed && dod.missionComplete && protectionLevel >= protectionTarget;
  const z = input.proof;
  const iavPass = z?.iav?.passed ?? false;
  const bldsScore = z?.blds?.minScore ?? 0;
  const bldsMin = z?.blds?.threshold ?? (input.grade === "F" ? 4 : 3);
  const traceability = z?.datasource?.passed ?? false;
  const yChannel = buildYChannel({
    specMapped: steps[8].passed,
    // G2 走線連通 = SPEC映射
    specMissingCount: steps[8].stats?.missing ?? 0,
    testsPass: steps[4].passed,
    // Step5 焊點測試
    testsFailed: ext?.test?.failed ?? 0,
    testsTotal: (ext?.test?.passed ?? 0) + (ext?.test?.failed ?? 0),
    coveragePercent: ext?.coverage?.percentage ?? 0,
    coverageThreshold: input.grade === "F" ? 90 : 80,
    implComplete: steps[8].passed
    // G2
  });
  const xChannel = buildXChannel({
    buildErrors: ext?.build?.exitCode === 0 ? 0 : 1,
    lintErrors: ext?.lint?.exitCode === 0 ? 0 : 1,
    redlineViolations: steps[6].stats?.violations ?? (steps[6].passed ? 0 : 1),
    // Step7 紅線全檢
    securityCritical: ext?.audit ? ext.audit.critical : 1,
    securityHigh: ext?.audit ? ext.audit.high : 0
  });
  const zChannel = buildZChannel({
    fraudCount: steps[1].stats?.violations ?? (steps[1].passed ? 0 : 1),
    iavPass,
    bldsScore,
    bldsMinimum: bldsMin,
    traceability
  });
  const waveform = buildWaveformReport(yChannel, xChannel, zChannel);
  return {
    version: "3.3",
    timestamp,
    targetPath: input.targetPath,
    productGrade: input.grade,
    protectionLevel,
    protectionTarget,
    nonce,
    evidenceHash,
    steps,
    gates,
    dod,
    waveform,
    passed,
    duration: Math.round(performance.now() - t0),
    evidenceDir: input.evidenceDir
  };
}
function formatPipelineReport(result) {
  const lines = [];
  lines.push("");
  lines.push("\u2554\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2557");
  lines.push("\u2551  Code-QC v3.3 \u54C1\u8CEA\u6AA2\u6E2C\u53F0 (Test Bench)                   \u2551");
  lines.push("\u2551  \u7A0B\u5F0F\u54C1\u8CEA\uFF0C\u7528\u786C\u9AD4\u6A19\u6E96\u4F86\u9A57\u3002                              \u2551");
  lines.push("\u255A\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u255D");
  lines.push("");
  lines.push(`\u{1F4C2} Target: ${result.targetPath}`);
  lines.push(`\u{1F3F7}\uFE0F  Grade: ${result.productGrade} (${result.productGrade === "E" ? "\u5546\u7528\u7D1A" : "\u6DF1\u79D1\u6280\u7D1A"})`);
  lines.push(`\u{1F512} Protection: LV1-${result.protectionLevel} (target=LV${result.protectionTarget})`);
  lines.push(`\u{1F9F7} Nonce (LV4): ${result.nonce}`);
  lines.push(`\u{1F50F} Evidence Hash (LV5): ${result.evidenceHash.substring(0, 16)}\u2026`);
  lines.push("");
  lines.push("\u2500\u2500\u2500 Pipeline \u5341\u6B65\u6AA2\u67E5 \u2500\u2500\u2500");
  lines.push("");
  for (const step of result.steps) {
    const icon = step.passed ? "\u2705" : "\u274C";
    const duration = `${step.duration}ms`;
    lines.push(`  ${icon} [${String(step.step).padStart(2, " ")}/10] ${step.circuitTerm} \u2014 ${step.name} (${duration})`);
    if (!step.passed) {
      lines.push(`         ${step.details}`);
    }
  }
  lines.push("");
  lines.push("\u2500\u2500\u2500 G1-G4 \u56DB\u9053\u54C1\u8CEA\u95DC\u5361 \u2500\u2500\u2500");
  lines.push("");
  const gateEntries = [
    ["G1", "\u63A5\u53E3\u5B8C\u6574", "\u9010\u9EDE\u6AA2\u67E5", result.gates.G1],
    ["G2", "\u898F\u683C\u8986\u84CB", "\u9023\u901A\u6E2C\u8A66", result.gates.G2],
    ["G3", "\u9632\u8B77\u5230\u4F4D", "\u58D3\u529B\u6E2C\u8A66", result.gates.G3],
    ["G4", "\u6574\u9AD4\u9A57\u6536", "\u7D9C\u5408\u5224\u5B9A", result.gates.G4]
  ];
  for (const [id, name, tool, gate] of gateEntries) {
    const icon = gate.passed ? "\u2705" : "\u274C";
    lines.push(`  ${icon} ${id} ${name} (${tool}): ${gate.details}`);
  }
  lines.push("");
  lines.push("\u2500\u2500\u2500 DoD 8 \u9EDE\u4EA4\u4ED8\u78BA\u8A8D \u2500\u2500\u2500");
  lines.push("");
  for (const item of result.dod.items) {
    const icon = item.passed ? "\u2705" : "\u274C";
    lines.push(`  ${icon} [${item.id}] ${item.name}: ${item.verification}`);
  }
  lines.push("");
  const passCount = result.steps.filter((s) => s.passed).length;
  const dodCount = result.dod.items.filter((i) => i.passed).length;
  lines.push("\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550");
  if (result.passed) {
    lines.push(`  \u2705 MISSION COMPLETE \u2014 Pipeline ${passCount}/10 \xB7 Gates ${result.gates.allPassed ? "4/4" : "FAIL"} \xB7 DoD ${dodCount}/8`);
    lines.push("  \u5168\u90E8\u5408\u683C\uFF0C\u53EF\u4EE5\u51FA\u8CA8\u3002");
  } else {
    lines.push(`  \u274C REJECTED \u2014 Pipeline ${passCount}/10 \xB7 Gates ${result.gates.allPassed ? "4/4" : "FAIL"} \xB7 DoD ${dodCount}/8`);
    lines.push("  \u6709\u554F\u984C\uFF0C\u9700\u8981\u4FEE\u5FA9\u518D\u4EA4\u3002");
  }
  lines.push("\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550");
  lines.push("");
  return lines.join("\n");
}

// src/index.ts
var VERSION = "0.3.3";

export { ANTI_FRAUD_CHECKERS, AXIOMS, AXIOMS_BY_PRIORITY, BLDS_GATE_MINIMUM, BLDS_LEVELS, CIRCUIT_QUICK_CARD, CIRCUIT_WORLDVIEW, CODEQC_VERSION, DEFAULT_CONFIG, DOD_DEFINITIONS, FAULT_MODES, GATES, GATE_CIRCUIT_LABELS, HANDOVER_TAGS, HANDOVER_TEMPLATE, HARDWARE_QUICK_CARD, HARDWARIZATION_PILLARS, IAV_DISQUALIFIERS, PRODUCT_GRADES, PROHIBITIONS, PROHIBITION_CHECKERS, PROTECTION_COMPONENTS, PROTECTION_LAYERS, PROTECTION_LEVELS, REDLINES, REDLINE_CHECKERS, VERSION, analyze, analyzeFile, calculateComplianceScore, calculateDualAxisScore, calculateOutcomeScore, checkAntifraud, checkFraud, checkProhibitions, checkProtection, checkRedlines, checkRules, collectEvidence, consoleReporter, createGateStatus, detectLanguage, evaluateGate, extractHandoverTags, formatAxiomsPrompt, formatPipelineReport, generateAllGatesChecklist, generateGateChecklist, generateProofPackManifest, getAxiom, getProhibition, getRedline, getReporter, htmlReporter, isSupported, jsonReporter, judgeDod, quickCheck, reporters, resolveProtectionLevel, runGatesV33, runPipeline };
//# sourceMappingURL=index.js.map
//# sourceMappingURL=index.js.map