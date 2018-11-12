use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use futures::{future, Future};
use rand::{thread_rng, Rng};
use trust_dns_resolver::error::{ResolveError, ResolveErrorKind};
use trust_dns_resolver::ResolverFuture;

use utils::{boxed, NameOrIpAddr, Protocol};

pub struct Resolver {
    inner: ResolverFuture,
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "not found")]
    NotFound,
    #[fail(display = "unknown error")]
    Error,
}

impl From<ResolveError> for Error {
    fn from(err: ResolveError) -> Self {
        if let ResolveErrorKind::NoRecordsFound(_) = err.kind() {
            Error::NotFound
        } else {
            Error::Error
        }
    }
}

impl Resolver {
    pub fn new() -> impl Future<Item = Self, Error = Error> {
        let future = future::result(ResolverFuture::from_system_conf()).flatten();

        future
            .map_err(From::from)
            .and_then(|inner| Ok(Resolver { inner }))
    }

    pub fn resolve(
        &self,
        name: NameOrIpAddr,
        protocol: Protocol,
    ) -> impl Future<Item = (u64, IpAddr), Error = Error> {
        let st = ::time::precise_time_ns();
        match name {
            NameOrIpAddr::IpAddr(addr) => boxed(future::ok((0, addr))),
            NameOrIpAddr::Name(name) => {
                let future = match protocol {
                    Protocol::V4 => boxed(
                        self.inner
                            .ipv4_lookup(name.as_ref())
                            .map_err(From::from)
                            .and_then(|addrs| {
                                let addrs: Vec<Ipv4Addr> = addrs.iter().cloned().collect();
                                let random_addr =
                                    thread_rng().choose(&addrs).ok_or(Error::NotFound)?;
                                Ok(IpAddr::from(random_addr.clone()))
                            }),
                    ),
                    Protocol::V6 => boxed(
                        self.inner
                            .ipv6_lookup(name.as_ref())
                            .map_err(From::from)
                            .and_then(|addrs| {
                                let addrs: Vec<Ipv6Addr> = addrs.iter().cloned().collect();
                                let random_addr =
                                    thread_rng().choose(&addrs).ok_or(Error::NotFound)?;

                                Ok(IpAddr::from(random_addr.clone()))
                            }),
                    ),
                };

                boxed(future.and_then(move |addr| Ok((::time::precise_time_ns() - st, addr))))
            }
        }
    }
}
