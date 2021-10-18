use clap::{App, AppSettings, Arg, crate_version};
use crate::Result;
use super::{Cli, logging, parse};

fn argv0(argv: &[String]) -> &str {
    let default = "knobs";
    argv
        .first()
        .map(|s| s.as_str())
        .unwrap_or(default)
        .split('/')
        .last()
        .unwrap_or(default)
}

const HELP_ENV: &str = r#"ENVS:
        KNOBS_LOG=<error|warn|info|debug|trace>    Log level, default error
"#;

pub fn parse(argv: &[String]) -> Result<Cli> {
    logging::configure();
    let m = App::new(argv0(argv))
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::DisableVersion)
        .setting(AppSettings::UnifiedHelpMessage)
        .version(crate_version!())

        .after_help(HELP_ENV)

        .arg(Arg::with_name("show-cpu")
            .long("show-cpu")
            .takes_value(false)
            .help("Print cpu and cpufreq values"))

        .arg(Arg::with_name("show-pstate")
            .long("show-pstate")
            .takes_value(false)
            .help("Print intel_pstate values"))

        .arg(Arg::with_name("show-drm")
            .long("show-drm")
            .takes_value(false)
            .help("Print drm values"))

        .arg(Arg::with_name("show-nvml")
            .long("show-nvml")
            .takes_value(false)
            .help("Print nvidia management values"))

        .arg(Arg::with_name("cpu")
            .short("c")
            .long("cpu")
            .takes_value(true)
            .value_name("INDICES")
            .help("Target cpu ids, default all, ex. 0,1,3-5"))

        .arg(Arg::with_name("cpu-on")
            .short("o")
            .long("cpu-on")
            .takes_value(true)
            .value_name("0|1")
            .help("Set cpu online status per --cpu"))

        .arg(Arg::with_name("cpu-on-each")
            .short("O")
            .long("cpu-on-each")
            .takes_value(true)
            .value_name("[0|1|-]+")
            .help("Set cpu online status, ex. 10-1 → 0=ON 1=OFF 2=SKIP 3=ON"))

        .arg(Arg::with_name("cpufreq-gov")
            .short("g")
            .long("cpufreq-gov")
            .takes_value(true)
            .value_name("NAME")
            .help("Set cpufreq governor per --cpu"))

        .arg(Arg::with_name("cpufreq-min")
            .short("n")
            .long("cpufreq-min")
            .takes_value(true)
            .value_name("HZ")
            .help("Set cpufreq min freq per --cpu, ex. 1200, 1200mhz, 1.2ghz"))

        .arg(Arg::with_name("cpufreq-max")
            .short("x")
            .long("cpufreq-max")
            .takes_value(true)
            .value_name("HZ")
            .help("Set cpufreq max freq per --cpu, ex. 1200, 1200mhz, 1.2ghz"))

        .arg(Arg::with_name("pstate-epb")
            .long("pstate-epb")
            .takes_value(true)
            .value_name("0-15")
            .help("Set intel_pstate energy/performance bias per --cpu"))

        .arg(Arg::with_name("pstate-epp")
            .long("pstate-epp")
            .takes_value(true)
            .value_name("NAME")
            .help("Set intel_pstate energy/performance pref per --cpu"))

        .arg(Arg::with_name("drm-i915")
            .long("drm-i915")
            .takes_value(true)
            .value_name("INDICES")
            .help("Target i915 card ids, default all, ex. 0,1,3-5"))

        .arg(Arg::with_name("drm-i915-min")
            .long("drm-i915-min")
            .takes_value(true)
            .value_name("HZ")
            .help("Set i915 min frequency per --drm-i915, ex. 1200, 1200mhz, 1.2ghz"))

        .arg(Arg::with_name("drm-i915-max")
            .long("drm-i915-max")
            .takes_value(true)
            .value_name("HZ")
            .help("Set i915 max frequency per --drm-i915, ex. 1200, 1200mhz, 1.2ghz"))

        .arg(Arg::with_name("drm-i915-boost")
            .long("drm-i915-boost")
            .takes_value(true)
            .value_name("HZ")
            .help("Set i915 boost frequency per --drm-i915, ex. 1200, 1200mhz, 1.2ghz"))

        .arg(Arg::with_name("nvml")
            .long("nvml")
            .takes_value(true)
            .value_name("INDICES")
            .help("Target nvidia gpu ids, default all, ex. 0,1,3-5"))

        .arg(Arg::with_name("nvml-gpu-clock")
            .long("nvml-gpu-clock")
            .takes_value(true)
            .value_name("HZ|HZ,HZ")
            .conflicts_with("nvml-gpu-clock-reset")
            .help("Set nvidia gpu min,max frequency per --nvml, ex. 1200mhz or 900mhz,1.4ghz"))

        .arg(Arg::with_name("nvml-gpu-clock-reset")
            .long("nvml-gpu-clock-reset")
            .takes_value(false)
            .conflicts_with("nvml-gpu-clock")
            .help("Reset nvidia gpu min,max frequency per --nvml"))

        .arg(Arg::with_name("nvml-power-limit")
            .long("nvml-power-limit")
            .takes_value(true)
            .value_name("WATTS")
            .help("Set nvidia gpu power limit per --nvml, ex. 260, 260w, 0.26kw"))

        .get_matches_from(argv);

    Ok(Cli {
        show_cpu: if m.is_present("show-cpu") { Some(()) } else { None },
        show_intel_pstate: if m.is_present("show-pstate") { Some(()) } else { None },
        show_drm: if m.is_present("show-drm") { Some(()) } else { None },
        show_nvml: if m.is_present("show-nvml") { Some(()) } else { None },
        cpu: if let Some(v) = m.value_of("cpu") { Some(parse::cpu(v)?) } else { None },
        cpu_on: if let Some(v) = m.value_of("cpu-on") { Some(parse::cpu_on(v)?) } else { None },
        cpu_on_each: if let Some(v) = m.value_of("cpu-on-each") { Some(parse::cpu_on_each(v)?) } else { None },
        cpufreq_gov: m.value_of("cpufreq-gov").map(String::from),
        cpufreq_min: if let Some(v) = m.value_of("cpufreq-min") { Some(parse::cpufreq_min(v)?) } else { None },
        cpufreq_max: if let Some(v) = m.value_of("cpufreq-max") { Some(parse::cpufreq_max(v)?) } else { None },
        pstate_epb: if let Some(v) = m.value_of("pstate-epb") { Some(parse::pstate_epb(v)?) } else { None },
        pstate_epp: m.value_of("pstate-epp").map(String::from),
        drm_i915: if let Some(v) = m.value_of("drm-i915") { Some(parse::drm_i915(v)?) } else { None },
        drm_i915_min: if let Some(v) = m.value_of("drm-i915-min") { Some(parse::drm_i915_min(v)?) } else { None },
        drm_i915_max: if let Some(v) = m.value_of("drm-i915-max") { Some(parse::drm_i915_max(v)?) } else { None },
        drm_i915_boost: if let Some(v) = m.value_of("drm-i915-boost") { Some(parse::drm_i915_boost(v)?) } else { None },
        nvml: if let Some(v) = m.value_of("nvml") { Some(parse::nvml(v)?) } else { None },
        nvml_gpu_clock: if let Some(v) = m.value_of("nvml-gpu-clock") { Some(parse::nvml_gpu_clock(v)?) } else { None },
        nvml_gpu_clock_reset: if m.is_present("nvml-gpu-clock-reset") { Some(()) } else { None },
        nvml_power_limit: if let Some(v) = m.value_of("nvml-power-limit") { Some(parse::nvml_power_limit(v)?) } else { None },
    })
}