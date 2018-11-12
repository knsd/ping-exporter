use std::env;
use std::fmt;
use std::net::SocketAddr;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use utils::Protocol;

static ENV_PREFIX: &str = "PING_EXPORTER";
static ENV_SEPARATOR: &str = "_";

lazy_static! {
    static ref DEFAULT_LISTEN: SocketAddr =
        SocketAddr::from_str("[::]:9346").expect("DEFAULT_LISTEN");
}

#[derive(Debug, Clone)]
pub struct Settings {
    inner: Arc<SettingsInner>,
}

impl fmt::Display for Settings {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "listen address: {}, ", self.listen)?;
        write!(f, "preferred protocol: {}, ", self.protocol)?;
        write!(f, "default number of ICMP packets: {}, ", self.count)?;
        write!(f, "maximum number of ICMP packets: {}, ", self.max_count)?;
        write!(
            f,
            "timeout for each ICMP packet: {} ms, ",
            self.ping_timeout
        )?;
        write!(
            f,
            "maximum timeout for each ICMP packet: {} ms, ",
            self.max_ping_timeout
        )?;
        write!(f, "resolve timeout: {} ms, ", self.resolve_timeout)?;
        write!(
            f,
            "maximum resolve timeout: {} ms.",
            self.max_resolve_timeout
        )?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct SettingsInner {
    pub listen: SocketAddr,
    pub protocol: Protocol,
    pub count: usize,
    pub max_count: usize,
    pub ping_timeout: u64,
    pub max_ping_timeout: u64,
    pub resolve_timeout: u64,
    pub max_resolve_timeout: u64,
}

impl Settings {
    pub fn from_env() -> Result<Self, Error> {
        Ok(Self {
            inner: Arc::new(SettingsInner {
                listen: get_env_or("LISTEN", DEFAULT_LISTEN.clone())?,
                protocol: get_env_or("DEFAULT_PROTOCOL", Protocol::V4)?,
                count: get_env_or("DEFAULT_COUNT", 5)?,
                max_count: get_env_or("MAX_COUNT", 30)?,
                ping_timeout: get_env_or("DEFAULT_PING_TIMEOUT", 1000)?,
                max_ping_timeout: get_env_or("MAX_PING_TIMEOUT", 10000)?,
                resolve_timeout: get_env_or("DEFAULT_RESOLVE_TIMEOUT", 1000)?,
                max_resolve_timeout: get_env_or("MAX_RESOLVE_TIMEOUT", 10000)?,
            }),
        })
    }
}

impl Deref for Settings {
    type Target = SettingsInner;

    fn deref(&self) -> &<Self as Deref>::Target {
        &self.inner
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "missing environment variable: {}", name)]
    MissingEnvVar { name: String },
    #[fail(display = "invalid environment variable type: {}", name)]
    InvalidVariableType { name: String },
}

fn get_env_or<T: FromStr>(name: &str, default: T) -> Result<T, Error> {
    match get_env_(name) {
        Ok(v) => Ok(v),
        Err(Error::MissingEnvVar { .. }) => Ok(default),
        Err(err) => Err(err),
    }
}

fn get_env_<T: FromStr>(name: &str) -> Result<T, Error> {
    let env_var_name = format!("{}{}{}", ENV_PREFIX, ENV_SEPARATOR, name.to_uppercase());

    let string = env::var(&env_var_name).map_err(|_| Error::MissingEnvVar {
        name: env_var_name.clone(),
    })?;
    T::from_str(&string).map_err(|_| Error::InvalidVariableType { name: env_var_name })
}

#[cfg(test)]
mod tests {
    use super::Settings;

    #[test]
    fn test_valid_default_settings() {
        assert!(Settings::from_env().is_ok());
    }
}
