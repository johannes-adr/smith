use std::collections::{HashMap, HashSet};
use smith_core::{SmithProgram, SmithType, parser::{ASTRootType, ParsedStruct, ParsedEnum}};


pub fn generate(b: &SmithProgram<usize>) -> Result<String, String> {
   let res = b.generics.iter().map(|(_,imps)|{

      let (imps, orig) = imps;

      let (q_generic,rootdec) = if let Some(s) = orig {
          (format!("<{}>",s.generics().join(",")),s)
      } else {
          let val = imps.get(&vec![]).unwrap();
          (String::new(),&val.1)
      };
      match rootdec.typ(){
          ASTRootType::Struct(v) => {generate_struct(q_generic,v)}
          ASTRootType::Enum(v) => {generate_enum(q_generic, v)}
      }
   });
   let s = res.collect::<Vec<String>>().join("\n\n");

Ok(format!(r#"
{s}
"#))
}


fn generate_struct(q_generic: String, val: &ParsedStruct) -> String{
   let q_field = val.fields
      .iter()
      .map(|f|{
       format!("  {}: {}",f.0, as_js_type(&f.1))
   }).collect::<Vec<_>>().join("\n");
   format!("export interface {}{q_generic}{{\n{q_field}\n}}",val.name)
}

fn generate_enum(q_generic: String, val: &ParsedEnum) -> String{
   let name = &val.name;

   let tag_names = val.variants
         .iter()
         .map(|v|format!("'{}'",v.0))
         .collect::<Vec<_>>().join(" | ");
   //Count how many variants have a value
   let vals_count = val.variants.iter()
         .filter(|v|
            v.1.is_some()).map(|v|&v.1).collect::<Vec<_>>();

   let optional = if val.variants.len() != vals_count.len() {"?"}else{""};
   //If no variant have a value, there is no need for a val field
   let mut vals_unique = HashSet::new();
   vals_count.iter().for_each(|v|{
      if let Some(s) = v{
         vals_unique.insert(as_js_type(s));
      }
   });
   let vals_joined = vals_unique.into_iter().collect::<Vec<_>>().join(" | ");
   let field_val = if vals_count.len() > 0{
      format!("private val{optional}: {vals_joined}")
   }else{"".to_owned()};

   let variant_functions = val.variants.iter().map(|(varname,typ)|{
      let (joined,param_name,typ_js) = if let Some(typ) = typ{
         let typ = as_js_type(typ);
         (format!("v: {typ}",),"v".to_owned(),typ)
      }else{
         Default::default()
      };

      let generic = if val.generics.contains(&typ_js){
         format!("<{typ_js}>")
      }else{
         String::new()
      };

      let cast_function = if let Some(_) = typ{
         format!(r#"return this.val as {typ_js}"#)
      }else{
         String::new()
      };

      format!(r#"
   static {varname}{generic}({joined}){{return new {name}("{varname}",{param_name})}}
   as_{varname}(){{
      if (this.tag != '{varname}'){{
         throw new Error("Enum {name}: trying to cast variant '" + this.tag + "' into '{varname}'")
      }}
      {cast_function}    
   }}"#)
      }
   ).collect::<Vec<_>>().join("\n");

   let get_tag_func = if val.variants.len() > 0{
      format!("getTag(){{ return this.tag }}")
   }else{
      String::new()
   };
format!(r#"export class {name}{q_generic}{{
   private tag: {tag_names}
   {field_val}
   private constructor(tag: {tag_names}{}){{
     this.tag = tag;
     {}
   }}
   {variant_functions}
   {get_tag_func}
}}"#, if vals_count.len() > 0 {format!(", val{optional}: {vals_joined}")}else{String::new()},
    if vals_count.len() > 0 {"this.val = val;"}else{""})
}

fn as_js_type(typ: &SmithType<String>) -> String {
   match typ {
      SmithType::I8 => "number".to_string(),
      SmithType::I16 => "number".to_string(),
      SmithType::I32 => "number".to_string(),
      SmithType::F32 => "number".to_string(),
      SmithType::F64 => "number".to_string(),
      SmithType::U8 => "number".to_string(),
      SmithType::U16 => "number".to_string(),
      SmithType::U32 => "number".to_string(),
      SmithType::U64 => "number".to_string(),
      SmithType::UInt => "number".to_string(),
      SmithType::Bool => "boolean".to_string(),
      SmithType::String => "string".to_string(),
      SmithType::CustomType(name,gen) => {
         let mut n = name.clone();
         if gen.len() > 0{
            let gens = gen.iter().map(as_js_type).collect::<Vec<_>>();
            n.push('<');
            n.push_str(&gens.join(","));
            n.push('>');
         }
         n
      },
      SmithType::Array(typ) => format!("{}[]", as_js_type(&typ)),
   }
}
