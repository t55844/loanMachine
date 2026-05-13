// loan_machine_web/src/wallet_auth/auth_header.rs

#[cfg(target_arch = "wasm32")]
pub async fn current_access_token() -> Option<String> {
    use js_sys::{Function, Promise, Reflect};
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window()?;
    let func = Reflect::get(&window, &JsValue::from_str("loan_machine_get_access_token")).ok()?;
    let func = func.dyn_into::<Function>().ok()?;
    let promise: Promise = func.call0(&JsValue::NULL).ok()?.dyn_into().ok()?;
    JsFuture::from(promise).await.ok()?.as_string()
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn current_access_token() -> Option<String> { None }