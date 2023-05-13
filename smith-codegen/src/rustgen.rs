use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use smith_core::SmithType;
use smith_core::SmithProgram;


use quote::{format_ident, quote};
use quote::__private::TokenStream;
use rust_format::{Formatter, RustFmt};
use smith_core::generics_engine::SmithCustomTyp;
use smith_core::parser::{ASTRootType, ParsedEnum, ParsedStruct, RootDeclaration};

pub fn generate(b: &SmithProgram<usize>) -> Result<String, String> {
    RustFmt::default()
        .format_tokens(generate_tokens(b))
        .map_err(|e| e.to_string())
}

pub fn generate_tokens(b: &SmithProgram<usize>) -> TokenStream {
    //b.expanded.iter().for_each(|e|{elems.insert(e.get_souce().name(),e.get_souce().clone());});
    let decs = b.generics.iter()
        .map(|(_, imps)| {

            let (imps, orig) = imps;

            let (q_generic,rootdec) = if let Some(s) = orig {
                let generics: TokenStream = s.generics().iter().map(|t|{
                    format!("{t}")
                }).collect::<Vec<_>>().join(",").parse().unwrap();
                (quote!(<#generics>),s)
            } else {
                let val = imps.get(&vec![]).unwrap();
                (quote!(),&val.1)
            };
            match rootdec.typ(){
                ASTRootType::Struct(v) => {generate_struct(q_generic,v)}
                ASTRootType::Enum(v) => {generate_enum(q_generic,v)}
            }

        })
        .collect::<Vec<TokenStream>>();

    quote!(
        use serde::{Deserialize, Serialize};
        use std::fmt::Debug;
        /*trait _Bound: Serialize + Deserialize + PartialEq + Debug + Clone{}*/
        
        #(#decs)*
    )
}

fn generate_struct(q_generic: TokenStream, val: &ParsedStruct) -> TokenStream{
    let q_name = format_ident!("{}",val.name);
    let q_field = val.fields.iter().map(|f|{
        let ident = format_ident!("{}",f.0);
        let typ = as_rust_type2(&f.1);
        quote!(#ident: #typ)
    });
    quote!(
        #[derive(Serialize,Deserialize,PartialEq,Debug,Clone)]
        pub struct #q_name #q_generic{
            #(pub #q_field ,)*
        }
    )
}


fn generate_enum(q_generic: TokenStream, val: &ParsedEnum) -> TokenStream{
    let q_name = format_ident!("{}",val.name);
    let q_field = val.variants.iter().map(|f|{
        let ident = format_ident!("{}",f.0);
        let new_type_val = if let Some(s) = &f.1{
            let typ = as_rust_type2(s);
            quote!((#typ))
        }else{
            quote!()
        };
        quote!(#ident #new_type_val)
    });
    quote!(
        #[derive(Serialize,Deserialize,PartialEq,Debug,Clone)]
        pub enum #q_name #q_generic{
            #(#q_field ,)*
        }
    )
}



fn as_rust_type2(typ: &SmithType<String>) -> TokenStream {
    if let SmithType::Array(typ) = typ {
        let typ = as_rust_type2(typ);
        return quote!(
            Box<[#typ]>
        );
    }

    if let SmithType::CustomType(name,gen) = typ{
        let name = format_ident!("{name}");
        let gen = if gen.len() == 0{
            quote!()
        }else{
            let genjoined = gen.iter().map(as_rust_type2);
            quote!(<#(#genjoined,)*>)
        };
        return quote!(#name #gen)
    }

    let t = format_ident!(
        "{}",
        match typ {
            SmithType::I8 => "i8",
            SmithType::I16 => "i16",
            SmithType::I32 => "i32",
            SmithType::F32 => "f32",
            SmithType::F64 => "f64",
            SmithType::U8 => "u8",
            SmithType::U16 => "u16",
            SmithType::U32 => "u32",
            SmithType::U64 => "u64",
            SmithType::UInt => "u64",
            SmithType::Bool => "bool",
            SmithType::String => "String",
            _=>{panic!()}
        }
    );

    quote!(#t)
}
