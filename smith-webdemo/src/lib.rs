
use std::{panic, fmt::Write};

use smith_codegen::Language;
use smith_core::SmithType;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn wasminit(){
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[wasm_bindgen]
pub fn generate_ts(schema: &str) -> Result<String, String>{
    smith_codegen::generate_lang(schema, Language::TypeScript)
}

#[wasm_bindgen]
pub fn generate_rust(schema: &str) -> String{
    let s = smith_codegen::generate_rust(schema);
    let file = syn::parse_file(&s.to_string()).unwrap();
    return prettyplease::unparse(&file)
}

#[wasm_bindgen]
pub fn show_imps_map(schema: &str) -> String{
    let gen = smith_core::compile(schema).generics;
    fn format_vec(v: Vec<SmithType<String>>) -> String{
        let mut s = String::new();
        for v in v{
            let mut sbuff = String::new();
            v.write_self(&mut sbuff);
            s.push_str(&format!("{},", sbuff))
        }
        s.pop();
        return s
    }
    let mut s = String::new();
    _=s.write_str("{\n");
    for (imp, (gen,_)) in gen{
        _=s.write_str(&format!("  \"{}\": \n", imp));
        for (keytype, (_,version)) in gen{
            let mut s2 = String::new();
            match version.typ(){
                smith_core::parser::ASTRootType::Struct(s) => {
                    _=s2.write_str(&format!("{:?}",s.fields));
                },
                smith_core::parser::ASTRootType::Enum(s) => {
                    let mut svar = String::new();
                    for var in &s.variants{
                        _=svar.write_str(&format!("{}{}, ",var.0, if let Some(s) = &var.1{format!("({})",format_vec(vec![s.clone()]))}else{String::new()}));
                    }
                    svar.pop();
                    svar.pop();
                    _=s2.write_str(&format!("[{svar}]"));
                },
            }

            _=s.write_str(&format!("      [{}] => {}\n", format_vec(keytype),s2));
        }
    }
    _=s.write_str("}");
    return s
}