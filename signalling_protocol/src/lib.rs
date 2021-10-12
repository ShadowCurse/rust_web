use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct SessionId(String);

impl SessionId {
    pub fn new(string: String) -> Self {
        SessionId(string)
    }
    pub fn value(&self) -> &String {
        &self.0
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct UserId(String);

impl UserId {
    pub fn new(string: String) -> Self {
        UserId(string)
    }
    pub fn value(&self) -> &String {
        &self.0
    }
}

#[derive(Serialize, Deserialize)]
pub enum Signal {
    NewUser(UserId),

    SessionNew,
    SessionCreated(SessionId),
    SessionJoin(SessionId),
    SessionJoinSuccess(SessionId),
    SessionJoinError(SessionId),

    VideoOffer(SessionId, String),
    VideoAnswer(SessionId, String),
    ICECandidate(SessionId, String),
    ICEError(SessionId, String),
}

impl std::fmt::Debug for Signal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NewUser(id) => write!(f, "NewUser: {:?}", id),
            Self::SessionNew => write!(f, "SessionNew"),
            Self::SessionCreated(id) => write!(f, "SessionCreated: {:?}", id),
            Self::SessionJoin(id) => write!(f, "SessionJoin: {:?}", id),
            Self::SessionJoinSuccess(id) => write!(f, "SessionJoinSuccess: {:?}", id),
            Self::SessionJoinError(id) => write!(f, "SessionJoinError: {:?}", id),
            Self::VideoOffer(id, _) => write!(f, "VideoOffer: {:?}", id),
            Self::VideoAnswer(id, _) => write!(f, "VideoAnswer: {:?}", id),
            Self::ICECandidate(id, _) => write!(f, "ICECandidate: {:?}", id),
            Self::ICEError(id, _) => write!(f, "ICEError: {:?}", id),
        }
    }
}
