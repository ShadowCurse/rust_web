use wasm_bindgen::prelude::*;
use wasm_bindgen::*;
use web_sys::*;
use yew::prelude::*;

use signalling_protocol::*;
use std::cell::RefCell;
use std::fmt::Formatter;
use std::rc::Rc;

mod sdp;
use sdp::*;

mod ice;
use ice::*;

#[derive(Debug)]
enum Msg {
    Initialize,
    CreatedMedia(MediaStream),
    FailedMedia(JsValue),
    CreateSession,
    ConnectToSession,
    ReceivedMessageEvent(MessageEvent),
    EventHandled(bool),
    EventError(JsValue),
}

enum SessionStatus {
    Connected,
    NotConnected,
    Error,
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionStatus::Connected => write!(f, "connected"),
            SessionStatus::NotConnected => write!(f, "not connected"),
            SessionStatus::Error => write!(f, "error"),
        }
    }
}

pub fn log(msg: &str) {
    console::log_1(&msg.into());
}

pub fn log_error(msg: &str) {
    console::log_1(&msg.into());
}

struct ModelData {
    server_socket: String,
    local_stream: Option<MediaStream>,
    web_socket: Option<WebSocket>,
    rtc_connection: RtcPeerConnection,
    session_id: SessionId,
    session_status: SessionStatus,
}

struct Model {
    link: ComponentLink<Self>,
    data: Rc<RefCell<ModelData>>,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let model_data = Rc::new(RefCell::new(ModelData {
            server_socket: "wss://0.0.0.0:9999".to_string(),
            local_stream: None,
            web_socket: None,
            rtc_connection: RtcPeerConnection::new().unwrap_throw(),
            session_id: SessionId::default(),
            session_status: SessionStatus::NotConnected,
        }));
        Self {
            link,
            data: model_data,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        log(&format!("hadling message: {:?}", msg));
        match msg {
            Msg::Initialize => {
                log("Starting session");

                log("Initializing video");
                self.link.send_future(async {
                    match Self::init_video().await {
                        Ok(md) => Msg::CreatedMedia(md),
                        Err(e) => Msg::FailedMedia(e),
                    }
                });
                

                log("Initializing websocket");
                match self.open_web_socket() {
                    Ok(socket) => {
                        log("successfully create websocket");
                        self.data.borrow_mut().web_socket = Some(socket);
                    }
                    Err(e) => {
                        log_error(&format!("failed to create websocket with error: {:?}", e));
                        return false;
                    }
                };
            }
            Msg::CreatedMedia(media) => {
                log("successfully create media device");
                self.data.borrow_mut().local_stream = Some(media.clone());

                self.data.borrow()
                    .rtc_connection 
                    .add_stream(&media);
            }
            Msg::FailedMedia(e) => {
                log_error(&format!(
                    "failed to create media device with error: {:?}",
                    e
                ));
            }
            Msg::CreateSession => match self.data.borrow_mut().web_socket.as_ref() {
                Some(socket) => {
                    let signal_new_session = Signal::SessionNew;
                    let message = match serde_json::to_string(&signal_new_session) {
                        Ok(msg) => msg,
                        Err(e) => {
                            log_error(&format!(
                                "error serializing SessionNew{:?}: {:?}",
                                signal_new_session, e
                            ));
                            return false;
                        }
                    };
                    let _ = socket.send_with_str(&message);
                }
                None => {
                    log_error("web socket not opened");
                }
            },
            Msg::ConnectToSession => match self.data.borrow_mut().web_socket.as_ref() {
                Some(socket) => {
                    let session = match Self::get_session_to_connect() {
                        Ok(session) => SessionId::new(session),
                        Err(e) => {
                            log_error(&format!("error getting session to connect to: {:?}", e));
                            return false;
                        }
                    };
                    let signal_session_join = Signal::SessionJoin(session);
                    let message = match serde_json::to_string(&signal_session_join) {
                        Ok(msg) => msg,
                        Err(e) => {
                            log_error(&format!(
                                "error serializing SessionJoin{:?}: {:?}",
                                signal_session_join, e
                            ));
                            return false;
                        }
                    };
                    let _ = socket.send_with_str(&message);
                }
                None => {
                    log_error("web socket not opened");
                }
            },
            Msg::ReceivedMessageEvent(event) => {
                if let Ok(message) = event.data().dyn_into::<js_sys::JsString>() {
                    let data = self.data.clone();
                    self.link.send_future(async move {
                        match Self::handle_message(data, String::from(message)).await {
                            Ok(redraw) => Msg::EventHandled(redraw),
                            Err(e) => Msg::EventError(e),
                        }
                    });
                } else {
                    log_error("received invalid message event type");
                }
            }
            Msg::EventHandled(result) => return result,
            Msg::EventError(error) => {
                log_error(&format!("error handling the message: {:?}", error));
            }
        }
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        let initialize = self.link.callback(|_| Msg::Initialize);
        let create_session = self.link.callback(|_| Msg::CreateSession);
        let connect_to_session = self.link.callback(|_| Msg::ConnectToSession);
        html! {
            <div class="uk-position-center uk-background-default">
                <h1 class="uk-heading-medium">{"Web Video Chat in Rust"}</h1>
                <span class="uk-label">{"Hosting Session ID: "}</span>
                <span class="uk-text-default">{ &self.data.borrow().session_id.value() }</span>
                <br/>
                <span class="uk-label">{" Status: "}{ &self.data.borrow().session_status }</span>
                <br/>
                <span class="uk-label">{"Current server web socket: "}{ &self.data.borrow().server_socket }</span>
                <h1 class="uk-heading-small">{"Peer A Video"}</h1>
                <video id="external_video" width="320" height="240" style="color: black; outline-style: solid;" autoplay=true></video>
                <br/>
                <button class="uk-button uk-button-default" onclick={connect_to_session}>{"Connect to Session"}</button>
                <input id="session_to_connect" type="text" class="uk-input"/>
                <hr/>
                <h1 class="uk-heading-small">{"Peer B Video"}</h1>
                <video id="local_video" width="320" height="240" style="color: black; outline-style: solid;" autoplay=true></video>
                <br/>
                <button class="uk-button uk-button-default" onclick={initialize}>{"Initialize"}</button>
                <hr/>
                <button class="uk-button uk-button-default" onclick={create_session}>{"Create session"}</button>
                <br/>
            </div>
        }
    }
}

