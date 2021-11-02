use crate::{CardId, Chain, Knobs, Result};
use crate::cli::{ARG_CHAIN, NAME, lazy, parser::Parser};

// Parse the knobs call chain and resolve resource ids if needed.
pub async fn resolve_chain(mut c: Chain) -> Result<Chain> {
    for k in c.iter_mut() {
        if k.has_cpu_related_values() && k.cpu.is_none() {
            k.cpu = lazy::cpu_ids().await;
        }
        if k.has_drm_i915_values() && k.drm_i915.is_none() {
            k.drm_i915 = lazy::drm_i915_ids().await
                .map(|ids| ids
                    .into_iter()
                    .map(CardId::Id)
                    .collect());
        }
        #[cfg(feature = "nvml")]
        if k.has_nvml_values() && k.nvml.is_none() {
            k.nvml = lazy::nvml_ids().await
                .map(|ids| ids
                    .into_iter()
                    .map(CardId::Id)
                    .collect());
        }
    }
    Ok(c)
}

// Parse and return the knobs call chain.
fn parse_chain(first: Parser) -> Result<Chain> {
    let mut chain: Vec<Knobs> = vec![];
    let mut p = first;
    loop {
        chain.push(Knobs::try_from(p.clone())?);
        if !p.arg_present(ARG_CHAIN) { break; }
        match p.arg_values(ARG_CHAIN) {
            Some(v) => {
                let mut v: Vec<String> = v.map(String::from).collect();
                if v.is_empty() { break; }
                v.insert(0, NAME.to_string());
                p = Parser::new(&v)?;
            },
            None => break,
        };
    }
    Ok(chain.into())
}

// Parse the knobs call chain and resolve resource ids if needed.
pub async fn resolve_parser(p: Parser<'_>) -> Result<Chain> {
    let c = parse_chain(p)?;
    resolve_chain(c).await
}
