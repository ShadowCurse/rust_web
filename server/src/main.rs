use futures::{FutureExt, StreamExt};
use std::{collections::HashMap, convert::Infallible, result::Result, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::{
    ws::{Message, WebSocket},
    Filter, Rejection, Reply,
};

use signalling_protocol::*;

pub struct User {
    sender: mpsc::UnboundedSender<Result<Message, warp::Error>>,
    session_id: Option<SessionId>,
    user_id: UserId,
}

#[derive(Debug)]
pub struct Session {
    host: UserId,
    guest: Option<UserId>,
}

type Users = Arc<Mutex<HashMap<UserId, User>>>;
type Sessions = Arc<Mutex<HashMap<SessionId, Session>>>;

pub async fn send_signal(user: &User, signal: Signal) -> Result<(), String> {
    println!("Sending to user: {:#?} signal: {:#?}", user.user_id, signal);
    let message = match serde_json::to_string(&signal) {
        Ok(msg) => msg,
        Err(_) => return Err(format!("can not serialize signal: {:?}", signal)),
    };

    match user.sender.send(Ok(Message::text(message))) {
        Ok(()) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

pub async fn handle_message(
    user_id: &UserId,
    msg: &Message,
    users: Users,
    sessions: Sessions,
) -> Result<(), String> {
    let msg = match msg.to_str() {
        Ok(m) => m,
        Err(_) => {
            return Err("message is not a str".to_string());
        }
    };

    let result: Signal = match serde_json::from_str(&msg) {
        Ok(x) => x,
        Err(e) => {
            return Err(e.to_string());
        }
    };

    println!("Handling signal: {:#?}", result);

    match result {
        Signal::SessionNew => {
            let new_session_id = SessionId::new(Uuid::new_v4().to_string());
            let new_session = Session {
                host: user_id.clone(),
                guest: None,
            };

            println!("Created new session: {:?}", new_session_id);

            sessions
                .lock()
                .await
                .insert(new_session_id.clone(), new_session);

            match users.lock().await.get_mut(user_id) {
                Some(user) => {
                    user.session_id = Some(new_session_id.clone());
                    let sig_msg = Signal::SessionCreated(new_session_id);
                    send_signal(&user, sig_msg).await?;
                }
                None => return Err(format!("can not find user {:?}", user_id)),
            }
        }
        Signal::SessionJoin(session_id) => match sessions.lock().await.get_mut(&session_id) {
            Some(session) => {
                session.guest = Some(user_id.clone());
                match users.lock().await.get_mut(user_id) {
                    Some(user) => {
                        let sig_msg = Signal::SessionJoinSuccess(session_id);
                        send_signal(&user, sig_msg).await?;
                    }
                    None => return Err(format!("can not find user {:?}", user_id)),
                }
            }
            None => match users.lock().await.get(user_id) {
                Some(user) => {
                    let sig_msg = Signal::SessionJoinError(session_id);
                    send_signal(&user, sig_msg).await?;
                }
                None => return Err(format!("can not find user {:?}", user_id)),
            },
        },
        Signal::VideoOffer(session_id, offer) => match sessions.lock().await.get(&session_id) {
            Some(session) => match users.lock().await.get(&session.host) {
                Some(host) => {
                    let sig_msg = Signal::VideoOffer(session_id, offer);
                    send_signal(&host, sig_msg).await?;
                }
                None => return Err(format!("can not find user {:?}", user_id)),
            },
            None => return Err(format!("can not find session {:?}", session_id)),
        },
        Signal::VideoAnswer(session_id, answer) => match sessions.lock().await.get(&session_id) {
            Some(session) => match users.lock().await.get(session.guest.as_ref().unwrap()) {
                Some(guest) => {
                    let sig_msg = Signal::VideoAnswer(session_id, answer);
                    send_signal(&guest, sig_msg).await?;
                }
                None => return Err(format!("can not find user {:?}", user_id)),
            },
            None => return Err(format!("can not find session {:?}", session_id)),
        },
        Signal::ICECandidate(session_id, candidate) => match sessions.lock().await.get(&session_id)
        {
            Some(session) => {
                println!(
                    "Got ICECandidate: user_id: {:#?}, session: {:#?}",
                    user_id, session
                );
                let destination = if *user_id == session.host {
                    session.guest.as_ref().unwrap().clone()
                } else {
                    session.host.clone()
                };
                match users.lock().await.get(&destination) {
                    Some(user) => {
                        let sig_msg = Signal::ICECandidate(session_id, candidate);
                        send_signal(&user, sig_msg).await?;
                    }
                    None => return Err(format!("can not find user {:?}", user_id)),
                }
            }
            None => return Err(format!("can not find session {:?}", session_id)),
        },
        _ => {}
    }

    Ok(())
}

pub async fn user_connection(ws: WebSocket, users: Users, sessions: Sessions) {
    println!("establishing client connection... {:?}", ws);

    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    let client_rcv = UnboundedReceiverStream::new(client_rcv);

    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|res| {
        if let Err(e) = res {
            println!("error sending websocket msg: {}", e);
        }
    }));

    let user_id = UserId::new(Uuid::new_v4().to_string());
    let user = User {
        sender: client_sender.clone(),
        session_id: None,
        user_id: user_id.clone(),
    };

    println!("Created new user: {:?}", user_id);
    users.lock().await.insert(user_id.clone(), user);

    let new_user_signal = Signal::NewUser(user_id.clone());
    let message = match serde_json::to_string(&new_user_signal) {
        Ok(msg) => msg,
        Err(e) => {
            println!("error serializing NewUser{:?}: {:?}", new_user_signal, e);
            return;
        }
    };

    match client_sender.send(Ok(Message::text(message))) {
        Err(e) => {
            println!("error sending NewUser{:?}: {:?}", new_user_signal, e);
        }
        _ => {}
    }

    while let Some(res) = client_ws_rcv.next().await {
        let msg = match res {
            Ok(msg) => msg,
            Err(e) => {
                println!("error receving message: {}", e);
                break;
            }
        };
        match handle_message(&user_id, &msg, users.clone(), sessions.clone()).await {
            Ok(()) => {
                println!("successfully hadnled message")
            }
            Err(e) => {
                println!("error hadnling message: {:?}, msg: {:?}", e, msg);
            }
        }
    }

    users.lock().await.remove(&user_id);
}

pub async fn ws_handler(
    ws: warp::ws::Ws,
    users: Users,
    sessions: Sessions,
) -> Result<impl Reply, Rejection> {
    println!("Hadling WebSocket...");
    Ok(ws.on_upgrade(move |socket| user_connection(socket, users, sessions)))
}

fn with_users(users: Users) -> impl Filter<Extract = (Users,), Error = Infallible> + Clone {
    warp::any().map(move || users.clone())
}

fn with_sessions(
    sessions: Sessions,
) -> impl Filter<Extract = (Sessions,), Error = Infallible> + Clone {
    warp::any().map(move || sessions.clone())
}

#[tokio::main]
async fn main() {
    let users: Users = Arc::new(Mutex::new(HashMap::new()));
    let sessions: Sessions = Arc::new(Mutex::new(HashMap::new()));

    println!("Configuring websocket route");
    let ws_route = warp::any()
        .and(warp::ws())
        .and(with_users(users.clone()))
        .and(with_sessions(sessions.clone()))
        .and_then(ws_handler);
    let routes = ws_route.with(warp::cors().allow_any_origin());
    println!("Starting server");
    warp::serve(routes)
        .tls()
        .cert_path("cert.crt")
        .key_path("key.rsa")
        .run(([0, 0, 0, 0], 9999))
        .await;
}
