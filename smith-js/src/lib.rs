use std::{fmt::Debug, panic, rc::Rc};
extern crate wee_alloc;

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
use smith_core::{Smith};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
fn init_wasm() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[wasm_bindgen]
pub struct SmithJS {
    backend: Smith,
}

#[wasm_bindgen]
impl SmithJS {
    #[wasm_bindgen(constructor)]
    pub fn new(src: &str) -> Self {
        Self {
            backend: Smith::new(src),
        }
    }

    pub fn serialize(&self, json: JsValue, typename: &str) -> Result<Box<[u8]>, String> {
        let json = js_sys::JSON::stringify(&json).unwrap().as_string().unwrap();
        let typ = self.backend
                    .get_type(typename)
                    .ok_or(format!("'{typename}' not found"))?;
        self.backend.json2binary(&json, &typ)
    }

    pub fn deserialize(
        &self,
        bin: &[u8],
        typename: &str,
    ) -> Result<JsValue, String> {
        let typ = self.backend
                    .get_type(typename)
                    .ok_or(format!("'{typename}' not found"))?;
        self.backend
            .binary2json(bin, &typ)
            .err_string()
            .map(|v| {
                js_sys::JSON::parse(&v.to_string()).unwrap()
            })
    }
}

trait ErrAsString<OK, ERR: Debug> {
    fn err_string(self) -> Result<OK, String>;
}

impl<OK, ERR: Debug> ErrAsString<OK, ERR> for Result<OK, ERR> {
    fn err_string(self) -> Result<OK, String> {
        self.map_err(|err| format!("{err:?}"))
    }
}
