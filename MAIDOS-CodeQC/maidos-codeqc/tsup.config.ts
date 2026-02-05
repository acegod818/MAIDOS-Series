import { defineConfig } from 'tsup';

export default defineConfig({
  entry: {
    index: 'src/index.ts',
    cli: 'src/cli.ts',
  },
  format: ['esm', 'cjs'],
  dts: true,
  sourcemap: true,
  clean: true,
  splitting: false,
  treeshake: true,
  minify: false,
  target: 'node18',
  outDir: 'dist',
  external: [
    'tree-sitter-typescript',
    'tree-sitter-javascript', 
    'tree-sitter-python',
    'tree-sitter-rust',
    'tree-sitter-go',
  ],
  banner: {
    js: '#!/usr/bin/env node',
  },
  esbuildOptions(options) {
    options.banner = {
      js: options.entryPoints?.toString().includes('cli') 
        ? '#!/usr/bin/env node' 
        : '',
    };
  },
});
