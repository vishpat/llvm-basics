mod env;
use crate::env::*;
use std::path::Path;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::IntType;
use inkwell::values::FunctionValue;
use inkwell::AddressSpace;
use std::cell::RefCell;
use std::rc::Rc;

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
//     int a = 10;
//     int b = 0;
//     if (a > 0) {
//        /* If block */
//        b = 1;
//     } else {
//        /* Else block */
//        b = 2;
//     }
//     /* Merge block */
//     printf("%d\n", b);
// }

fn main() {
    let context = Context::create();
    let compiler = Compiler::new(&context);
    let main_block = compiler
        .context
        .append_basic_block(compiler.main_func, "entry");
    compiler.builder.position_at_end(main_block);
    let global_env = Rc::new(RefCell::new(Env::new(None)));

    let ptr = compiler.builder.build_alloca(compiler.i32_type, "a");
    compiler
        .builder
        .build_store(ptr, compiler.i32_type.const_int(10, false));
    global_env.borrow_mut().add(
        "a",
        Pointer {
            ptr,
            data_type: DataType::Number,
        },
    );

    let ptr = compiler.builder.build_alloca(compiler.i32_type, "b");
    compiler
        .builder
        .build_store(ptr, compiler.i32_type.const_int(0, false));
    global_env.borrow_mut().add(
        "b",
        Pointer {
            ptr,
            data_type: DataType::Number,
        },
    );

    let if_true_block = compiler
        .context
        .append_basic_block(compiler.main_func, "if_true");

    let if_false_block = compiler
        .context
        .append_basic_block(compiler.main_func, "if_false");

    let merge_block = compiler
        .context
        .append_basic_block(compiler.main_func, "merge");

    let a_ptr = global_env.borrow().get("a").unwrap().ptr;
    let lhs = compiler.builder.build_load(compiler.i32_type, a_ptr, "a");

    let comparison = compiler.builder.build_int_compare(
        inkwell::IntPredicate::SGT,
        lhs.into_int_value(),
        compiler.i32_type.const_int(0, false),
        "a > 0",
    );

    compiler
        .builder
        .build_conditional_branch(comparison, if_true_block, if_false_block);

    // Generate code for if true block
    compiler.builder.position_at_end(if_true_block);
    let b_ptr = global_env.borrow().get("b").unwrap().ptr;
    compiler
        .builder
        .build_store(b_ptr, compiler.i32_type.const_int(1, false));
    compiler.builder.build_unconditional_branch(merge_block);
    let then_block = compiler.builder.get_insert_block().unwrap();

    // Generate code for if false block
    compiler.builder.position_at_end(if_false_block);
    let b_ptr = global_env.borrow().get("b").unwrap().ptr;
    compiler
        .builder
        .build_store(b_ptr, compiler.i32_type.const_int(2, false));
    compiler.builder.build_unconditional_branch(merge_block);
    let else_block = compiler.builder.get_insert_block().unwrap();

    // Generate code for merge block
    compiler.builder.position_at_end(merge_block);
    let phi = compiler.builder.build_phi(compiler.i32_type, "phi");
    phi.add_incoming(&[
        (&compiler.i32_type.const_int(1, false), then_block),
        (&compiler.i32_type.const_int(2, false), else_block),
    ]);
    let b_ptr = global_env.borrow().get("b").unwrap().ptr;
    compiler
        .builder
        .build_store(b_ptr, phi.as_basic_value().into_int_value());

    let int_fmt_str = unsafe { compiler.builder.build_global_string("%d\n", "int_fmt_str") };

    let b_ptr = global_env.borrow().get("b").unwrap().ptr;
    let b = compiler.builder.build_load(compiler.i32_type, b_ptr, "b");
    compiler.builder.build_call(
        compiler.printf_func,
        &[int_fmt_str.as_pointer_value().into(), b.into()],
        "printf",
    );
    let ret_val = compiler.i32_type.const_int(0, false);
    compiler.builder.build_return(Some(&ret_val));
    compiler.module.print_to_file(Path::new("main.ll")).unwrap();
}
