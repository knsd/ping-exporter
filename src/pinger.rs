use std::net::IpAddr;
use std::time::{Duration, Instant};
use std::sync::Arc;

use futures::{Future, Stream, future};
use tokio::timer::Deadline;
use tokio_ping::{Pinger as LowLevelPinger, Error as PingError};

use resolver::{Resolver, Error as ResolveError};
use utils::{Protocol, NameOrIpAddr};


#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "ping error")]
    PingError {
        error: PingError
    },
    #[fail(display = "resolve error")]
    ResolveError {
        error: ResolveError
    },
    #[fail(display = "resolve timeout error")]
    ResolveTimeoutError {
    }
}

impl From<PingError> for Error {
    fn from(error: PingError) -> Self {
        Error::PingError {
            error
        }
    }
}

impl From<ResolveError> for Error {
    fn from(error: ResolveError) -> Self {
        Error::ResolveError {
            error
        }
    }
}

#[derive(Clone)]
pub struct Pinger {
    inner: Arc<PingerInner>
}

struct PingerInner {
    resolver: Resolver,
    pinger: LowLevelPinger,
}

impl Pinger {
    pub fn new() -> impl Future<Item=Self, Error=Error> {
        let resolver_future = Resolver::new().map_err(From::from);
        let pinger_future = LowLevelPinger::new().map_err(From::from);
        resolver_future.join(pinger_future).and_then(|(resolver, pinger)| Ok(Self {
            inner: Arc::new(PingerInner {
                resolver,
                pinger,
            })
        }))
    }

    pub fn ping(&self, name: NameOrIpAddr, protocol: Protocol, count: usize, resolve_timeout: u64, timeout: u64) -> impl Future<Item=(u64, IpAddr, Vec<Option<f64>>), Error=Error> {
        let resolve_timeout = Duration::from_millis(resolve_timeout);
        let timeout = Duration::from_millis(timeout);

        let future = self.inner.resolver.resolve(name, protocol);
        let future = Deadline::new(future, Instant::now() + resolve_timeout);
        let future = future.map_err(|err| match err.into_inner() {
            Some(err) => err.into(),
            None => Error::ResolveTimeoutError {}
        });

        let pinger = self.inner.pinger.clone();
        let future = future.and_then(move |(resolve_time_ns, addr)| {
            let stream = pinger.chain(addr)
                .timeout(timeout)
                .stream()
                .map_err(From::from)
                .take(count as u64);

            let future = stream.fold(Vec::new(), |mut acc, result| {
                acc.push(result);
                future::ok::<Vec<Option<f64>>, Error>(acc)
            });

            future.and_then(move |times| {
                Ok((resolve_time_ns, addr, times))
            })
        });

        future
    }
}
