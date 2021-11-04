use crate::NAME;

pub fn var_name(n: &str) -> String { format!("{}_{}", NAME, n) }

pub fn var(n: &str) -> Option<String> { std::env::var(&var_name(n)).ok() } // FIXME handle result

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
