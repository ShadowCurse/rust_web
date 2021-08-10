use yew::prelude::*;

enum Msg {
    AddOne,
    SubOne,
}

struct Model {
    // `ComponentLink` is like a reference to a component.
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
    value: i64,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, value: 0 }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AddOne => {
                self.value += 1;
                // the value has changed so we need to
                // re-render for it to appear on the page
                true } Msg::SubOne => { self.value -= 1;
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        html! {
            <div id="container">
                <h1><a title="web_video_chat in rust" style="color: white;">{"Web Video Chat in Rust"}</a></h1>
                <label id="sessionid_heading" style="color: rgb(145, 133, 197);">{"Hosting Session ID:"}</label> 
                <label id="sessionid_lbl" style="color: rgb(143, 137, 201);"></label>
                <br/>
                <label id="session_connection_status" style="color: rgb(255, 255, 255);"></label>
                <label id="session_connection_status_error" style="color: rgb(200, 10, 10);"></label>
                <h3><a title="Peer A Video" style="color: white; ">{"Peer A Video"}</a></h3>
                <video id="peer_a_video" width="320" height="240" style="color: white; outline-style: solid;" autoplay="muted"></video>
                <br/>
                <button id="connect_to_session" style="height:50px">{"Connect to Session"}</button>
                <input type="text" id="sid_input" name="sid_input"/>
                <br/>
                <hr/>
                <h3><a title="Peer B Video" style="color: white; ">{"Peer B Video"}</a></h3>
                <video id="peer_b_video" width="320" height="240" style="color: white; outline-style: solid;" autoplay="muted playsinline"></video>
                <br/>
                <button id="start_session" style="height:50px">{"Start Session"}</button>
                <hr/>
                <button id="debug_client_state" style="height:50px">{"Print Client State"}</button>
                <label id="ws_conn_lbl" style="color: rgb(29, 161, 69);"></label> 
                <br/>
                <button id="debug_signal_server_state" style="height:50px">{"Print Signalling Server State(In Terminal)"}</button>
                <label id="ws_conn_lbl_err" style="color: red;"></label>
                <br/>
            </div>
        }
  } 
    
}

fn main() {
    yew::start_app::<Model>();
}
