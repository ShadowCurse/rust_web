use yew::prelude::*;

enum Msg {

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

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="uk-position-center uk-background-default">
                <h1 class="uk-heading-medium">{"Web Video Chat in Rust"}</h1>
                <span class="uk-label">{"Hosting Session ID:"}</span>
                <h1 class="uk-heading-small">{"Peer A Video"}</h1>
                <video width="320" height="240" style="color: black; outline-style: solid;" autoplay="muted"></video>
                <br/>
                <button class="uk-button uk-button-default">{"Connect to Session"}</button>
                <input type="text" class="uk-input"/>
                <hr/>
                <h1 class="uk-heading-small">{"Peer B Video"}</h1>
                <video width="320" height="240" style="color: black; outline-style: solid;" autoplay="muted playsinline"></video>
                <br/>
                <button class="uk-button uk-button-default">{"Start Session"}</button>
                <hr/>
                <button class="uk-button uk-button-default">{"Print Client State"}</button>
                <button class="uk-button uk-button-default">{"Print Signalling Server State(In Terminal)"}</button>
            </div>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