impl Model {
    async fn init_video() -> Result<MediaStream, JsValue> {
        let window = web_sys::window().ok_or("no window found")?;
        let navigator = window.navigator();
        let media_device = navigator.media_devices()?;
        let stream_promise = media_device.get_display_media()?; 
        let doc = window.document().ok_or("no doc found")?;
        let video_element = doc
            .get_element_by_id("local_video")
            .expect("no local_video element");
        let video_element = video_element.dyn_into::<HtmlVideoElement>()?;

        let media_stream = match wasm_bindgen_futures::JsFuture::from(stream_promise).await {
            Ok(ms) => MediaStream::from(ms),
            Err(e) => {
                return Err(format!("error in getting media stream: {:?}", e).into());
            }
        };
        video_element.set_src_object(Some(&media_stream));
        Ok(media_stream)
    }

    fn open_web_socket(&self) -> Result<WebSocket, JsValue> {
        let ws = WebSocket::new(&self.data.borrow().server_socket)?;
        ws.set_binary_type(BinaryType::Arraybuffer);

        let on_message = self.link.callback(Msg::ReceivedMessageEvent);
        let closure = Closure::wrap(Box::new(move |event: MessageEvent| on_message.emit(event))
            as Box<dyn FnMut(MessageEvent)>);
        ws.set_onmessage(Some(closure.as_ref().unchecked_ref()));
        closure.forget();

        let on_error = Closure::wrap(Box::new(|error: ErrorEvent| {
            log_error(&format!("ws: error: {:?}", error));
        }) as Box<dyn FnMut(ErrorEvent)>);
        ws.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        on_error.forget();

        let on_open = Closure::wrap(Box::new(|_| {
            log("ws: opened");
        }) as Box<dyn FnMut(JsValue)>);
        ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
        on_open.forget();

        Ok(ws)
    }

    fn get_session_to_connect() -> Result<String, JsValue> {
        let window = web_sys::window().ok_or("no window found")?;
        let doc = window.document().ok_or("no doc found")?;
        let video_element = doc
            .get_element_by_id("session_to_connect")
            .expect("no session_to_connect element");
        let input = video_element.dyn_into::<HtmlInputElement>()?;
        Ok(input.value())
    }

    async fn handle_message(
        data: Rc<RefCell<ModelData>>,
        message: String,
    ) -> Result<bool, JsValue> {
        let signal: Signal = match serde_json::from_str(&message) {
            Ok(x) => x,
            Err(_) => return Err("could not deserialize message".into()),
        };
        log(&format!("hadling signal: {:?}", signal));
        let res = match signal {
            Signal::NewUser(user_id) => {
                log(&format!("new user id: {:?}", user_id));
                false
            }
            Signal::SessionCreated(session_id) => {
                data.borrow_mut().session_id = session_id.clone();

                setup_rtc_connection_ice(
                    &data.borrow().rtc_connection,
                    session_id,
                    data.borrow().web_socket.as_ref().unwrap().clone(),
                );
                true
            }
            Signal::SessionJoinSuccess(session_id) => {
                data.borrow_mut().session_status = SessionStatus::Connected;

                setup_rtc_connection_ice(
                    &data.borrow().rtc_connection,
                    session_id.clone(),
                    data.borrow().web_socket.as_ref().unwrap().clone(),
                );

                let offer = create_sdp_offer(&data.borrow().rtc_connection).await?;

                let msg = Signal::VideoOffer(session_id, offer.into());
                let result: String = match serde_json::to_string(&msg) {
                    Ok(x) => x,
                    Err(e) => return Err(e.to_string().into()),
                };

                match data
                    .borrow()
                    .web_socket
                    .as_ref()
                    .unwrap()
                    .send_with_str(&result)
                {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }

                true
            }
            Signal::SessionJoinError(_) => {
                data.borrow_mut().session_status = SessionStatus::Error;
                true
            }
            Signal::VideoOffer(_, offer) => {
                let answer = create_sdp_answer(&data.borrow().rtc_connection, &offer).await?;

                let msg = Signal::VideoAnswer(data.borrow().session_id.clone(), answer.into());
                let result: String = match serde_json::to_string(&msg) {
                    Ok(x) => x,
                    Err(e) => return Err(e.to_string().into()),
                };

                match data
                    .borrow()
                    .web_socket
                    .as_ref()
                    .unwrap()
                    .send_with_str(&result)
                {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }

                true
            }
            Signal::VideoAnswer(_, answer) => {
                handle_sdp_answer(&data.borrow().rtc_connection, &answer).await?;
                true
            }
            Signal::ICECandidate(_, candidate) => {
                handle_ice_candidate(&data.borrow().rtc_connection, &candidate).await?;
                true
            }
            Signal::ICEError(_, error) => {
                log_error(&format!("ice error: {}", error));
                false
            }
            _ => return Err("received invalid signal".into()),
        };
        Ok(res)
    }
}

fn main() {
    yew::start_app::<Model>();
}
