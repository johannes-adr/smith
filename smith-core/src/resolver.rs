use std::sync::atomic::Ordering;

use crate::{
    generics_engine::{SmithCustomTyp, SmithEnum, SmithProgram, SmithStruct},
    parser::{RootDeclaration, SmithType},
};
use crate::generics_engine::SmithCustomTypVariant;

pub type ResolvedSmithProgram = Vec<SmithCustomTyp<usize>>;
pub fn resolve(program: SmithProgram<String>) -> Result<SmithProgram<usize>, String> {
    let mut resolved_types = Vec::with_capacity(program.expanded.len());
    for custom_type in &program.expanded {
        let resolved = resolve_custom_type(custom_type, &program.expanded)?;
        resolved_types.push(resolved);
    }

    //TODO: The following code only works when the customtype id depends on its position in the resolved vec
    resolved_types
        .iter()
        .enumerate()
        .for_each(|(pos, e)| e.id.store(pos, Ordering::Relaxed));

    Ok(SmithProgram {
        expanded: resolved_types,
        generics: program.generics,
    })
}

fn resolve_custom_type(
    custom_type: &SmithCustomTyp<String>,
    custom_types: &[SmithCustomTyp<String>],
) -> Result<SmithCustomTyp<usize>, String> {
    let resolved = match &custom_type.variant {
        SmithCustomTypVariant::Struct(s) => {
            let resolved_field: Result<Vec<_>, String> = s
                .fields
                .iter()
                .map(|f| Ok((f.0.clone(), resolve_typ(&f.1, custom_types)?)))
                .collect();

            SmithCustomTypVariant::Struct(SmithStruct {
                name: s.name.clone(),
                fields: resolved_field?,
            })
        }
        SmithCustomTypVariant::Enum(e) => {
            let resolved_variants: Result<Vec<_>, String> = e
                .variants
                .iter()
                .map(|f| {
                    Ok((
                        f.0.clone(),
                        if let Some(typ) = &f.1 {
                            Some(resolve_typ(typ, custom_types)?)
                        } else {
                            None
                        },
                    ))
                })
                .collect();

            SmithCustomTypVariant::Enum(SmithEnum {
                name: e.name.clone(),
                variants: resolved_variants?,
            })
        }
    };
    Ok(SmithCustomTyp {
        variant: resolved,
        id: custom_type.id.clone(),
    })
}

pub fn resolve_typ(
    typ: &SmithType<String>,
    custom_types: &[SmithCustomTyp<String>],
) -> Result<SmithType<usize>, String> {
    let typ: SmithType<usize> = match typ {
        SmithType::I8 => SmithType::I8,
        SmithType::I16 => SmithType::I16,
        SmithType::I32 => SmithType::I32,

        SmithType::U8 => SmithType::U8,
        SmithType::U16 => SmithType::U16,
        SmithType::U32 => SmithType::U32,
        SmithType::U64 => SmithType::U64,

        SmithType::F32 => SmithType::F32,
        SmithType::F64 => SmithType::F64,
        SmithType::UInt => SmithType::UInt,
        SmithType::Bool => SmithType::Bool,
        SmithType::String => SmithType::String,
        SmithType::Array(typ) => SmithType::Array(Box::new(resolve_typ(&typ, custom_types)?)),
        SmithType::CustomType(_name, gen) => {
            let mut name = String::with_capacity(50);
            typ.write_self(&mut name);
            let found = custom_types
                .iter()
                .position(|el| el.get_name() == name)
                .ok_or(format!("Type '{name}' could not be resolved"))?;
            SmithType::CustomType(
                found,
                gen.iter()
                    .map(|t| resolve_typ(t, custom_types))
                    .collect::<Result<Vec<_>, String>>()?,
            )
        }
    };

    Ok(typ)
}
