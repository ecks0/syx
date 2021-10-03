mod cpu;
mod drm;
mod intel_pstate;

fn dot() -> String { "•".to_string() }

pub fn print() {
    cpu::print();
    intel_pstate::print();
    drm::print();
}
