use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

use crate::{
    parser::{AST, RootDeclaration},
    SmithType,
};
use crate::parser::ASTRootType;

#[derive(Debug)]
pub struct SmithStruct<T> {
    pub name: String,
    pub fields: Vec<(String, SmithType<T>)>,
}
#[derive(Debug)]
pub struct SmithEnum<T> {
    pub name: String,
    pub variants: Vec<(String, Option<SmithType<T>>)>,
}
#[derive(Debug)]
pub enum SmithCustomTypVariant<T> {
    Struct(SmithStruct<T>),
    Enum(SmithEnum<T>),
}

#[derive(Debug)]
pub struct SmithCustomTyp<T> {
    pub variant: SmithCustomTypVariant<T>,
    pub id: Arc<AtomicUsize>,
}

impl<T> SmithCustomTyp<T> {
    pub fn get_name(&self) -> &str {
        match &self.variant {
            SmithCustomTypVariant::Struct(s) => &s.name,
            SmithCustomTypVariant::Enum(e) => &e.name,
        }
    }

    pub fn as_struct(&self) -> Option<&SmithStruct<T>> {
        match &self.variant {
            SmithCustomTypVariant::Struct(s) => Some(s),
            SmithCustomTypVariant::Enum(_e) => None,
        }
    }

    pub fn as_enum(&self) -> Option<&SmithEnum<T>> {
        match &self.variant {
            SmithCustomTypVariant::Struct(_s) => None,
            SmithCustomTypVariant::Enum(e) => Some(e),
        }
    }
}

impl From<&(Arc<AtomicUsize>, Rc<dyn RootDeclaration>)> for SmithCustomTyp<String> {
    fn from((id, dec): &(Arc<AtomicUsize>, Rc<dyn RootDeclaration>)) -> Self {
        let variant = match dec.typ() {
            ASTRootType::Struct(s) => SmithCustomTypVariant::Struct(SmithStruct {
                name: s.name.clone(),
                fields: s.fields.clone(),
            }),
            ASTRootType::Enum(e) => SmithCustomTypVariant::Enum(SmithEnum {
                name: e.name.clone(),
                variants: e.variants.clone(),
            }),
        };
        Self {
            variant,
            id: id.clone(),
        }
    }
}

//Contains the expanded types and the implementations for each version of a generic type
#[derive(Debug)]
pub struct SmithProgram<T> {
    pub expanded: Vec<SmithCustomTyp<T>>,
    pub generics: ImpsMap,
}
impl<T> SmithProgram<T> {
    pub fn get_by_name(&self, name: &str) -> Option<&SmithCustomTyp<T>> {
        self.expanded.iter().find(|f| f.get_name() == name)
    }

    pub fn type_by_name(&self, name: &str) -> Option<SmithType<usize>> {
        let s = self.expanded.iter().position(|f| f.get_name() == name)?;
        Some(SmithType::CustomType(s, vec![]))
    }
}

type ImpsMap = HashMap<
    //Name of type: "result"
    String,
    (
        //Contains the implementations for each version of a generic type
        HashMap<
            //Generic dependency: <string,u8>
            Vec<SmithType<String>>,
            //Corresponding implementation
            (Arc<AtomicUsize>, Rc<dyn RootDeclaration>),
        >,
        //If generic, contains its rootdeclaration
        Option<Rc<dyn RootDeclaration>>,
    ),
>;
//Expands the AST to a SmithProgram by resolving generics
pub fn expand(ast: AST) -> Result<SmithProgram<String>, &'static str> {
    //Enums or Struct without generic Dependencies
    let decs: Vec<_> = ast.0.into_iter().map(|s| (Default::default(), s)).collect();

    let mut generics_engine = GenericEngine {
        imps_map: HashMap::new(),
        declarations: &decs,
    };

    decs.iter()
        .filter(|e| e.1.generics().len() == 0)
        .flat_map(|e| e.1.get_field_implementors())
        .for_each(|e| {
            generics_engine.do_typ(e);
        });
    let mut imps = generics_engine.imps_map;
    for i in decs.into_iter().filter(|e| e.1.generics().len() == 0) {
        imps.insert(i.1.name().to_owned(), (HashMap::from([(vec![], i)]), None));
    }

    let expanded = imps
        .iter()
        .flat_map(|(_, v)| v.0.iter().map(|(_, t)| t.into()))
        .collect::<Vec<SmithCustomTyp<String>>>();

    Ok(SmithProgram {
        expanded,
        generics: imps,
    })
}

