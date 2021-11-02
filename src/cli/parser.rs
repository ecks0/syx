use crate::{Error, Result};
use crate::cli::{app, env};
use std::str::FromStr;

// Argument parsing helper.
#[derive(Clone, Debug)]
pub struct Parser<'a>(clap::ArgMatches<'a>);

impl<'a> Parser<'a> {
    pub fn new(argv: &[String]) -> Result<Self> {
        let m = app::build().get_matches_from_safe(argv)?;
        Ok(Self(m))
    }

    // Return true if the given argument is present in argv. (Env vars not checked).
    pub fn arg_present(&self, name: &str) -> bool { self.0.is_present(name) }

    // Return the values for an argument from argv. (Env vars not checked).
    pub fn arg_values(&self, name: &str) -> Option<clap::Values> { self.0.values_of(name) }

    // Parse a flag argument from the argv or from env vars if present.
    pub fn flag(&self, name: &str) -> Option<()> {
        match self.0.is_present(name) {
            true => Some(()),
            false =>
                match env::var(name)
                    .map(|v| !v.is_empty() && v != "0" && v.to_lowercase() != "false")
                    .unwrap_or(false)
                {
                    true => Some(()),
                    false => None,
                },
        }
    }

    // Parse an integer argument from the argv or from env vars.
    pub fn int<T: FromStr<Err = std::num::ParseIntError>>(&self, name: &str) -> Result<Option<T>> {
        match self.0.value_of(name)
            .map(|v| v.to_string())
            .or_else(|| env::var(name))
        {
            Some(v) => Ok(Some(
                T::from_str(&v)
                    .map_err(|_| Error::parse_flag(name, "Expected integer value".into()))?
            )),
            None => Ok(None),
        }
    }

    // Parse a string argument from the argv or from env vars.
    pub fn str(&self, name: &str) -> Option<String> {
        self.0.value_of(name)
            .map(|v| v.to_string())
            .or_else(|| env::var(name))
    }

    // Parse an argument using `FromStr` from the argv or from env vars.
    pub fn from_str<S>(&self, name: &str) -> Result<Option<S>>
    where
        S: FromStr<Err = Error>,
    {
        match self.0.value_of(name)
            .map(String::from)
            .or_else(|| env::var(name))
        {
            Some(v) => Ok(Some(
                S::from_str(&v)
                    .map_err(|e| Error::parse_flag(name, e.to_string()))?
            )),
            None => Ok(None),
        }
    }

    // Parse an argument using `FromStr` from the argv or from env vars
    // and convert to the given type.
    pub fn from_str_as<S, T>(&self, name: &str) -> Result<Option<T>>
    where
        S: FromStr<Err = Error>,
        T: From<S>,
    {
        Ok(self.from_str::<S>(name)?.map(|v| T::from(v)))
    }
}
