# Build wasm for the web
```bash
wasm-pack build --target web
```
Which will create a pkg directory containing the Wasm module (.wasm), JavaScript bindings (.js), and TypeScript definitions (.d.ts).

Which you may import like
```javascript
import init, { greet } from "./pkg/stox_wasm.js";

async function run() {
    await init(); // Initialize the Wasm module
    console.log(greet("World")); // Call your Rust function
}

run();
```