use crate::log;
use js_sys::Reflect;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{RtcPeerConnection, RtcSdpType, RtcSessionDescriptionInit};

pub async fn create_sdp_offer(connection: &RtcPeerConnection) -> Result<String, JsValue> {
    log("create_sdp_offer");
    let offer = JsFuture::from(connection.create_offer()).await?;
    let offer_sdp = Reflect::get(&offer, &JsValue::from_str("sdp"))?
        .as_string()
        .unwrap();
    let mut offer_obj = RtcSessionDescriptionInit::new(RtcSdpType::Offer);
    offer_obj.sdp(&offer_sdp);

    let sld_promise = connection.set_local_description(&offer_obj);
    JsFuture::from(sld_promise).await?;

    log("ok");
    Ok(offer_sdp)
}

pub async fn create_sdp_answer(
    connection: &RtcPeerConnection,
    offer: &str,
) -> Result<String, JsValue> {
    log("create_sdp_answer");
    let mut offer_obj = RtcSessionDescriptionInit::new(RtcSdpType::Offer);
    offer_obj.sdp(&offer);
    let srd_promise = connection.set_remote_description(&offer_obj);
    JsFuture::from(srd_promise).await?;

    let answer = JsFuture::from(connection.create_answer()).await?;
    let answer_sdp = Reflect::get(&answer, &JsValue::from_str("sdp"))?
        .as_string()
        .unwrap();

    let mut answer_obj = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
    answer_obj.sdp(&answer_sdp);

    let sld_promise = connection.set_local_description(&answer_obj);
    JsFuture::from(sld_promise).await?;

    log("ok");
    Ok(answer_sdp)
}

pub async fn handle_sdp_answer(
    connection: &RtcPeerConnection,
    answer: &str,
) -> Result<(), JsValue> {
    log("handle_sdp_answer");
    let mut rtc_answer = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
    rtc_answer.sdp(answer);
    let promise = connection.set_remote_description(&rtc_answer);
    JsFuture::from(promise).await?;
    log("ok");
    Ok(())
}
