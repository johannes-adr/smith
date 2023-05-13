# Smith short instruction

## Some commands only work on mac

run `cargo run --release` to execute smith-test.  
It will run a benchmark between smith, protobuf and json in terms of speed and resulting filesize.

------
For tests, run 
`cargo test --workspace --release` (Relase is imporntant, some tests will take some time)

------
 
```sh
cd smith-codegen
sh test.sh
```

-----
```sh
cd smith-js
wasm-pack build --target web
```
use a fileserver to serve files in smith-js/example and smith-js/pkg 

-----
Webdemo uses webassembly to transpile a scheme file into rust and typescript
and gives inside information into the internal structure of the parsed scheme.

