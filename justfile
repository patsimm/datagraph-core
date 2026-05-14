default: build

build:
  wasm-pack build --target web
  node -e " \
    const fs = require('fs'); \
    const p = JSON.parse(fs.readFileSync('pkg/package.json', 'utf8')); \
    p.name = '@patsimm/datagraph-core'; \
    fs.writeFileSync('pkg/package.json', JSON.stringify(p, null, 2) + '\n'); \
    "

test:
  cargo test