struct GenericEngine<'a> {
    imps_map: ImpsMap,
    declarations: &'a Vec<(Arc<AtomicUsize>, Rc<dyn RootDeclaration>)>,
}
impl<'a> GenericEngine<'a> {
    fn do_typ(&mut self, typ: &SmithType<String>) {
        match typ {
            SmithType::CustomType(name, gen) => {
                //Return if type has no generic dependencys

                if gen.len() == 0 {
                    return;
                }
                //Do recursivly if generic implementation consists of other generic implementations
                //For example: Result<Option<u8>,...>
                //                       ^
                //Needs to create type for Optional<u8> first
                gen.iter().for_each(|t| self.do_typ(t));

                let imps = self.imps_map.entry(name.clone()).or_default();

                //if imps doesnt contains the key already, generate the the implementation of the generic type with
                //its dependencies
                if imps.0.contains_key(gen) {
                    return;
                }

                let blueprint = self
                    .declarations
                    .iter()
                    .find(|e| e.1.name() == name)
                    .expect(&format!("Unable to find generic type '{name}'"));

                let gen = gen.clone();
                let expanded = self.expand_generic(&blueprint.1, &gen);
                //Require reference, since expand_generic may modify self - no multiple mutable borrows
                let imps = self.imps_map.get_mut(name.as_str()).expect(
                    "Fatal Logic Error: ImpsMap should contain value for Key at this point in code",
                );

                imps.0.insert(gen, (Default::default(), expanded.into()));
                if let None = imps.1 {
                    _ = imps.1.insert(blueprint.1.clone());
                }
            }
            SmithType::Array(gen) => self.do_typ(gen),
            _ => {}
        }
    }
    //Expands a rootdeclaration based on the provided types for the generic dependencys
    fn expand_generic(
        &mut self,
        blueprint: &Rc<dyn RootDeclaration>,
        dep: &[SmithType<String>],
    ) -> Box<dyn RootDeclaration> {
        if dep.len() != blueprint.generics().len() {
            panic!("Amount generic arguments for type '{}' not matching (expecting: {:?} - provided: {:?})"
                   ,blueprint.name(),blueprint.generics(), dep
            )
        }

        let mut cpy = blueprint.deep_clone();
        let typ = SmithType::CustomType(cpy.name().to_string(), dep.into());
        let mut s = String::new();
        typ.write_self(&mut s);
        cpy.set_name(s);
        for field in cpy.get_field_implementors_mut() {
            self.expand_field(field, blueprint, dep);
            self.do_typ(field);
        }
        cpy
    }

    //Expands a field a generic type with its dependencies
    //(if it has any) and replaces the generic type with the corresponding type
    fn expand_field(
        &mut self,
        field: &mut SmithType<String>,
        blueprint: &Rc<dyn RootDeclaration>,
        dep: &[SmithType<String>],
    ) {
        match field {
            SmithType::CustomType(name, gen) => {
                if let Some(pos) = blueprint.generics().iter().position(|s| s == name) {
                    if gen.len() > 0 {
                        panic!("Generic Type '{name}' cannot have generic arguments!",);
                    }
                    *field = dep[pos].clone();
                } else {
                    for t in gen {
                        self.expand_field(t, blueprint, dep);
                    }
                }
            }
            SmithType::Array(t) => {
                self.expand_field(t, blueprint, dep);
                self.do_typ(t);
            }
            _ => {}
        }
    }
}
