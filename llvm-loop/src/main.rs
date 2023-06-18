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

//
// int main() {
//   int a = 10;
//   while (a > 0) {
//     a = a - 1;
//     printf("%d\n", a);
//   }
//   return 0;
// }

fn main() {
    let context = Context::create();
    let compiler = Compiler::new(&context);
    let main_block = compiler
        .context
        .append_basic_block(compiler.main_func, "entry");
    compiler.builder.position_at_end(main_block);

    let ptr = compiler.builder.build_alloca(compiler.i32_type, "a");
    compiler
        .builder
        .build_store(ptr, compiler.i32_type.const_int(10, false));

    let cond_block = compiler
        .context
        .append_basic_block(compiler.main_func, "cond");
    let body_block = compiler
        .context
        .append_basic_block(compiler.main_func, "body");
    let loop_end_block = compiler
        .context
        .append_basic_block(compiler.main_func, "loop_end");

    compiler.builder.build_unconditional_branch(cond_block);

    compiler.builder.position_at_end(cond_block);
    let lhs = compiler.builder.build_load(compiler.i32_type, ptr, "a");
    let comparison = compiler.builder.build_int_compare(
        inkwell::IntPredicate::SGT,
        lhs.into_int_value(),
        compiler.i32_type.const_int(0, false),
        "a > 0",
    );

    compiler
        .builder
        .build_conditional_branch(comparison, body_block, loop_end_block);

    // Generate code for body block
    compiler.builder.position_at_end(body_block);
    let lhs = compiler.builder.build_load(compiler.i32_type, ptr, "a");
    let new_val = compiler.builder.build_int_sub(
        lhs.into_int_value(),
        compiler.i32_type.const_int(1, false),
        "a - 1",
    );
    compiler.builder.build_store(ptr, new_val);

    let int_fmt_str = unsafe { compiler.builder.build_global_string("%d\n", "int_fmt_str") };
    let a = compiler.builder.build_load(compiler.i32_type, ptr, "a");
    compiler.builder.build_call(
        compiler.printf_func,
        &[int_fmt_str.as_pointer_value().into(), a.into()],
        "printf",
    );

    compiler.builder.build_unconditional_branch(cond_block);

    // Generate code for merge block
    compiler.builder.position_at_end(loop_end_block);

    let ret_val = compiler.i32_type.const_int(0, false);
    compiler.builder.build_return(Some(&ret_val));
    compiler.module.print_to_file(Path::new("main.ll")).unwrap();
}
