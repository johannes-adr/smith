extern crate pest;

use std::{
    fmt::{Debug, Write},
    rc::Rc,
};

use pest::{
    iterators::{Pair, Pairs},
    Parser,
};


#[derive(Parser)]
#[grammar = "grammar.pest"]
struct DeclParser;


pub struct AST(pub Vec<Rc<dyn RootDeclaration>>);

impl AST {
    fn new() -> Self {
        Self(vec![])
    }
    pub fn to_string(&self) -> String {
        let s: Vec<_> = self.0.iter().map(|i| i.typ()).collect();
        format!("{:?}", s)
    }
}

pub fn parse(src: &str) -> AST {
    let rules = DeclParser::parse(Rule::document, src)
        .unwrap()
        .next()
        .unwrap()
        .into_inner();

    let mut root_declarations = AST::new();
    for rule in rules {
        match rule.as_rule() {
            Rule::EOI | Rule::comment | Rule::comment_multiline => continue,

            Rule::Struct => root_declarations.0.push(Rc::new(parse_struct(rule))),

            Rule::Enum => root_declarations.0.push(Rc::new(parse_enum(rule))),
            _ => {
                panic!("Unexpected rule {:?}", rule.as_rule())
            }
        }
    }
    root_declarations
}

fn parse_struct_generics(rule: Pair<Rule>) -> Vec<String> {
    let mut list = Vec::new();
    for dec in rule.into_inner() {
        let mut dec = dec.into_inner();
        let name = dec.next().unwrap().as_str().to_string();
        list.push(name)
    }
    list
}

fn parse_struct(rule: Pair<Rule>) -> ParsedStruct {
    let mut rule = rule.into_inner();
    let name = rule.next().unwrap().as_str().to_owned();
    let mut generics = Vec::new();
    if let Some(s) = rule.peek() {
        if s.as_rule() == Rule::generics {
            generics.append(&mut parse_struct_generics(rule.next().unwrap()));
        }
    }
    let mut fields = Vec::new();
    for field in rule {
        let mut field = field.into_inner();
        let name = field.next().unwrap().as_str().to_owned();
        let typ = parse_typ(field.next().unwrap());
        fields.push((name, typ));
    }
    return ParsedStruct {
        name,
        fields,
        generics,
    };
}

fn parse_enum(rule: Pair<Rule>) -> ParsedEnum {
    let mut rule = rule.into_inner();
    let name = rule.next().unwrap().as_str().to_owned();
    let mut generics = Vec::new();
    if let Some(s) = rule.peek() {
        if s.as_rule() == Rule::generics {
            generics.append(&mut parse_struct_generics(rule.next().unwrap()));
        }
    }
    let mut variants = Vec::new();
    for field in rule {
        let mut field = field.into_inner();
        let name = field.next().unwrap().as_str().to_owned();
        let mut typ = None;
        if let Some(s) = field.next() {
            _ = typ.insert(parse_typ(s));
        }
        variants.push((name, typ));
    }
    ParsedEnum {
        name,
        generics,
        variants,
    }
}

pub trait RootDeclaration: Debug {
    fn name(&self) -> &str;
    fn set_name(&mut self, s: String);
    fn generics(&self) -> &Vec<String>;
    fn typ<'a>(&'a self) -> ASTRootType<'a>;
    //Function to generally get all possible generic implementors
    fn get_field_implementors(&self) -> Vec<&SmithType<String>>;
    fn get_field_implementors_mut(&mut self) -> Vec<&mut SmithType<String>>;
    fn deep_clone(&self) -> Box<dyn RootDeclaration>;
}

#[derive(Debug)]
pub enum ASTRootType<'a> {
    Struct(&'a ParsedStruct),
    Enum(&'a ParsedEnum),
}

#[derive(Debug, Clone)]
pub struct ParsedStruct {
    pub name: String,
    pub generics: Vec<String>,
    pub fields: Vec<(String, SmithType<String>)>,
}

impl RootDeclaration for ParsedStruct {
    fn name(&self) -> &str {
        &self.name
    }

    fn generics(&self) -> &Vec<String> {
        &self.generics
    }

    fn typ<'a>(&'a self) -> ASTRootType<'a> {
        ASTRootType::Struct(self)
    }

    fn get_field_implementors(&self) -> Vec<&SmithType<String>> {
        self.fields.iter().map(|f| &f.1).collect()
    }

    fn get_field_implementors_mut(&mut self) -> Vec<&mut SmithType<String>> {
        self.fields.iter_mut().map(|f| &mut f.1).collect()
    }
    fn deep_clone(&self) -> Box<dyn RootDeclaration> {
        Box::new(self.clone())
    }

    fn set_name(&mut self, s: String) {
        self.name = s
    }
}

