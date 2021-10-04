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

pub fn parse(argv: &[String]) -> Result<Cli> {
    let m = App::new(argv0(argv))
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::DisableVersion)
        .version(crate_version!())

        .arg(Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .takes_value(false)
            .help("Enables verbose output"))

        .arg(Arg::with_name("cpus")
            .short("c")
            .long("cpus")
            .takes_value(true)
            .value_name("INDICES")
            .help("Target cpu ids, default all, ex. 0,1,3-5"))

        .arg(Arg::with_name("cpu-on")
            .short("o")
            .long("cpu-on")
            .takes_value(true)
            .value_name("0|1")
            .help("Set cpu online status per --cpus"))

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
            .help("Set cpufreq governor per --cpus"))

        .arg(Arg::with_name("cpufreq-min")
            .short("n")
            .long("cpufreq-min")
            .takes_value(true)
            .value_name("HZ")
            .help("Set cpufreq min freq per --cpus, ex. 1200mhz, 1.2ghz"))

        .arg(Arg::with_name("cpufreq-max")
            .short("x")
            .long("cpufreq-max")
            .takes_value(true)
            .value_name("HZ")
            .help("Set cpufreq max freq per --cpus, ex. 1200mhz, 1.2ghz"))

        .arg(Arg::with_name("pstate-epb")
            .long("pstate-epb")
            .takes_value(true)
            .value_name("0-15")
            .help("Set intel_pstate energy/performance bias per --cpus"))

        .arg(Arg::with_name("pstate-epp")
            .long("pstate-epp")
            .takes_value(true)
            .value_name("NAME")
            .help("Set intel_pstate energy/performance pref per --cpus"))

        .arg(Arg::with_name("drm-i915")
            .long("drm-i915")
            .takes_value(true)
            .value_name("INDICES")
            .help("Target i915 card ids, default all, ex. 0,1,3-5"))

        .arg(Arg::with_name("drm-i915-min")
            .long("drm-i915-min")
            .takes_value(true)
            .value_name("HZ")
            .help("Set i915 min frequency per --drm-i915, ex. 1200mhz, 1.2ghz"))

        .arg(Arg::with_name("drm-i915-max")
            .long("drm-i915-max")
            .takes_value(true)
            .value_name("HZ")
            .help("Set i915 max frequency per --drm-i915, ex. 1200mhz, 1.2ghz"))

        .arg(Arg::with_name("drm-i915-boost")
            .long("drm-i915-boost")
            .takes_value(true)
            .value_name("HZ")
            .help("Set i915 boost frequency per --drm-i915, ex. 1200mhz, 1.2ghz"))

        .get_matches_from(argv);

    logging::configure(m.is_present("verbose"))?;

    Ok(Cli {
        cpus: parse::cpus(m.value_of("cpus"))?,
        cpu_on: parse::cpu_on(m.value_of("cpu-on"))?,
        cpu_on_each: parse::cpu_on_each(m.value_of("cpu-on-each"))?,
        cpufreq_gov: parse::cpufreq_gov(m.value_of("cpufreq-gov")),
        cpufreq_min: parse::cpufreq_min(m.value_of("cpufreq-min"))?,
        cpufreq_max: parse::cpufreq_max(m.value_of("cpufreq-max"))?,
        pstate_epb: parse::pstate_epb(m.value_of("pstate-epb"))?,
        pstate_epp: parse::pstate_epp(m.value_of("pstate-epp")),
        drm_i915: parse::drm_i915(m.value_of("drm-i915"))?,
        drm_i915_min: parse::drm_i915_min(m.value_of("drm-i915-min"))?,
        drm_i915_max: parse::drm_i915_max(m.value_of("drm-i915-max"))?,
        drm_i915_boost: parse::drm_i915_boost(m.value_of("drm-i915-boost"))?,
    })
}