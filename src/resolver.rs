use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use futures::{future, Future};
use rand::{seq::SliceRandom, thread_rng};
use trust_dns_resolver::config::{self, NameServerConfig, ResolverConfig, ResolverOpts};
use trust_dns_resolver::error::{ResolveError, ResolveErrorKind};
use trust_dns_resolver::AsyncResolver;

use settings::Settings;
use utils::{boxed, NameOrIpAddr, Protocol};

pub struct Resolver {
    inner: AsyncResolver,
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
        if let ResolveErrorKind::NoRecordsFound { .. } = err.kind() {
            Error::NotFound
        } else {
            Error::Error
        }
    }
}

impl Resolver {
    pub fn new(settings: Settings) -> impl Future<Item = Self, Error = Error> {
        let future = future::result(match settings.resolver {
            Some(resolver_addr) => {
                let mut config = ResolverConfig::new();
                config.add_name_server(NameServerConfig {
                    socket_addr: SocketAddr::new(resolver_addr, 53),
                    protocol: config::Protocol::Udp,
                    tls_dns_name: None,
                });
                let (client, future) = AsyncResolver::new(config, ResolverOpts::default());
                tokio::spawn(future);
                Ok(client)
            }
            None => AsyncResolver::from_system_conf().map(|(client, future)| {
                tokio::spawn(future);
                client
            }),
        });

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
                            .ipv4_lookup(name.as_ref().clone())
                            .map_err(From::from)
                            .and_then(|addrs| {
                                let addrs: Vec<Ipv4Addr> = addrs.iter().cloned().collect();

                                let random_addr =
                                    addrs.choose(&mut thread_rng()).ok_or(Error::NotFound)?;
                                Ok(IpAddr::from(random_addr.clone()))
                            }),
                    ),
                    Protocol::V6 => boxed(
                        self.inner
                            .ipv6_lookup(name.as_ref().clone())
                            .map_err(From::from)
                            .and_then(|addrs| {
                                let addrs: Vec<Ipv6Addr> = addrs.iter().cloned().collect();
                                let random_addr =
                                    addrs.choose(&mut thread_rng()).ok_or(Error::NotFound)?;

                                Ok(IpAddr::from(random_addr.clone()))
                            }),
                    ),
                };

                boxed(future.and_then(move |addr| Ok((::time::precise_time_ns() - st, addr))))
            }
        }
    }
}
