use quote::__private::TokenStream;
use std::env;
use std::path::PathBuf;
use smith_core::{compile, SmithProgram, Smith};
use structopt::StructOpt;
mod rustgen;
mod typescriptgen;

pub fn generate_lang(schema: &str, export_typ: Language) -> Result<String, String> {
    let prog = compile(schema);
    export_typ.generate(&prog)
}

pub fn generate_rust(schema: &str) -> TokenStream {
    rustgen::generate_tokens(&compile(schema))
}

#[derive(Debug, Clone, Copy)]
pub enum Language {
    TypeScript,
    Rust,
}

impl Language {
    pub fn languages() -> &'static [Language] {
        &[Language::TypeScript, Language::Rust]
    }

    pub fn get_name(&self) -> &'static str {
        match self {
            Language::TypeScript => "typescript",
            Language::Rust => "rust",
        }
    }

    fn generate(&self, b: &SmithProgram<usize>) -> Result<String, String> {
        match self {
            Language::TypeScript => typescriptgen::generate(b),
            Language::Rust => rustgen::generate(b),
        }
    }
}
