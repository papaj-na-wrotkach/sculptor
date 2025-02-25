use uuid::Uuid;

use super::MessageLoadError;
use std::convert::{TryFrom, TryInto};

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum C2SMessage {
    Token(Vec<u8>) = 0,
    Ping(u32, bool, Vec<u8>) = 1,
    Sub(Uuid) = 2, // owo
    Unsub(Uuid) = 3,
}
// 6 - 6
impl TryFrom<&[u8]> for C2SMessage {
    type Error = MessageLoadError;
    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        if buf.is_empty() {
            Err(MessageLoadError::BadLength("C2SMessage", 1, false, 0))
        } else {
            match buf[0] {
                0 => Ok(C2SMessage::Token(buf[1..].to_vec())),
                1 => {
                    if buf.len() >= 6 {
                        Ok(C2SMessage::Ping(
                            u32::from_be_bytes((&buf[1..5]).try_into().unwrap()),
                            buf[5] != 0,
                            buf[6..].to_vec(),
                        ))
                    } else {
                        Err(MessageLoadError::BadLength(
                            "C2SMessage::Ping",
                            6,
                            false,
                            buf.len(),
                        ))
                    }
                }
                2 => {
                    if buf.len() == 17 {
                        Ok(C2SMessage::Sub(Uuid::from_bytes(
                            (&buf[1..]).try_into().unwrap(),
                        )))
                    } else {
                        Err(MessageLoadError::BadLength(
                            "C2SMessage::Sub",
                            17,
                            true,
                            buf.len(),
                        ))
                    }
                }
                3 => {
                    if buf.len() == 17 {
                        Ok(C2SMessage::Unsub(Uuid::from_bytes(
                            (&buf[1..]).try_into().unwrap(),
                        )))
                    } else {
                        Err(MessageLoadError::BadLength(
                            "C2SMessage::Unsub",
                            17,
                            true,
                            buf.len(),
                        ))
                    }
                }
                a => Err(MessageLoadError::BadEnum(
                    "C2SMessage.type",
                    0..=3,
                    a.into(),
                )),
            }
        }
    }
}
impl From<C2SMessage> for Vec<u8> {
    fn from(val: C2SMessage) -> Self {
        use std::iter;
        let a: Vec<u8> = match val {
            C2SMessage::Token(t) => iter::once(0).chain(t.iter().copied()).collect(),
            C2SMessage::Ping(p, s, d) => iter::once(1)
                .chain(p.to_be_bytes())
                .chain(iter::once(s.into()))
                .chain(d.iter().copied())
                .collect(),
            C2SMessage::Sub(s) => iter::once(2).chain(s.into_bytes()).collect(),
            C2SMessage::Unsub(s) => iter::once(3).chain(s.into_bytes()).collect(),
        };
        a
    }
}
impl C2SMessage {
    pub fn name(&self) -> &'static str {
        match self {
            C2SMessage::Token(_) => "c2s>token",
            C2SMessage::Ping(_, _, _) => "c2s>ping",
            C2SMessage::Sub(_) => "c2s>sub",
            C2SMessage::Unsub(_) => "c2s>unsub",
        }
    }
}

// impl<'a> C2SMessage<'a> {
//     pub fn to_array(&self) -> Box<[u8]> {
//         <C2SMessage as Into<Box<[u8]>>>::into(self.clone())
//     }
//     pub fn to_vec(&self) -> Vec<u8> {
//         self.to_array().to_vec()
//     }
// }