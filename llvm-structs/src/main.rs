use std::path::Path;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::IntType;
use inkwell::types::StructType;
use inkwell::values::FunctionValue;
use inkwell::AddressSpace;

const MAIN_FUNC_NAME: &str = "main";

pub struct Compiler<'ctx> {
    pub context: &'ctx Context,
    pub builder: Builder<'ctx>,
    pub module: Module<'ctx>,
    pub i32_type: IntType<'ctx>,
    pub point_type: StructType<'ctx>,
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
        let point_type = context.struct_type(&[i32_type.into(), i32_type.into()], false);

        Compiler {
            context,
            builder,
            module,
            i32_type,
            main_func,
            printf_func,
            point_type,
        }
    }
}

//
// struct Point {
//    int a;
//    int b;
// };
//
// int main() {
//    struct Point p;
//    p.a = 10;
//    p.b = 20;
//    int c = p.a + p.b;
//    printf("%d\n", c);
//    return 0;
// }
//
fn main() {
    let context = Context::create();
    let compiler = Compiler::new(&context);
    let main_block = compiler
        .context
        .append_basic_block(compiler.main_func, "entry");
    compiler.builder.position_at_end(main_block);

    let point = compiler.builder.build_alloca(compiler.point_type, "p");

    let a = compiler
        .builder
        .build_struct_gep(compiler.point_type, point, 0, "a");
    let a = match a {
        Ok(a) => a,
        Err(e) => {
            println!("Error: {:?}", e);
            return;
        }
    };
    compiler
        .builder
        .build_store(a, compiler.i32_type.const_int(10, false));

    let b = compiler
        .builder
        .build_struct_gep(compiler.point_type, point, 1, "b");
    let b = match b {
        Ok(b) => b,
        Err(e) => {
            println!("Error: {:?}", e);
            return;
        }
    };
    compiler
        .builder
        .build_store(b, compiler.i32_type.const_int(20, false));

    let a = compiler.builder.build_load(compiler.i32_type, a, "a");
    let b = compiler.builder.build_load(compiler.i32_type, b, "b");

    let c = compiler
        .builder
        .build_int_add(a.into_int_value(), b.into_int_value(), "c");

    let int_fmt_str = unsafe { compiler.builder.build_global_string("%d\n", "int_fmt_str") };
    compiler.builder.build_call(
        compiler.printf_func,
        &[int_fmt_str.as_pointer_value().into(), c.into()],
        "printf",
    );

    compiler
        .builder
        .build_return(Some(&compiler.i32_type.const_int(0, false)));
    compiler.module.print_to_file(Path::new("main.ll")).unwrap();
}
