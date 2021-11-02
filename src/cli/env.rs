const ENV_PREFIX: &str = "KNOBS";

// Return the environment variable name for the given cli argument name.
fn var_name(cli_name: &str) -> String {
    format!("{}_{}", ENV_PREFIX, cli_name.to_uppercase().replace("-", "_"))
}

// Return the environment variable value for the given cli argument name.
pub fn var(cli_name: &str) -> Option<String> {

    match std::env::var(&var_name(cli_name)) {
        Ok(v) => {
            log::debug!("--{}: using value from environment: {}", cli_name, v);
            Some(v)
        },
        _ => None,
    }
}

// Return the system's hostname.
pub fn hostname() -> Option<String> {
    let mut buf = [0u8; 64];
    match nix::unistd::gethostname(&mut buf) {
        Ok(h) => match h.to_str() {
            Ok(h) => Some(h.to_string()),
            Err(_) => {
                log::error!("ERR nix r UTF8 hostname() utf8 conversion failed");
                None
            }
        },
        Err(e) => {
            log::error!("ERR nix r {} hostname() failed to determine system hostname", e);
            None
        },
    }
}
