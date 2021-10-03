mod cpu;
mod drm;
mod intel_pstate;

fn dot() -> String { "•".to_string() }

pub fn print() {
    let mut s = vec![];
    if let Some(ss) = cpu::format() { s.push(ss); }
    if let Some(ss) = intel_pstate::format() { s.push(ss); }
    if let Some(ss) = drm::format() { s.push(ss); }
    if !s.is_empty() {
        println!("{}", s.join("\n"));
    }
}
