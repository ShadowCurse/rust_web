use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::{
    ws::{Message, WebSocket},
    Filter, Rejection, Reply,
};

type Result<T> = std::result::Result<T, Rejection>;

pub async fn handle_message(msg: Message) {
    let msg = match msg.to_str() {
        Ok(m) => m,
        Err(e) => {
            println!("error while hadling message: {:?}", e);
            return;
        }
    };
    println!("recevied message: {}", msg);
}

pub async fn client_connection(ws: WebSocket) {
    println!("establishing client connection... {:?}", ws);

    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    let client_rcv = UnboundedReceiverStream::new(client_rcv);

    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|res| {
        if let Err(e) = res {
            println!("error sending websocket msg: {}", e);
        }
    }));

    while let Some(res) = client_ws_rcv.next().await {
        let msg = match res {
            Ok(msg) => msg,
            Err(e) => {
                println!("error receving message: {}", e);
                break;
            }
        };
        let _ = client_sender.send(Ok(Message::text("message receved")));
        handle_message(msg).await;
    }
}

pub async fn ws_handler(ws: warp::ws::Ws) -> Result<impl Reply> {
    println!("Hadling WebSocket...");
    Ok(ws.on_upgrade(move |socket| client_connection(socket)))
}

#[tokio::main]
async fn main() {
    println!("Configuring websocket route");
    let ws_route = warp::any().and(warp::ws()).and_then(ws_handler);
    let routes = ws_route.with(warp::cors().allow_any_origin());
    println!("Starting server");
    warp::serve(routes)
        .tls()
        .cert_path("cert.crt")
        .key_path("key.rsa")
        .run(([0, 0, 0, 0], 9999))
        .await;
}
