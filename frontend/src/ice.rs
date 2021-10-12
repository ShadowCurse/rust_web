use crate::{log, log_error};
use serde::{Deserialize, Serialize};
use signalling_protocol::*;
use std::convert::TryInto;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    HtmlVideoElement, MediaStream, RtcIceCandidate, RtcIceCandidateInit, RtcIceConnectionState,
    RtcPeerConnection, RtcPeerConnectionIceEvent, WebSocket,
};

#[derive(Debug, Serialize, Deserialize)]
struct IceCandidate {
    pub candidate: String,
    pub sdp_mid: String,
    pub sdp_m_line_index: u16,
}

pub fn setup_rtc_connection_ice(
    connection: &RtcPeerConnection,
    session_id: SessionId,
    web_socket: WebSocket,
) {
    log("setup_rtc_connection_ice");
    let on_ice_candidate =
        Closure::wrap(Box::new(
            move |event: RtcPeerConnectionIceEvent| match event.candidate() {
                Some(candidate) => {
                    let candidate = IceCandidate {
                        candidate: candidate.candidate(),
                        sdp_mid: candidate.sdp_mid().unwrap(),
                        sdp_m_line_index: candidate.sdp_m_line_index().unwrap(),
                    };

                    let candidate = serde_json::to_string(&candidate).unwrap();

                    let signal = Signal::ICECandidate(session_id.clone(), candidate);
                    let ice_candidate: String = serde_json::to_string(&signal).unwrap();

                    match web_socket.send_with_str(&ice_candidate) {
                        Ok(_) => log("ICE candidate send"),
                        Err(_) => log_error("error sending ICE candidate"),
                    }
                }
                None => {
                    log_error("no ICE candidate found");
                }
            },
        ) as Box<dyn FnMut(RtcPeerConnectionIceEvent)>);
    connection.set_onicecandidate(Some(on_ice_candidate.as_ref().unchecked_ref()));
    on_ice_candidate.forget();

    let connection_clone = connection.clone();
    let on_ice_state_change =
        Closure::wrap(
            Box::new(move || match connection_clone.ice_connection_state() {
                RtcIceConnectionState::Connected => {
                    log("RtcIceConnectionState::Connected");
                    let remote_streams = connection_clone.get_remote_streams().to_vec();

                    // for now handles only 1 stream
                    let first_stream = remote_streams[0].clone();
                    let media_stream: MediaStream = first_stream.try_into().unwrap();

                    let window = web_sys::window().unwrap();
                    let doc = window.document().unwrap();
                    let video_element = doc
                        .get_element_by_id("external_video")
                        .expect("no external_video element");
                    let video_element = video_element.dyn_into::<HtmlVideoElement>().unwrap();
                    video_element.set_src_object(Some(&media_stream));
                }
                _ => {
                    log_error(&format!(
                        "RtcConnectionState {:?}",
                        connection_clone.ice_connection_state()
                    ));
                }
            }) as Box<dyn FnMut()>,
        );
    connection.set_oniceconnectionstatechange(Some(on_ice_state_change.as_ref().unchecked_ref()));
    on_ice_state_change.forget();
    log("ok");
}

pub async fn handle_ice_candidate(
    connection: &RtcPeerConnection,
    candidate: &str,
) -> Result<(), JsValue> {
    log("handle_ice_candidate");
    let ice_candidate: IceCandidate = match serde_json::from_str(candidate) {
        Ok(x) => x,
        Err(_) => return Err("Could not deserialize ice candidate".into()),
    };

    let mut rtc_ice = RtcIceCandidateInit::new(&"");
    rtc_ice.candidate(&ice_candidate.candidate);
    rtc_ice.sdp_m_line_index(Some(ice_candidate.sdp_m_line_index));
    rtc_ice.sdp_mid(Some(&ice_candidate.sdp_mid));

    match RtcIceCandidate::new(&rtc_ice) {
        Ok(x) => {
            let promise = connection
                .clone()
                .add_ice_candidate_with_opt_rtc_ice_candidate(Some(&x));
            JsFuture::from(promise).await?;
        }
        Err(e) => return Err(e),
    }
    log("ok");
    Ok(())
}
