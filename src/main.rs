mod parser;
mod spec;

use inkwell::{AddressSpace, context::Context};

use crate::spec::*;

const SAMPLE_CODE: &str = r#"
fn main() -> i32 {
    printf("Hello world");
    printf("My balls itch");
    printf("Blah blah blah");
    return 69;
}
"#;

fn main() {
    let program = program()
        .parse(SAMPLE_CODE.to_string())
        .expect("to parse program correctly.")
        .0;

    let context = Context::create();

    // Perhaps a language.
    let module = context.create_module("pal");

    let ptr_type = context.ptr_type(AddressSpace::default());

    // Later "extern" this
    module.add_function(
        "printf",
        context.i32_type().fn_type(&[ptr_type.into()], false),
        None,
    );

    for item in program.0 {
        match item {
            Item::FnDef(name, statements) => {
                let fn_element =
                    module.add_function(&name, context.i32_type().fn_type(&[], false), None);

                let fn_block = context.append_basic_block(fn_element, &name);

                let builder = context.create_builder();
                builder.position_at_end(fn_block);

                for statement in statements {
                    match statement {
                        Statement::FnCall(name, args) => {
                            let calling_fn = module.get_function(&name).unwrap();

                            let args = builder.build_global_string_ptr(&args, "").unwrap();

                            builder
                                .build_call(calling_fn, &[args.as_pointer_value().into()], "")
                                .unwrap();
                        }
                        Statement::Return(value) => {
                            builder
                                .build_return(Some(
                                    &context.i32_type().const_int(value.into(), false),
                                ))
                                .unwrap();
                        }
                    }
                }
            }
        }
    }

    module.verify().unwrap();

    module.write_bitcode_to_path("bitcode.ll");
}
