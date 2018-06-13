#[macro_use] extern crate failure;
extern crate futures;
extern crate hyper;
#[macro_use] extern crate lazy_static;
extern crate rand;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_urlencoded;
#[macro_use] extern crate slog;
#[macro_use] extern crate log;
extern crate slog_async;
extern crate slog_scope;
extern crate slog_stdlog;
extern crate slog_term;
extern crate tacho;
extern crate time;
extern crate tokio;
extern crate tokio_ping;
extern crate trust_dns_resolver;
extern crate tokio_signal;

use futures::{Future, Stream};
use futures::sync::oneshot;
use futures::future::Either;
use slog::Drain;
use tokio_signal::unix::{Signal, SIGTERM, SIGINT};

mod http;
mod metrics;
mod pinger;
mod resolver;
mod settings;
mod utils;

fn signals() -> impl Future<Item=i32, Error=::std::io::Error> {
    futures::future::select_all((&[SIGTERM, SIGINT]).iter().map(|&signum| {
        Signal::new(signum).flatten_stream().into_future()
            .map(|(signum, _rest)| signum)
            .map_err(|(err, _rest)| err)
    })).then(|res| {
        match res {
            Ok((Some(signum), _, _)) => Ok(signum),
            Err((err, _, _)) => Err(err),
            Ok((None, _, _)) => unreachable!("signals")
        }
    })
}

fn run() -> i32 {
    let decorator = ::slog_term::PlainDecorator::new(::std::io::stdout());
    let drain = ::slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = ::slog_async::Async::new(drain).chan_size(4096).build().fuse();
    let log = slog::Logger::root(drain, o!());
    let _scope_guard = slog_scope::set_global_logger(log.new(o!()));
    slog_stdlog::init().expect("Init std logger");

    info!(concat!("Starting ", env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION")));

    let settings = match settings::Settings::from_env() {
        Ok(settings) => settings,
        Err(err) => {
            error!("{}", err);
            return 1
        }
    };

    info!("Using settings: {}", settings);

    http::init();
    metrics::init();

    let mut runtime = tokio::runtime::Runtime::new().expect("Tokio runtime");
    let (stop_sender, stop_receiver) = oneshot::channel();

    runtime.spawn(futures::lazy(move || {

        let server_future = pinger::Pinger::new().map_err(|_| {
            error!("Unable to create pinger, please check capabilities");
        }).and_then(move |pinger| {
            http::server(settings, pinger)
        });

        let signals_future = signals().map_err(|_| {
            error!("Signal handling error");
        });

        signals_future.select2(server_future).then(move |res| {
            if let Ok(Either::A((signum, _))) = res {
                info!("Got signal {}.", signum)
            }
            stop_sender.send(()).ok();
            Ok(())
        })
    }));

    stop_receiver.wait().ok();
    runtime.shutdown_now().wait().ok();
    info!("Exiting");
    0
}

fn main() {
    ::std::process::exit(run())
}
