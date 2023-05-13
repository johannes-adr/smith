# Smith-codegen

smith-codegen is a command-line tool that generates code from a schema file for the smith-core crate. It currently supports Rust and TypeScript as export types.

## Installation

To use smith-codegen, you need to have Rust installed on your system.
Clone the repository and build it from source.


## Usage
Once installed, you can use smith-codegen to generate code from a schema file. The basic usage is:

```sh
$ cargo run -- -f <schema_file> -o <output_file> -t <export_type>
```
where `schema_file` is the path to the schema file, `output_file` is the path to the output file, and `export_type` is either rust or typescript.


## Example
For example, to generate Rust code from a schema file schema.bdr and save it to out.rs, you would run:

```sh
$ cargo run -- -f schema.bdr -o out.rs -t rust
```

## smith-codegen as rust build dependecy

It's possible to use smith-codegen as build dependency for a build-script:
```rs
let inp = std::fs::read_to_string(".types.schema.bdr").unwrap();
let out = smith_codegen::generate_lang(&inp, smith_codegen::Language::Rust).unwrap();
std::fs::write("./src/generated_types.rs", out).unwrap();
```