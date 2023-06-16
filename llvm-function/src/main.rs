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
// int sum(int a, int b) {
//     return a + b;
// }
//
// int main() {
//     int a = 10;
//     int b = 0;
//     int c = sum(a, b);
//
//     printf("%d\n", b);
// }

fn main() {
    let context = Context::create();
    let compiler = Compiler::new(&context);

    let func_params = [compiler.i32_type.into(), compiler.i32_type.into()];

    let func_type = compiler.i32_type.fn_type(&func_params, false);
    let func = compiler.module.add_function("sum", func_type, None);
    let current_bb = compiler.context.append_basic_block(func, "entry");
    compiler.builder.position_at_end(current_bb);

    let a_ptr = compiler.builder.build_alloca(compiler.i32_type, "a");
    compiler
        .builder
        .build_store(a_ptr, func.get_nth_param(0).unwrap());

    let b_ptr = compiler.builder.build_alloca(compiler.i32_type, "b");
    compiler
        .builder
        .build_store(b_ptr, func.get_nth_param(1).unwrap());

    let a = compiler
        .builder
        .build_load(compiler.i32_type, a_ptr, "a")
        .into_int_value();
    let b = compiler
        .builder
        .build_load(compiler.i32_type, b_ptr, "b")
        .into_int_value();

    let c = compiler.builder.build_int_add(a, b, "c");
    compiler.builder.build_return(Some(&c));
    compiler.builder.position_at_end(current_bb);

    let main_block = compiler
        .context
        .append_basic_block(compiler.main_func, "entry");
    compiler.builder.position_at_end(main_block);

    let a_ptr = compiler.builder.build_alloca(compiler.i32_type, "a");
    compiler
        .builder
        .build_store(a_ptr, compiler.i32_type.const_int(10, false));

    let b_ptr = compiler.builder.build_alloca(compiler.i32_type, "b");
    compiler
        .builder
        .build_store(b_ptr, compiler.i32_type.const_int(20, false));

    let func_args = [
        compiler
            .builder
            .build_load(compiler.i32_type, a_ptr, "a")
            .into(),
        compiler
            .builder
            .build_load(compiler.i32_type, b_ptr, "b")
            .into(),
    ];
    let sum_func = compiler.module.get_function("sum").unwrap();
    let c = compiler
        .builder
        .build_call(sum_func, &func_args, "sum")
        .try_as_basic_value()
        .left()
        .unwrap()
        .into_int_value();

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
