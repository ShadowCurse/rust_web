use wasm_bindgen::prelude::*;
use web_sys::{
    console,
    MessageEvent,
    ErrorEvent,
    RtcPeerConnection,
    WebSocket,
    BinaryType,
};
use wasm_bindgen::*;

pub fn open_web_socket(ws_ip_port: &str) -> Result<WebSocket, JsValue> {
    let ws = WebSocket::new(ws_ip_port)?;
    ws.set_binary_type(BinaryType::Arraybuffer);

    let on_message = Closure::wrap(Box::new(|event: MessageEvent| {
        if let Ok(abuf) = event.data().dyn_into::<js_sys::ArrayBuffer>() {
            console::log_1(&format!("ws: message event, received arraybuffer: {:?}", abuf).into());
        } else if let Ok(blob) = event.data().dyn_into::<web_sys::Blob>() {
            console::log_1(&format!("ws: message event, received blob: {:?}", blob).into());
        } else if let Ok(txt) = event.data().dyn_into::<js_sys::JsString>() {
            console::log_1(&format!("ws: message event, received string: {:?}", txt).into());
        }
    }) as Box<dyn FnMut(MessageEvent)>);
    ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    on_message.forget();

    let on_error = Closure::wrap(Box::new(|error: ErrorEvent| {
        console::log_1(&format!("ws: error: {:?}", error).into());
    }) as Box<dyn FnMut(ErrorEvent)>);
    ws.set_onerror(Some(on_error.as_ref().unchecked_ref()));
    on_error.forget();

    let on_open = Closure::wrap(Box::new(|_| {
        console::log_1(&"ws: opened".into());
    }) as Box<dyn FnMut(JsValue)>);
    ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
    on_open.forget();

    Ok(ws)
}