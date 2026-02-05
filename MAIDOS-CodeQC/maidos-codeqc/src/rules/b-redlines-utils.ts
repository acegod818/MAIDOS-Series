export function maskJsStringsAndComments(source: string): string {
  const chars = source.split('');
  type State = 'normal' | 'single' | 'double' | 'template' | 'line_comment' | 'block_comment';
  let state: State = 'normal';

  for (let i = 0; i < chars.length; i++) {
    const c = chars[i]!;
    const n = chars[i + 1];

    if (state === 'normal') {
      if (c === '/' && n === '/') {
        chars[i] = ' ';
        chars[i + 1] = ' ';
        i++;
        state = 'line_comment';
        continue;
      }
      if (c === '/' && n === '*') {
        chars[i] = ' ';
        chars[i + 1] = ' ';
        i++;
        state = 'block_comment';
        continue;
      }
      if (c === '\'') { state = 'single'; continue; }
      if (c === '"') { state = 'double'; continue; }
      if (c === '`') { state = 'template'; continue; }
      continue;
    }

    if (state === 'line_comment') {
      if (c === '\n') { state = 'normal'; continue; }
      if (c !== '\r') chars[i] = ' ';
      continue;
    }

    if (state === 'block_comment') {
      if (c === '*' && n === '/') {
        chars[i] = ' ';
        chars[i + 1] = ' ';
        i++;
        state = 'normal';
        continue;
      }
      if (c !== '\n') chars[i] = ' ';
      continue;
    }

    if (state === 'single') {
      if (c === '\\') {
        chars[i] = ' ';
        if (n && n !== '\n') chars[i + 1] = ' ';
        i++;
        continue;
      }
      if (c === '\n') { state = 'normal'; continue; }
      if (c === '\'') { state = 'normal'; continue; }
      chars[i] = ' ';
      continue;
    }

    if (state === 'double') {
      if (c === '\\') {
        chars[i] = ' ';
        if (n && n !== '\n') chars[i + 1] = ' ';
        i++;
        continue;
      }
      if (c === '\n') { state = 'normal'; continue; }
      if (c === '"') { state = 'normal'; continue; }
      chars[i] = ' ';
      continue;
    }

    // template
    if (c === '\\') {
      chars[i] = ' ';
      if (n && n !== '\n') chars[i + 1] = ' ';
      i++;
      continue;
    }
    if (c === '`') { state = 'normal'; continue; }
    if (c !== '\n') chars[i] = ' ';
  }

  return chars.join('');
}

function maskRustStringsAndComments(source: string): string {
  const chars = source.split('');
  type State = 'normal' | 'line_comment' | 'block_comment' | 'string' | 'raw_string';
  let state: State = 'normal';
  let blockDepth = 0;
  let rawHashes = 0;

  for (let i = 0; i < chars.length; i++) {
    const c = chars[i]!;
    const n = chars[i + 1];

    if (state === 'normal') {
      if (c === '/' && n === '/') {
        chars[i] = ' ';
        chars[i + 1] = ' ';
        i++;
        state = 'line_comment';
        continue;
      }
      if (c === '/' && n === '*') {
        chars[i] = ' ';
        chars[i + 1] = ' ';
        i++;
        blockDepth = 1;
        state = 'block_comment';
        continue;
      }

      // Raw strings: r".." / r#".."# / br#".."#
      if (c === 'r' || (c === 'b' && n === 'r')) {
        let j = i;
        if (c === 'b') j += 2;
        else j += 1;

        let hashes = 0;
        while (chars[j] === '#') { hashes++; j++; }
        if (chars[j] === '"') {
          rawHashes = hashes;
          state = 'raw_string';
          i = j; // jump to the opening quote
          continue;
        }
      }

      // Byte strings: b"..."
      if (c === 'b' && n === '"') {
        // Keep the prefix 'b', enter normal string at the quote.
        state = 'string';
        i = i + 1;
        continue;
      }

      if (c === '"') {
        state = 'string';
        continue;
      }

      continue;
    }

    if (state === 'line_comment') {
      if (c === '\n') { state = 'normal'; continue; }
      if (c !== '\r') chars[i] = ' ';
      continue;
    }

    if (state === 'block_comment') {
      if (c === '/' && n === '*') {
        chars[i] = ' ';
        chars[i + 1] = ' ';
        i++;
        blockDepth++;
        continue;
      }
      if (c === '*' && n === '/') {
        chars[i] = ' ';
        chars[i + 1] = ' ';
        i++;
        blockDepth--;
        if (blockDepth === 0) state = 'normal';
        continue;
      }
      if (c !== '\n') chars[i] = ' ';
      continue;
    }

    if (state === 'string') {
      if (c === '\\') {
        chars[i] = ' ';
        if (n && n !== '\n') chars[i + 1] = ' ';
        i++;
        continue;
      }
      if (c === '"') { state = 'normal'; continue; }
      if (c !== '\n') chars[i] = ' ';
      continue;
    }

    // raw_string
    if (c === '"') {
      let ok = true;
      for (let k = 0; k < rawHashes; k++) {
        if (chars[i + 1 + k] !== '#') { ok = false; break; }
      }
      if (ok) {
        state = 'normal';
        i = i + rawHashes; // skip trailing hashes
        continue;
      }
    }
    if (c !== '\n') chars[i] = ' ';
  }

  return chars.join('');
}

export function stripRustCfgTestBlocks(source: string): string {
  const normalized = source.replace(/\r\n/g, '\n');
  const masked = maskRustStringsAndComments(normalized);
  const out = normalized.split('');

  const cfgRe = /#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]/g;
  let match;
  while ((match = cfgRe.exec(masked)) !== null) {
    const start = match.index;
    const openBrace = masked.indexOf('{', cfgRe.lastIndex);
    if (openBrace === -1) continue;

    let depth = 0;
    let end = -1;
    for (let i = openBrace; i < masked.length; i++) {
      const ch = masked[i]!;
      if (ch === '{') depth++;
      else if (ch === '}') {
        depth--;
        if (depth === 0) {
          end = i + 1;
          break;
        }
      }
    }
    if (end === -1) continue;

    for (let i = start; i < end; i++) {
      if (out[i] !== '\n') out[i] = ' ';
    }

    cfgRe.lastIndex = end;
  }

  return out.join('');
}

