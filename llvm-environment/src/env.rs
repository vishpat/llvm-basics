use inkwell::values::PointerValue;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataType {
    Number,
}

#[derive(Debug, Clone)]
pub struct Pointer<'ctx> {
    pub ptr: PointerValue<'ctx>,
    pub data_type: DataType,
}

pub struct Env<'ctx> {
    parent: Option<Rc<RefCell<Env<'ctx>>>>,
    symbols: HashMap<String, Pointer<'ctx>>,
}

impl<'ctx> Env<'ctx> {
    pub fn new(parent: Option<Rc<RefCell<Env<'ctx>>>>) -> Self {
        Self {
            parent,
            symbols: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: &str, ptr: Pointer<'ctx>) {
        self.symbols.insert(name.to_string(), ptr);
    }

    pub fn get(&self, name: &str) -> Option<Pointer<'ctx>> {
        match self.symbols.get(name) {
            Some(value) => Some(value.clone()),
            None => self.parent.as_ref().and_then(|o| o.borrow().get(name)),
        }
    }
}
