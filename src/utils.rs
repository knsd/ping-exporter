use std::fmt;
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;

use futures::Future;
use serde::{Deserialize, Deserializer, de::Error as SerdeDeError};
use trust_dns_resolver::Name;

#[derive(Debug, Clone, Copy)]
pub enum Protocol {
    V4,
    V6,
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Protocol::V4 => write!(f, "v4"),
            &Protocol::V6 => write!(f, "v6"),
        }
    }
}

impl FromStr for Protocol {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        match s {
            "v4" => Ok(Protocol::V4),
            "v6" => Ok(Protocol::V6),
            other => Err(format!("'{}' is not valid protocol, use v4 or v6", other)),
        }

    }
}

impl<'de> Deserialize<'de> for Protocol {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        Protocol::from_str(String::deserialize(deserializer)?.as_str())
            .map_err(D::Error::custom)
    }
}

#[derive(Debug, Clone)]
pub enum NameOrIpAddr {
    Name(Arc<Name>),
    IpAddr(IpAddr),
}

impl fmt::Display for NameOrIpAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &NameOrIpAddr::Name(ref name) => name.fmt(f),
            &NameOrIpAddr::IpAddr(ref addr) => addr.fmt(f),
        }
    }
}

impl FromStr for NameOrIpAddr {
    type Err = <Name as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        IpAddr::from_str(s).map(|addr| NameOrIpAddr::IpAddr(addr))
            .or_else(|_| Ok(NameOrIpAddr::Name(Arc::new(Name::from_str(s)?))))
    }
}

impl<'de> Deserialize<'de> for NameOrIpAddr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        NameOrIpAddr::from_str(String::deserialize(deserializer)?.as_str())
            .map_err(|_|D::Error::custom("Invalid hostname or IP addr"))
    }
}

pub fn boxed<F: Future<Item=I, Error=E> + Send + 'static, I, E>(future: F) -> Box<Future<Item=I, Error=E> + Send> {
    Box::new(future)
}
