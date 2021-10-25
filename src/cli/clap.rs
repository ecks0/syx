use clap::{App, AppSettings, Arg, ArgMatches, crate_version};
use log::debug;
use crate::cli::{Cli, Result};

const AFTER_HELP: &str = r#"    All present and supported subsystems are printed unless the --show-* or --quiet flags are used.

    The following special values and units are handled uniformly for all arguments.

         INDICES   A comma-delimited sequence of integers and/or integer ranges.

         TOGGLES   An enumeration of 0 (deactivate), 1 (activate) or _ (skip) characters, where the
                   character is an action, and the character's position is an ID on which to act.

           FREQ*     Default: megahertz when unspecified
                   Supported: hz/h - khz/k - mhz/m - ghz/g - thz/t

          POWER*     Default: milliwatts when unspecified
                   Supported: uw/u - mw/m - w - kw/k

        DURATION     Default: milliseconds when unspecified
                   Supported: ns/n - us/u - ms/m - s

        * Floating point values may be given for these units.

    All flags may be expressed as env vars. For example:

        --show-cpu                 → KNOBS_SHOW_CPU=1
        --cpu 1,3-5                → KNOBS_CPU=1,3-5
        --nvml-gpu-freq 800,1.2ghz → KNOBS_NVML_GPU_FREQ=800,1.2ghz

    The KNOBS_LOG env var may be set to trace, debug, info, warn, or error (default).
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
                    debug!("--{}: using value from environment: {}", name, v);
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

    let a = App::new(argv0(argv))

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
            .help("Print intel-pstate values"))

        .arg(Arg::with_name("show-rapl")
            .long("show-rapl")
            .takes_value(false)
            .help("Print intel-rapl values"))

        .arg(Arg::with_name("show-drm")
            .long("show-drm")
            .takes_value(false)
            .help("Print drm values"))

        .arg(Arg::with_name("show-nvml")
            .long("show-nvml")
            .takes_value(false)
            .help("Print nvidia management values"))

        .arg(Arg::with_name("quiet")
            .short("q")
            .long("quiet")
            .takes_value(false)
            .help("Do not print values"))

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
            .value_name("TOGGLES")
            .help("Set cpu online status, ex. 10_1 → 0=ON 1=OFF 2=SKIP 3=ON"))

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
            .value_name("FREQ")
            .help("Set cpufreq min freq per --cpu, ex. 1200 or 1.2ghz"))

        .arg(Arg::with_name("cpufreq-max")
            .short("x")
            .long("cpufreq-max")
            .takes_value(true)
            .value_name("FREQ")
            .help("Set cpufreq max freq per --cpu, ex. 1200 or 1.2ghz"))

        .arg(Arg::with_name("pstate-epb")
            .long("pstate-epb")
            .takes_value(true)
            .value_name("0-15")
            .help("Set intel-pstate energy/performance bias per --cpu"))

        .arg(Arg::with_name("pstate-epp")
            .long("pstate-epp")
            .takes_value(true)
            .value_name("NAME")
            .help("Set intel-pstate energy/performance pref per --cpu"))

        .arg(Arg::with_name("rapl-package")
            .short("p")
            .long("rapl-package")
            .takes_value(true)
            .value_name("INT")
            .help("Target intel-rapl package"))

        .arg(Arg::with_name("rapl-zone")
            .short("z")
            .long("rapl-zone")
            .takes_value(true)
            .value_name("INT")
            .help("Target intel-rapl zone"))

        .arg(Arg::with_name("rapl-constraint")
            .short("C")
            .long("rapl-constraint")
            .takes_value(true)
            .value_name("INT")
            .help("Target intel-rapl constraint"))

        .arg(Arg::with_name("rapl-limit")
            .short("l")
            .long("rapl-limit")
            .takes_value(true)
            .value_name("POWER")
            .requires_all(&["rapl-package", "rapl-constraint"])
            .help("Set intel-rapl power limit"))

        .arg(Arg::with_name("rapl-window")
            .short("w")
            .long("rapl-window")
            .takes_value(true)
            .value_name("DURATION")
            .requires_all(&["rapl-package", "rapl-constraint"])
            .help("Set intel-rapl time window"))

        .arg(Arg::with_name("drm-i915")
            .long("drm-i915")
            .takes_value(true)
            .value_name("INDICES")
            .help("Target i915 card ids or pci ids, default all, ex. 0,1,3-5"))

        .arg(Arg::with_name("drm-i915-min")
            .long("drm-i915-min")
            .takes_value(true)
            .value_name("FREQ")
            .help("Set i915 min frequency per --drm-i915, ex. 1200 or 1.2ghz"))

        .arg(Arg::with_name("drm-i915-max")
            .long("drm-i915-max")
            .takes_value(true)
            .value_name("FREQ")
            .help("Set i915 max frequency per --drm-i915, ex. 1200 or 1.2ghz"))

        .arg(Arg::with_name("drm-i915-boost")
            .long("drm-i915-boost")
            .takes_value(true)
            .value_name("FREQ")
            .help("Set i915 boost frequency per --drm-i915, ex. 1200 or 1.2ghz"));

    #[cfg(feature = "nvml")]
    let a = a
        .arg(Arg::with_name("nvml")
            .long("nvml")
            .takes_value(true)
            .value_name("INDICES")
            .help("Target nvidia card ids or pci ids, default all, ex. 0,1,3-5"))

        .arg(Arg::with_name("nvml-gpu-freq")
            .long("nvml-gpu-freq")
            .takes_value(true)
            .value_name("FREQ[,FREQ]")
            .conflicts_with("nvml-gpu-freq-reset")
            .help("Set nvidia gpu min,max frequency per --nvml, ex. 800,1.2ghz"))

        .arg(Arg::with_name("nvml-gpu-freq-reset")
            .long("nvml-gpu-freq-reset")
            .takes_value(false)
            .conflicts_with("nvml-gpu-freq")
            .help("Reset nvidia gpu frequency to default per --nvml"))

        .arg(Arg::with_name("nvml-power-limit")
            .long("nvml-power-limit")
            .takes_value(true)
            .value_name("POWER")
            .help("Set nvidia card power limit per --nvml"));

    let m = a.get_matches_from(argv);

    use crate::cli::parse;

    Ok(Cli {
        show_cpu: flag("show-cpu", &m),
        show_intel_pstate: flag("show-pstate", &m),
        show_drm: flag("show-drm", &m),
        show_intel_rapl: flag("show-rapl", &m),
        show_nvml: flag("show-nvml", &m),
        quiet: flag("quiet", &m),
        cpu: arg("cpu", &m, parse::cpu)?,
        cpu_on: arg("cpu-on", &m, parse::cpu_on)?,
        cpu_on_each: arg("cpu-on-each", &m, parse::cpu_on_each)?,
        cpufreq_gov: arg("cpufreq-gov", &m, parse::cpufreq_gov)?,
        cpufreq_min: arg("cpufreq-min", &m, parse::cpufreq_min)?,
        cpufreq_max: arg("cpufreq-max", &m, parse::cpufreq_max)?,
        pstate_epb: arg("pstate-epb", &m, parse::pstate_epb)?,
        pstate_epp: arg("pstate-epp", &m, parse::pstate_epp)?,
        rapl_package: arg("rapl-package", &m, parse::rapl_package)?,
        rapl_zone: arg("rapl-zone", &m, parse::rapl_zone)?,
        rapl_constraint: arg("rapl-constraint", &m, parse::rapl_constraint)?,
        rapl_limit: arg("rapl-limit", &m, parse::rapl_limit)?,
        rapl_window: arg("rapl-window", &m, parse::rapl_window)?,
        drm_i915: arg("drm-i915", &m, parse::drm_i915)?,
        drm_i915_min: arg("drm-i915-min", &m, parse::drm_i915_min)?,
        drm_i915_max: arg("drm-i915-max", &m, parse::drm_i915_max)?,
        drm_i915_boost: arg("drm-i915-boost", &m, parse::drm_i915_boost)?,
        #[cfg(feature = "nvml")]
        nvml: arg("nvml", &m, parse::nvml)?,
        #[cfg(feature = "nvml")]
        nvml_gpu_freq: arg("nvml-gpu-freq", &m, parse::nvml_gpu_freq)?,
        #[cfg(feature = "nvml")]
        nvml_gpu_freq_reset: flag("nvml-gpu-freq-reset", &m),
        #[cfg(feature = "nvml")]
        nvml_power_limit: arg("nvml-power-limit", &m, parse::nvml_power_limit)?,
    })
}
