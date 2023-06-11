use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::FloatType;    
use inkwell::values::FunctionValue;

pub struct Compiler<'ctx> {
    pub context: &'ctx Context,
    pub builder: Builder<'ctx>,
    pub module: Module<'ctx>,
    pub f32_type: FloatType<'ctx>, 
    pub main_func: FunctionValue<'ctx>,
  }

fn main() {
    println!("Hello, world!");
}