#[derive(Debug, Clone)]
pub struct ParsedEnum {
    pub name: String,
    pub generics: Vec<String>,
    pub variants: Vec<(String, Option<SmithType<String>>)>,
}

impl RootDeclaration for ParsedEnum {
    fn name(&self) -> &str {
        &self.name
    }

    fn set_name(&mut self, s: String) {
        self.name = s
    }

    fn generics(&self) -> &Vec<String> {
        &self.generics
    }

    fn typ(&self) -> ASTRootType {
        ASTRootType::Enum(self)
    }

    fn get_field_implementors(&self) -> Vec<&SmithType<String>> {
        Box::new(self.variants.iter().filter_map(
            |f| {
                if let Some(s) = &f.1 {
                    Some(s)
                } else {
                    None
                }
            },
        ))
        .collect::<Vec<_>>()
    }

    fn get_field_implementors_mut(&mut self) -> Vec<&mut SmithType<String>> {
        Box::new(self.variants.iter_mut().filter_map(|f| {
            if let Some(s) = &mut f.1 {
                Some(s)
            } else {
                None
            }
        }))
        .collect::<Vec<_>>()
    }

    fn deep_clone(&self) -> Box<dyn RootDeclaration> {
        Box::new(self.clone())
    }
}

pub fn parse_typ(rule: Pair<Rule>) -> SmithType<String> {
    let mut rule = rule.into_inner();
    let typname = rule.next().unwrap().as_str();

    let parse_generic_type = |rule: &mut Pairs<Rule>| rule.map(parse_typ).collect::<Vec<_>>();

    let typ = match typname {
        "i8" => SmithType::I8,
        "i16" => SmithType::I16,
        "i32" => SmithType::I32,
        "f32" => SmithType::F32,
        "f64" => SmithType::F64,
        "u8" => SmithType::U8,
        "u16" => SmithType::U16,
        "u32" => SmithType::U32,
        "u64" => SmithType::U64,
        "udInt" => SmithType::UInt,
        "bool" => SmithType::Bool,
        "string" => SmithType::String,
        "Array" => SmithType::Array(Box::new(parse_generic_type(&mut rule).pop().unwrap())),
        _ => SmithType::CustomType(typname.to_owned(), parse_generic_type(&mut rule)),
    };
    typ
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum SmithType<T> {
    I8,
    I16,
    I32,
    F32,
    F64,
    U8,
    U16,
    U32,
    U64,
    UInt,
    Bool,
    String,

    // (index_in_program, generics)
    CustomType(T, Vec<SmithType<T>>),
    Array(Box<SmithType<T>>),
}
impl SmithType<String> {
    pub fn write_self(&self, buff: &mut String) {
        _ = match self {
            SmithType::I8 => buff.write_str("i8"),
            SmithType::I16 => buff.write_str("i16"),
            SmithType::I32 => buff.write_str("i32"),
            SmithType::F32 => buff.write_str("f32"),
            SmithType::F64 => buff.write_str("f64"),
            SmithType::U8 => buff.write_str("u8"),
            SmithType::U16 => buff.write_str("u16"),
            SmithType::U32 => buff.write_str("u32"),
            SmithType::U64 => buff.write_str("u64"),
            SmithType::UInt => buff.write_str("udInt"),
            SmithType::Bool => buff.write_str("bool"),
            SmithType::String => buff.write_str("string"),
            SmithType::CustomType(name, gen) => {
                let r = buff.write_str(name);
                if gen.len() > 0 {
                    _ = buff.write_str("<");

                    gen.iter().for_each(|s| {
                        s.write_self(buff);
                        _ = buff.write_char(',');
                    });
                    buff.pop();
                    _ = buff.write_str(">");
                }

                r
            }
            SmithType::Array(t) => {
                _ = buff.write_str("Array<");
                t.write_self(buff);
                _ = buff.write_str(">");
                Ok(())
            }
        };
    }
}
