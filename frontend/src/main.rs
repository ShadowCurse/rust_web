use wasm_bindgen::*;
use web_sys::*;
use yew::prelude::*;

enum Msg {
    StartSession,
    GotMedia(MediaStream),
    FailedMedia(JsValue),
}

struct Model {
    // `ComponentLink` is like a reference to a component.
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::StartSession => {
                console::log_1(&"Starting session".into());
                console::log_1(&"Initializing video".into());

                self.link.send_future(async {
                    match Self::init_video().await {
                        Ok(md) => Msg::GotMedia(md),
                        Err(e) => Msg::FailedMedia(e),
                    }
                });
            }
            Msg::GotMedia(media) => {
                console::log_1(&"successfully".into());
                console::log_2(&"media_stream: {}".into(), &media);
            }
            Msg::FailedMedia(e) => {
                console::log_1(&format!("failed with error: {:?}", e).into());
            }
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        let start_session = self.link.callback(|_| Msg::StartSession);
        html! {
            <div class="uk-position-center uk-background-default">
                <h1 class="uk-heading-medium">{"Web Video Chat in Rust"}</h1>
                <span class="uk-label">{"Hosting Session ID:"}</span>
                <h1 class="uk-heading-small">{"Peer A Video"}</h1>
                <video width="320" height="240" style="color: black; outline-style: solid;" autoplay=false></video>
                <br/>
                <button class="uk-button uk-button-default">{"Connect to Session"}</button>
                <input type="text" class="uk-input"/>
                <hr/>
                <h1 class="uk-heading-small">{"Peer B Video"}</h1>
                <video id="local_video" width="320" height="240" style="color: black; outline-style: solid;" autoplay=false></video>
                <br/>
                <button class="uk-button uk-button-default" onclick={start_session}>{"Start Session"}</button>
                <hr/>
                <button class="uk-button uk-button-default">{"Print Client State"}</button>
                <button class="uk-button uk-button-default">{"Print Signalling Server State(In Terminal)"}</button>
            </div>
        }
    }
}

impl Model {
    async fn init_video() -> Result<MediaStream, JsValue> {
        let window = web_sys::window().ok_or("no window found")?;
        let navigator = window.navigator();
        let media_device = navigator.media_devices()?;
        let mut constrains = MediaStreamConstraints::new();
        constrains.audio(&JsValue::FALSE);
        constrains.video(&JsValue::TRUE);
        let stream_promise = media_device
            .get_user_media_with_constraints(&constrains)?;

        let doc = window.document().ok_or("no doc found")?;
        let video_element = doc
            .get_element_by_id("local_video")
            .expect("no local_video element");
        let video_element = video_element
            .dyn_into::<HtmlVideoElement>()?;

        let media_stream = match wasm_bindgen_futures::JsFuture::from(stream_promise).await {
            Ok(ms) => {
                MediaStream::from(ms)
            }
            Err(e) => {
                return Err(format!("error in getting media stream: {:?}", e).into());
            }
        };
        video_element.set_src_object(Some(&media_stream));
        Ok(media_stream)
    }
}

fn main() {
    yew::start_app::<Model>();
}
