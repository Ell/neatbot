use anyhow::{Error, Result};
use std::str::FromStr;
#[derive(Debug, PartialEq, Clone)]
pub enum ChannelMode {
    ColourFilter,
    BlockCTCP,
    FreeInvite,
    JoinThrottle,
    Password,
    JoinLimit,
    Moderated,
    Private,
    Quiet,
    BlockForwardedUsers,
    Secret,
    TLSOnly,
    Unfiltered,
    ReducedModeration,
    Unknown(String),
}

impl FromStr for ChannelMode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "c" => Ok(Self::ColourFilter),
            "C" => Ok(Self::BlockCTCP),
            "g" => Ok(Self::FreeInvite),
            "j" => Ok(Self::JoinThrottle),
            "k" => Ok(Self::Password),
            "l" => Ok(Self::JoinLimit),
            "m" => Ok(Self::Moderated),
            "p" => Ok(Self::Private),
            "q" => Ok(Self::Quiet),
            "Q" => Ok(Self::BlockForwardedUsers),
            "s" => Ok(Self::Secret),
            "S" => Ok(Self::TLSOnly),
            "u" => Ok(Self::Unfiltered),
            "z" => Ok(Self::ReducedModeration),
            _ => Ok(Self::Unknown(s.to_string())),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum AccessLevel {
    User,
    Voice,
    HalfOp,
    Op,
    SuperOp,
    Owner,
}

impl FromStr for AccessLevel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "~" => Ok(Self::Owner),
            "&" => Ok(Self::SuperOp),
            "@" => Ok(Self::Op),
            "%" => Ok(Self::HalfOp),
            "+" => Ok(Self::Voice),
            _ => Ok(Self::User),
        }
    }
}

#[derive(Debug, Clone)]
pub struct User {
    nick: Option<String>,
    hostmask: Option<String>,
    access: Option<AccessLevel>,
}

#[derive(Debug, Clone)]
pub struct Channel {
    name: String,
    users: Vec<User>,
    topic: String,
    modes: Vec<ChannelMode>,
}
