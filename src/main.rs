use std::path::Path;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::IntType;
use inkwell::values::FunctionValue;
use inkwell::AddressSpace;

const MAIN_FUNC_NAME: &str = "main";

pub struct Compiler<'ctx> {
    pub context: &'ctx Context,
    pub builder: Builder<'ctx>,
    pub module: Module<'ctx>,
    pub i32_type: IntType<'ctx>,
    pub main_func: FunctionValue<'ctx>,
    pub printf_func: FunctionValue<'ctx>,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        let builder = context.create_builder();
        let module = context.create_module(MAIN_FUNC_NAME);
        let i32_type = context.i32_type();
        let main_func = module.add_function(MAIN_FUNC_NAME, i32_type.fn_type(&[], false), None);
        let printf_func: FunctionValue = module.add_function(
            "printf",
            i32_type.fn_type(&[i32_type.ptr_type(AddressSpace::default()).into()], true),
            None,
        );

        Compiler {
            context,
            builder,
            module,
            i32_type,
            main_func,
            printf_func,
        }
    }
}

fn main() {
    let context = Context::create();
    let compiler = Compiler::new(&context);
    let main_block = compiler
        .context
        .append_basic_block(compiler.main_func, "entry");
    compiler.builder.position_at_end(main_block);
    let ret_val = compiler.i32_type.const_int(0, false);
    let hello_world_str = unsafe {
        compiler
            .builder
            .build_global_string("Hello World\n", "hello_world")
    };

    compiler.builder.build_call(
        compiler.printf_func,
        &[hello_world_str.as_pointer_value().into()],
        "printf",
    );
    compiler.builder.build_return(Some(&ret_val));
    compiler.module.print_to_file(Path::new("main.ll")).unwrap();
}
