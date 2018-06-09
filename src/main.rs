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

use futures::Future;

mod http;
mod logging;
mod metrics;
mod pinger;
mod resolver;
mod settings;
mod utils;

fn run() -> i32 {
    let log = logging::init();

    let _scope_guard = slog_scope::set_global_logger(log.new(o!()));
    slog_stdlog::init().expect("Init std logger");

    let settings = match settings::Settings::from_env() {
        Ok(settings) => settings,
        Err(err) => {
            error!("{}", err);
            return 1
        }
    };

    http::init();
    metrics::init();

    info!("Started");

    tokio::run(futures::lazy(move || {
        pinger::Pinger::new().map_err(|_| {
            error!("Unable to create pinger, please check capabilities");
        }).and_then(move |pinger| {
            http::server(settings, pinger)
        })
    }));

    0
}

fn main() {
    ::std::process::exit(run())
}
