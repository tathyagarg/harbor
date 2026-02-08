// #[repr(C)]
// struct JsContext {
//     _private: [u8; 0],
// }
//
// #[link(name = "jsruntime", kind = "static")]
// unsafe extern "C" {
//     fn js_create_context() -> *mut JsContext;
//     fn js_destroy_context(ctx: *mut JsContext);
// }
//
// pub struct JsRuntime {
//     ctx: *mut JsContext,
// }
//
// impl JsRuntime {
//     pub fn new() -> Self {
//         println!("Creating JS runtime context...");
//
//         unsafe {
//             let ctx = js_create_context();
//             Self { ctx }
//         }
//     }
// }
//
// impl Drop for JsRuntime {
//     fn drop(&mut self) {
//         unsafe {
//             js_destroy_context(self.ctx);
//         }
//     }
// }

#[link(name = "js", kind = "static")]
unsafe extern "C" {}
