use inkwell::{
    AddressSpace,
    builder::Builder,
    context::Context,
    module::Module as CodegenModule,
    types::{BasicType, BasicTypeEnum},
    values::{BasicValue, BasicValueEnum},
};
use thiserror::Error;

use crate::spec::ast::*;

#[derive(Error, Debug)]
pub enum CodegenError {
    #[error("tried to reference a type that does not exist.")]
    TypeDoesNotExist,
}

pub fn generate_codegen_type<'ctx>(
    context: &'ctx Context,
    typ: &Type,
) -> anyhow::Result<BasicTypeEnum<'ctx>> {
    match typ {
        Type::Atomic(ident) => match &ident[..] {
            "u32" => Ok(context.i32_type().as_basic_type_enum()),
            "char" => Ok(context.i8_type().as_basic_type_enum()),
            _ => Err(CodegenError::TypeDoesNotExist.into()),
        },
        Type::Pointer(_) => Ok(context
            .ptr_type(AddressSpace::default())
            .as_basic_type_enum()),
    }
}

pub fn generate_codegen_expression<'ctx>(
    context: &'ctx Context,
    builder: &'ctx Builder,
    expression: &Expression,
) -> anyhow::Result<BasicValueEnum<'ctx>> {
    match expression {
        Expression::NumericLiteral(value) => Ok(context
            .i32_type()
            .const_int(*value, false)
            .as_basic_value_enum()),
        Expression::StringLiteral(value) => Ok(builder
            .build_global_string_ptr(&value, "")?
            .as_basic_value_enum()),
    }
}

pub fn generate_codegen_statement(
    context: &Context,
    statement: &Statement,
    builder: &Builder,
) -> anyhow::Result<()> {
    match statement {
        Statement::Return(expression) => {
            builder.build_return(Some(&generate_codegen_expression(
                context, builder, expression,
            )?))?;
        }
    }

    Ok(())
}

pub fn generate_codegen_item<'a>(
    context: &'a Context,
    module: &CodegenModule<'a>,
    item: &Item,
) -> anyhow::Result<()> {
    match item {
        Item::ExternFunctionDefinition(name, args, typ) => {
            let argument_types: Vec<_> = args
                .iter()
                .map(|(_, typ)| generate_codegen_type(context, typ).unwrap().into())
                .collect();

            module.add_function(
                name,
                generate_codegen_type(context, typ)?.fn_type(&argument_types, false),
                None,
            );
        }
        Item::FunctionDeclaration(name, args, typ, body) => {
            let argument_types: Vec<_> = args
                .iter()
                .map(|(_, typ)| generate_codegen_type(context, typ).unwrap().into())
                .collect();

            let fn_decl = module.add_function(
                &name,
                generate_codegen_type(context, typ)?.fn_type(&argument_types, false),
                None,
            );
            let fn_block = context.append_basic_block(fn_decl, &name);

            let builder = context.create_builder();
            builder.position_at_end(fn_block);

            for statement in body {
                generate_codegen_statement(context, statement, &builder)?;
            }
        }
    }

    Ok(())
}

pub fn generate_codegen_module<'a>(
    context: &'a Context,
    module: &Module,
) -> anyhow::Result<CodegenModule<'a>> {
    let codegen_module = context.create_module(&module.0);

    for item in &module.1 {
        generate_codegen_item(context, &codegen_module, item)?;
    }

    Ok(codegen_module)
}
