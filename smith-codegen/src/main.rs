use std::{env, fs};
use std::path::PathBuf;
use smith_codegen::{generate_lang, generate_rust, Language};
use structopt::StructOpt;
mod rustgen;
mod typescriptgen;
mod out;
#[derive(StructOpt, Debug)]
#[structopt(name = "binader-codegen")]
pub struct Opt {
    #[structopt(short = "f", long, parse(from_os_str))]
    pub schema_file: PathBuf,

    #[structopt(short = "o", long, parse(from_os_str))]
    pub output_file: PathBuf,

    #[structopt(short = "t", long)]
    pub export_type: String,
}

fn test_rust(){
    let src = fs::read_to_string("./test.bdr").unwrap();
    let out = generate_lang(&src,Language::Rust);
    fs::write("./src/out.rs",out.unwrap()).unwrap();
}

fn main() {
    let opt = Opt::from_args();
    let lang = *Language::languages()
        .iter()
        .find(|l| l.get_name() == opt.export_type)
        .ok_or_else(|| {
            println!(
                "Error: Given export type '{}' not supported - use one of the following: {:?}",
                opt.export_type,
                Language::languages()
                    .iter()
                    .map(|l| l.get_name())
                    .collect::<Box<[_]>>()
            );
            std::process::exit(1);
        })
        .unwrap();

    let schema_code = std::fs::read_to_string(opt.schema_file).unwrap_or_else(|err| {
        println!("Error reading schema file: {err}");
        std::process::exit(1);
    });

    let res = generate_lang(&schema_code, lang).unwrap_or_else(|err| {
        println!("Error while generating export code: {err}");
        std::process::exit(1);
    });

    std::fs::write(opt.output_file, res).unwrap_or_else(|err| {
        println!("Error while saving to file: {err}");
        std::process::exit(1);
    });

    println!("Done :)");
}
