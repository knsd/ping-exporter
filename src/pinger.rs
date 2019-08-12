use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use futures::future::{ok, Either};
use futures::{future, Future, Stream};
use tokio::timer::Timeout;
use tokio_ping::{Error as PingError, Pinger as LowLevelPinger};

use resolver::{Error as ResolveError, Resolver};
use settings::Settings;
use utils::{NameOrIpAddr, Protocol};

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "ping error")]
    PingError { error: PingError },
    #[fail(display = "create resolver error")]
    CreateResolverError { error: ResolveError },
}

impl From<PingError> for Error {
    fn from(error: PingError) -> Self {
        Error::PingError { error }
    }
}

impl From<ResolveError> for Error {
    fn from(error: ResolveError) -> Self {
        Error::CreateResolverError { error }
    }
}

pub enum Report {
    ResolveNotFound,
    ResolveTimedOut,
    ResolveOtherError,
    Success {
        resolve_time_ns: u64,
        addr: IpAddr,
        pings: Vec<Option<f64>>,
    },
}

#[derive(Clone)]
pub struct Pinger {
    inner: Arc<PingerInner>,
}

struct PingerInner {
    resolver: Resolver,
    pinger: LowLevelPinger,
}

impl Pinger {
    pub fn new(settings: Settings) -> impl Future<Item = Self, Error = Error> {
        let resolver_future = Resolver::new(settings).map_err(From::from);
        let pinger_future = LowLevelPinger::new().map_err(From::from);
        resolver_future
            .join(pinger_future)
            .and_then(|(resolver, pinger)| {
                Ok(Self {
                    inner: Arc::new(PingerInner { resolver, pinger }),
                })
            })
    }

    pub fn ping(
        &self,
        name: NameOrIpAddr,
        protocol: Protocol,
        count: usize,
        resolve_timeout: u64,
        timeout: u64,
    ) -> impl Future<Item = Report, Error = Error> {
        let resolve_timeout = Duration::from_millis(resolve_timeout);
        let timeout = Duration::from_millis(timeout);

        let future = self.inner.resolver.resolve(name, protocol);
        let future = Timeout::new(future, resolve_timeout);

        let pinger = self.inner.pinger.clone();
        let future = future.then(move |result| match result {
            Ok((resolve_time_ns, addr)) => {
                let stream = pinger
                    .chain(addr)
                    .timeout(timeout)
                    .stream()
                    .map_err(From::from)
                    .take(count as u64);

                let future = stream.fold(Vec::new(), |mut acc, result| {
                    acc.push(result);
                    future::ok::<Vec<Option<f64>>, Error>(acc)
                });

                Either::A(future.and_then(move |pings| {
                    Ok(Report::Success {
                        resolve_time_ns,
                        addr,
                        pings,
                    })
                }))
            }
            Err(err) => Either::B(ok(match err.into_inner() {
                Some(ResolveError::NotFound) => Report::ResolveNotFound,
                Some(ResolveError::Error) => Report::ResolveOtherError,
                None => Report::ResolveTimedOut,
            })),
        });

        future.and_then(|report| Ok(report))
        //
        //        let future = future.map_err(|err| match err.into_inner() {
        //            Some(err) => err.into(),
        //            None => Error::ResolveTimeoutError {}
        //        });
        //
        //        let pinger = self.inner.pinger.clone();
        //        let future = future.and_then(move |(resolve_time_ns, addr)| {
        //            let stream = pinger.chain(addr)
        //                .timeout(timeout)
        //                .stream()
        //                .map_err(From::from)
        //                .take(count as u64);
        //
        //            let future = stream.fold(Vec::new(), |mut acc, result| {
        //                acc.push(result);
        //                future::ok::<Vec<Option<f64>>, Error>(acc)
        //            });
        //
        //            future.and_then(move |times| {
        //                Ok(Report::Success {
        //                    resolve_time_ns,
        //                    addr,
        //                    times,
        //                })
        //            })
        //        });
    }
}
