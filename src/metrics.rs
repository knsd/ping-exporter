use tacho::{self, Reporter, Scope};

lazy_static! {
    static ref __T: (Scope, Reporter) = tacho::new();
    pub static ref METRICS: Scope = __T.0.clone();
    pub static ref REPORTER: Reporter = __T.1.clone();
}

pub fn init() {
    ::lazy_static::initialize(&METRICS);
    ::lazy_static::initialize(&REPORTER);
}

#[cfg(test)]
mod tests {
    use super::init;

    #[test]
    fn test_lazy_static() {
        init()
    }
}
