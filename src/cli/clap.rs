use clap::{App, AppSettings, Arg, ArgMatches, crate_version};
use log::debug;
use crate::cli::{Cli, Result};

const AFTER_HELP: &str = r#"    All flags may be expressed as environment variables. For example:

        --show-cpu                     => KNOBS_SHOW_CPU=1
        --cpu 1,3-5                    => KNOBS_CPU=1,3-5
        --cpufreq-gov schedutil        => KNOBS_CPUFREQ_GOV=schedutil
        --nvml-gpu-clock 800mhz,1.2ghz => KNOBS_NVML_GPU_CLOCK=800mhz,1.2ghz

    The log level may be set via KNOBS_LOG. The default log level is error. For example:

        KNOBS_LOG=warn
        KNOBS_LOG=debug
"#;

// Determine the binary name from argv[0].
fn argv0(argv: &[String]) -> &str {
    const DEFAULT: &str = "knobs";
    argv
        .first()
        .map(|s| s.as_str())
        .unwrap_or(DEFAULT)
        .split('/')
        .last()
        .unwrap_or(DEFAULT)
}

// Convert a cli flag name to an environment variable name.
fn env_name(cli_name: &str) -> String {
    format!("KNOBS_{}", cli_name.to_uppercase().replace("-", "_"))
}

// Return a flag value, preferring the command line, falling back to environment variables.
fn flag(name: &str, m: &ArgMatches) -> Option<()> {
    match m.is_present(name) {
        true => Some(()),
        false =>
            if std::env::var(&env_name(name))
                .ok()
                .map(|v| !v.is_empty() && v != "0" && v.to_lowercase() != "false")
                .unwrap_or(false)
            {
                debug!("--{}: using value from environment", name);
                Some(())
            } else {
                None
            },
    }
}

// Parse and return an argument value, preferring the command line, falling back to environment variables.
fn arg<P, T>(name: &str, m: &ArgMatches, parse: P) -> Result<Option<T>>
where
    P: FnOnce(&str) -> Result<T>
{
    let val = match m.value_of(name) {
        Some(v) => v.to_string(),
        None =>
            match std::env::var(&env_name(name)) {
                Ok(v) => {
                    debug!("--{}: using value from environment", name);
                    v
                },
                _ => return Ok(None),
            },
    };
    Ok(Some(parse(&val)?))
}

// Build and parse the clap cli specification, returning a `Cli` instance.
pub fn parse(argv: &[String]) -> Result<Cli> {

    crate::cli::logging::configure();

    let m = App::new(argv0(argv))

        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::DisableVersion)
        .setting(AppSettings::UnifiedHelpMessage)

        .version(crate_version!())

        .after_help(AFTER_HELP)

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

    use crate::cli::parse;

    Ok(Cli {
        show_cpu: flag("show-cpu", &m),
        show_intel_pstate: flag("show-pstate", &m),
        show_drm: flag("show-drm", &m),
        show_nvml: flag("show-nvml", &m),
        cpu: arg("cpu", &m, parse::cpu)?,
        cpu_on: arg("cpu-on", &m, parse::cpu_on)?,
        cpu_on_each: arg("cpu-on-each", &m, parse::cpu_on_each)?,
        cpufreq_gov: arg("cpufreq-gov", &m, parse::cpufreq_gov)?,
        cpufreq_min: arg("cpufreq-min", &m, parse::cpufreq_min)?,
        cpufreq_max: arg("cpufreq-max", &m, parse::cpufreq_max)?,
        pstate_epb: arg("pstate-epb", &m, parse::pstate_epb)?,
        pstate_epp: arg("pstate-epp", &m, parse::pstate_epp)?,
        drm_i915: arg("drm-i915", &m, parse::drm_i915)?,
        drm_i915_min: arg("drm-i915-min", &m, parse::drm_i915_min)?,
        drm_i915_max: arg("drm-i915-max", &m, parse::drm_i915_max)?,
        drm_i915_boost: arg("drm-i915-boost", &m, parse::drm_i915_boost)?,
        nvml: arg("nvml", &m, parse::nvml)?,
        nvml_gpu_clock: arg("nvml-gpu-clock", &m, parse::nvml_gpu_clock)?,
        nvml_gpu_clock_reset: flag("nvml-gpu-clock-reset", &m),
        nvml_power_limit: arg("nvml-power-limit", &m, parse::nvml_power_limit)?,
    })
}
