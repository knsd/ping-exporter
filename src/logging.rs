use slog;
use slog::Drain;

use std::sync::atomic;
use std::sync::atomic::Ordering;
use std::result;

pub static DEBUG: atomic::AtomicBool = atomic::ATOMIC_BOOL_INIT;

struct RuntimeLevelFilter<D>{
    drain: D,
}

impl<D> Drain for RuntimeLevelFilter<D>
    where D : Drain {
    type Ok = Option<D::Ok>;
    type Err = Option<D::Err>;

    fn log(&self,
           record: &slog::Record,
           values: &slog::OwnedKVList)
           -> result::Result<Self::Ok, Self::Err> {
        let current_level = if DEBUG.load(Ordering::Relaxed) {
            slog::Level::Debug
        } else {
            slog::Level::Info
        };

        if record.level().is_at_least(current_level) {
            self.drain.log(
                record,
                values
            )
                .map(Some)
                .map_err(Some)
        } else {
            Ok(None)
        }
    }
}

pub fn init() -> slog::Logger {
    let decorator = ::slog_term::PlainDecorator::new(::std::io::stdout());
    let drain = ::slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = RuntimeLevelFilter { drain: drain }.fuse();
    let drain = ::slog_async::Async::new(drain).chan_size(4096).build().fuse();
    slog::Logger::root(drain, o!())
}
