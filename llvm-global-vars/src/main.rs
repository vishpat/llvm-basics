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
// int a = 10;
// int b = 20;
//
// int main(int argc, char **argv) {
//     int c = a + b;
//     printf("%d\n", c);
//     return 0;
// }

fn main() {
    let context = Context::create();
    let compiler = Compiler::new(&context);
    let main_block = compiler
        .context
        .append_basic_block(compiler.main_func, "entry");
    compiler.builder.position_at_end(main_block);

    let a = compiler.module.add_global(compiler.i32_type, None, "a");
    a.set_initializer(&compiler.i32_type.const_int(10, false));

    let b = compiler.module.add_global(compiler.i32_type, None, "b");
    b.set_initializer(&compiler.i32_type.const_int(20, false));

    let lhs = compiler
        .module
        .get_global("a")
        .unwrap()
        .get_initializer()
        .unwrap();
    let rhs = compiler
        .module
        .get_global("b")
        .unwrap()
        .get_initializer()
        .unwrap();

    let c = compiler
        .builder
        .build_int_add(lhs.into_int_value(), rhs.into_int_value(), "c");
    let int_fmt_str = unsafe { compiler.builder.build_global_string("%d\n", "int_fmt_str") };
    compiler.builder.build_call(
        compiler.printf_func,
        &[int_fmt_str.as_pointer_value().into(), c.into()],
        "printf",
    );
    let ret_val = compiler.i32_type.const_int(0, false);
    compiler.builder.build_return(Some(&ret_val));
    compiler.module.print_to_file(Path::new("main.ll")).unwrap();
}