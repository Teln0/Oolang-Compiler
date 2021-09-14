use crate::tir::{TIRRoot, TIRTypeKind, TIRMemberKind, TIRModifier, TIRTypeInfo, TIRType, TIRTypeInfoKind, PrimitiveType, TIRExpr, TIRExprKind, TIROperator};
use crate::type_ref_pool::{TypeRefPool, TypeRefKind};
use crate::field_ref_pool::{FieldRefPool, FieldRef};
use crate::method_ref_pool::{MethodRefPool, MethodRef};
use std::collections::HashMap;
use oolang_bytecode::typefile_structure::{TypeFileFormat, TypeKindFormat, FieldFormat, TypeInfoFormat, TypeInfoKindFormat};
use oolang_bytecode::BytecodeFormat;
use oolang_bytecode::poolfile_structure::FieldRefFormat;

pub mod context;
pub mod member_codegen;

#[derive(Debug)]
pub struct BytecodeFile<'a> {
    pub file_path: Vec<&'a str>,
    pub bytecode: Vec<u8>
}

pub struct Codegen<'a> {
    tir_root: TIRRoot<'a>,

    type_ref_pool: TypeRefPool<'a>,
    field_ref_pool: FieldRefPool<'a>,
    method_ref_pool: MethodRefPool<'a>
}

impl<'a> Codegen<'a> {
    pub fn new(tir_root: TIRRoot<'a>, type_ref_pool: TypeRefPool<'a>) -> Self {
        Self {
            tir_root,
            type_ref_pool,
            field_ref_pool: FieldRefPool::new(),
            method_ref_pool: MethodRefPool::new()
        }
    }

    fn create_field_and_method_refs(&mut self) -> Result<(), CodegenError<'a>> {
        for type_decl in &self.tir_root.types {
            match &type_decl.kind {
                TIRTypeKind::Class { members, .. } => {
                    let mut field_index = 0;
                    let mut method_index = 0;
                    let mut index_in_all_members = 0;
                    for member in members {
                        match &member.kind {
                            TIRMemberKind::Field { name_and_type, .. } => {
                                let field_ref_index = self.field_ref_pool.field_refs.len();

                                if let Some(_) = self.field_ref_pool.type_ref_index_and_name_to_field_ref_index.insert(
                                    (type_decl.type_ref_index, name_and_type.name),
                                    field_ref_index
                                ) {
                                    return Err(CodegenError::FieldNameTaken(name_and_type.name));
                                }
                                if self.method_ref_pool.name_and_type_ref_index_to_method_ref_indexes.contains_key(
                                    &(type_decl.type_ref_index, name_and_type.name)
                                ) {
                                    return Err(CodegenError::FieldNameTaken(name_and_type.name));
                                }

                                let mut is_static = false;
                                for modifier in &member.modifiers {
                                    match modifier {
                                        TIRModifier::Static => if is_static {
                                            return Err(CodegenError::DuplicateModifierOnField(TIRModifier::Static));
                                        } else {
                                            is_static = true
                                        },
                                        TIRModifier::Abstract => return Err(CodegenError::ModifierNotCompatibleForField(TIRModifier::Abstract)),
                                        TIRModifier::Native => return Err(CodegenError::ModifierNotCompatibleForField(TIRModifier::Native)),
                                    }
                                }

                                self.field_ref_pool.field_refs.push(FieldRef {
                                    associated_type_ref_index: type_decl.type_ref_index,
                                    name: name_and_type.name,
                                    index: field_index,
                                    index_in_all_members,
                                    type_info: name_and_type.type_info.clone(),
                                    is_static
                                });

                                field_index += 1;
                            }
                            TIRMemberKind::Method { name_and_type, parameters, .. } => {
                                let method_ref_index = self.method_ref_pool.method_refs.len();
                                let parameters: Vec<TIRTypeInfo> = parameters.iter().map(|p| p.type_info.clone()).collect();

                                if let Some(refs_hashmap) = self.method_ref_pool.name_and_type_ref_index_to_method_ref_indexes.get_mut(
                                    &(type_decl.type_ref_index, name_and_type.name)
                                ) {
                                    if let Some(_) = refs_hashmap.insert(parameters.clone(), method_ref_index) {
                                        return Err(CodegenError::MethodNameTaken(name_and_type.name));
                                    }
                                }
                                else {
                                    let mut refs_hashmap = HashMap::new();
                                    refs_hashmap.insert(parameters.clone(), method_ref_index);
                                    self.method_ref_pool.name_and_type_ref_index_to_method_ref_indexes.insert(
                                        (type_decl.type_ref_index, name_and_type.name),
                                        refs_hashmap
                                    );
                                }
                                if self.field_ref_pool.type_ref_index_and_name_to_field_ref_index.contains_key(
                                    &(type_decl.type_ref_index, name_and_type.name)
                                ) {
                                    return Err(CodegenError::MethodNameTaken(name_and_type.name));
                                }

                                let mut is_static = false;
                                let mut is_abstract = false;
                                let mut is_native = false;
                                for modifier in &member.modifiers {
                                    match modifier {
                                        TIRModifier::Static => if is_static {
                                            return Err(CodegenError::DuplicateModifierOnField(TIRModifier::Static));
                                        } else {
                                            is_static = true
                                        },
                                        TIRModifier::Abstract => if is_abstract {
                                            return Err(CodegenError::DuplicateModifierOnField(TIRModifier::Abstract));
                                        } else {
                                            is_abstract = true
                                        },
                                        TIRModifier::Native => if is_native {
                                            return Err(CodegenError::DuplicateModifierOnField(TIRModifier::Native));
                                        } else {
                                            is_native = true
                                        }
                                    }
                                }

                                self.method_ref_pool.method_refs.push(MethodRef {
                                    associated_type_ref_index: type_decl.type_ref_index,
                                    return_type: name_and_type.type_info.clone(),
                                    name: name_and_type.name,
                                    parameters,
                                    index: method_index,
                                    index_in_all_members,
                                    is_static,
                                    is_abstract,
                                    is_native
                                });

                                method_index += 1;
                            }
                        }

                        index_in_all_members += 1;
                    }
                }
            }
         }

