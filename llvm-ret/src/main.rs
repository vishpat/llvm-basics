use std::path::Path;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::IntType;
use inkwell::values::FunctionValue;

const MAIN_FUNC_NAME: &str = "main";

pub struct Compiler<'ctx> {
    pub context: &'ctx Context,
    pub builder: Builder<'ctx>,
    pub module: Module<'ctx>,
    pub i32_type: IntType<'ctx>,
    pub main_func: FunctionValue<'ctx>,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        let builder = context.create_builder();
        let module = context.create_module(MAIN_FUNC_NAME);
        let i32_type = context.i32_type();
        let main_func = module.add_function(MAIN_FUNC_NAME, i32_type.fn_type(&[], false), None);
        Compiler {
            context,
            builder,
            module,
            i32_type,
            main_func,
        }
    }
}

const HG2G: u64 = 108;
fn main() {
    let context = Context::create();
    let compiler = Compiler::new(&context);
    let main_block = compiler
        .context
        .append_basic_block(compiler.main_func, "entry");
    compiler.builder.position_at_end(main_block);
    let ret_val = compiler.i32_type.const_int(HG2G, false);
    compiler.builder.build_return(Some(&ret_val));
    compiler.module.print_to_file(Path::new("llvm_ret.ll")).unwrap();
}
