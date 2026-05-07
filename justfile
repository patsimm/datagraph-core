default: build

build:
  wasm-pack build --target web
  node -e " \
    const fs = require('fs'); \
    const p = JSON.parse(fs.readFileSync('pkg/package.json', 'utf8')); \
    p.name = '@datagraph/core'; \
    fs.writeFileSync('pkg/package.json', JSON.stringify(p, null, 2) + '\n'); \
    let js = fs.readFileSync('pkg/datagraph.js', 'utf8'); \
    js = js.replace( \
      'let cachedTextDecoder = new TextDecoder(\'utf-8\', { ignoreBOM: true, fatal: true });\ncachedTextDecoder.decode();', \
      'let cachedTextDecoder = null;' \
    ).replace( \
      'function decodeText(ptr, len) {', \
      'function decodeText(ptr, len) {\n    if (typeof TextDecoder === \'undefined\') { return Array.from(getUint8ArrayMemory0().subarray(ptr, ptr + len)).map(b => String.fromCharCode(b)).join(\'\'); }\n    if (cachedTextDecoder === null) { cachedTextDecoder = new TextDecoder(\'utf-8\', { ignoreBOM: true, fatal: true }); cachedTextDecoder.decode(); }' \
    ); \
    fs.writeFileSync('pkg/datagraph.js', js); \
  "

test:
  cargo test
