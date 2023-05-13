# smith-js

This package provides JavaScript bindings for the Rust crate [smith-core](https://github.com/johannes-adr/smith-core), allowing JavaScript developers to serialize data into a compact binary represesentation. For more information, see the main crate

## Installation

Clone the repository and build it with the following comamnd:
`wasm-pack build --target web`

note: this crate requires [wasm-pack](https://rustwasm.github.io/wasm-pack/) to build


## Usage

To use this package, first import the WASM initialisation function and the SmithJS class
```js
import init, { SmithJS } from "../pkg/smith_js.js"
```

Then, await the wasm initialisation and create an instance of Smith
```js
async function main() {
    await init();
    let smith = new SmithJS("..." /* src of schema file */)
    /* code */
}
main();
```

Now you can use the `serialize` and `deserialize` method to convert you data
```js
let bytes = smith.serialize(original,"SchemaType")
let deserialized = smith.deserialize(bytes,"SchemaType");
assert(original == deserialized)
```

# Example
See "example" folder in directory