        Ok(())
    }

    fn tir_type_info_to_type_info_format(&self, tir_type: &TIRTypeInfo) -> TypeInfoFormat {
        match &tir_type.kind {
            TIRTypeInfoKind::TypeRef { type_ref_index, array_dim, .. } => {
                TypeInfoFormat {
                    kind: TypeInfoKindFormat::TypeRef {
                        type_ref_index: *type_ref_index as u64
                    },
                    array_dim: *array_dim as u64
                }
            }
            TIRTypeInfoKind::Generic { .. } => todo!("Convert to oolang::Object"),
            TIRTypeInfoKind::Primitive { primitive, array_dim } => {
                TypeInfoFormat {
                    kind: match primitive {
                        PrimitiveType::Void => TypeInfoKindFormat::Void,
                        PrimitiveType::I64 => TypeInfoKindFormat::I64,
                        PrimitiveType::I32 => TypeInfoKindFormat::I32,
                        PrimitiveType::I16 => TypeInfoKindFormat::I16,
                        PrimitiveType::I8 => TypeInfoKindFormat::I8,
                        PrimitiveType::U64 => TypeInfoKindFormat::U64,
                        PrimitiveType::U32 => TypeInfoKindFormat::U32,
                        PrimitiveType::U16 => TypeInfoKindFormat::U16,
                        PrimitiveType::U8 => TypeInfoKindFormat::U8,
                        PrimitiveType::F64 => TypeInfoKindFormat::F64,
                        PrimitiveType::F32 => TypeInfoKindFormat::F32,
                        PrimitiveType::Boolean => TypeInfoKindFormat::Boolean,
                        PrimitiveType::Character => TypeInfoKindFormat::Character
                    },
                    array_dim: *array_dim as u64
                }
            }
        }
    }

    fn codegen_tir_type(&self, tir_type: &TIRType<'a>, bytecode: &mut Vec<u8>, poolfile_id: u64) -> Result<(), CodegenError<'a>> {
        match &tir_type.kind {
            TIRTypeKind::Class { members, super_class } => {
                // Contains field expression assignments
                let mut constructor_prefix: Vec<u8> = vec![];
                // Contains static field expression assignments
                let mut static_prefix: Vec<u8> = vec![];

                let type_ref = &self.type_ref_pool.type_refs[tir_type.type_ref_index];
                let class_type_ref = match &type_ref.kind {
                    TypeRefKind::Class(class) => class,
                    _ => unreachable!()
                };

                let mut fields = vec![];
                let mut methods = vec![];

                let super_class_type_ref_index = if let Some(sctri) = &class_type_ref.super_class {
                    match &sctri.kind {
                        TIRTypeInfoKind::TypeRef { type_ref_index, .. } => Some(*type_ref_index as u64),
                        _ => unreachable!()
                    }
                } else {
                    None
                };

                // TODO : optimize this so that we don't have to iterate through every field in the pool to get the fields of the current type
                for field_ref in &self.field_ref_pool.field_refs {
                    if field_ref.associated_type_ref_index == tir_type.type_ref_index {
                        fields.push(FieldFormat {
                            name: field_ref.name,
                            type_info: self.tir_type_info_to_type_info_format(&field_ref.type_info),
                            is_static: field_ref.is_static
                        });

                        match &members[field_ref.index_in_all_members].kind {
                            TIRMemberKind::Field { expression, .. } => {
                                if let Some(expression) = expression {
                                    self.codegen_tir_expr(
                                        &TIRExpr {
                                            kind: TIRExprKind::BinOp(
                                                Box::new(TIRExpr {
                                                    kind: TIRExprKind::VariableAccess(field_ref.name),
                                                    span: expression.span
                                                }),
                                                TIROperator::Assign,
                                                Box::new(expression.clone()
                                                )),
                                            span: expression.span
                                        },
                                        &mut constructor_prefix,
                                        todo!()
                                    )?;
                                }
                            }
                            _ => unreachable!()
                        }
                    }
                }

                todo!("methods");

                TypeFileFormat {
                    poolfile_id,
                    type_kind: TypeKindFormat::Class {
                        fields,
                        methods,
                        super_class_type_ref_index
                    }
                }.write(bytecode);
            }
        }

        Ok(())
    }

    pub fn get_bytecode(mut self) -> Result<Vec<BytecodeFile<'a>>, CodegenError<'a>> {
        self.create_field_and_method_refs()?;
        let mut bytecode_files = vec![];

        // TODO : replace by UUID
        let poolfile_id = 0xFF_FF_FF_FF_FF_FF_FF_FF;

        for tir_type in &self.tir_root.types {
            let mut bytecode_file = BytecodeFile {
                bytecode: vec![],
                file_path: self.type_ref_pool.type_refs[tir_type.type_ref_index].full_path.clone()
            };

            self.codegen_tir_type(tir_type, &mut bytecode_file.bytecode, poolfile_id)?;

            bytecode_files.push(bytecode_file);
        }

        Ok(bytecode_files)
    }
}

#[derive(Debug)]
pub enum CodegenError<'a> {
    MethodNameTaken(&'a str),
    FieldNameTaken(&'a str),
    DuplicateModifierOnField(TIRModifier),
    ModifierNotCompatibleForField(TIRModifier),
    DuplicateModifierOnMethod(TIRModifier)
}