use crate::cli::*;

// Build and return a clap app.
pub fn build<'a, 'b>() -> clap::App<'a, 'b> {
    use clap::{App, AppSettings, Arg, crate_version};

    let a = App::new(NAME)

        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::DisableVersion)
        .setting(AppSettings::TrailingVarArg)
        .setting(AppSettings::UnifiedHelpMessage)

        .version(crate_version!())

        .after_help(AFTER_HELP)

        .arg(Arg::with_name(ARG_QUIET)
            .short("q")
            .long(ARG_QUIET)
            .takes_value(false)
            .help("Do not print values"))

        .arg(Arg::with_name(ARG_SHOW_CPU)
            .long(ARG_SHOW_CPU)
            .takes_value(false)
            .help("Print cpu and cpufreq values"))

        .arg(Arg::with_name(ARG_SHOW_DRM)
            .long(ARG_SHOW_DRM)
            .takes_value(false)
            .help("Print drm values"));

    #[cfg(feature = "nvml")]
    let a = a

        .arg(Arg::with_name(ARG_SHOW_NVML)
            .long(ARG_SHOW_NVML)
            .takes_value(false)
            .help("Print nvidia management values"));

    let a = a

        .arg(Arg::with_name(ARG_SHOW_PSTATE)
            .long(ARG_SHOW_PSTATE)
            .takes_value(false)
            .help("Print intel-pstate values"))

        .arg(Arg::with_name(ARG_SHOW_RAPL)
            .long(ARG_SHOW_RAPL)
            .takes_value(false)
            .help("Print intel-rapl values"))

        .arg(Arg::with_name(ARG_CPU)
            .short("c")
            .long(ARG_CPU)
            .takes_value(true)
            .value_name("IDS")
            .help("Target cpu ids, default all, ex. 0,1,3-5"))

        .arg(Arg::with_name(ARG_CPU_ON)
            .short("o")
            .long(ARG_CPU_ON)
            .takes_value(true)
            .value_name("BOOL")
            .help("Set cpu online status per --cpu"))

        .arg(Arg::with_name(ARG_CPU_ON_EACH)
            .short("O")
            .long(ARG_CPU_ON_EACH)
            .takes_value(true)
            .value_name("TOGGLES")
            .help("Set cpu online status, ex. 10_1 → 0:ON 1:OFF 2:SKIP 3:ON"))

        .arg(Arg::with_name(ARG_CPUFREQ_GOV)
            .short("g")
            .long(ARG_CPUFREQ_GOV)
            .takes_value(true)
            .value_name("STR")
            .help("Set cpufreq governor per --cpu"))

        .arg(Arg::with_name(ARG_CPUFREQ_MIN)
            .short("n")
            .long(ARG_CPUFREQ_MIN)
            .takes_value(true)
            .value_name("HERTZ")
            .help("Set cpufreq min freq per --cpu, ex. 1200 or 1.2ghz"))

        .arg(Arg::with_name(ARG_CPUFREQ_MAX)
            .short("x")
            .long(ARG_CPUFREQ_MAX)
            .takes_value(true)
            .value_name("HERTZ")
            .help("Set cpufreq max freq per --cpu, ex. 1200 or 1.2ghz"))

        .arg(Arg::with_name(ARG_DRM_I915)
            .long(ARG_DRM_I915)
            .takes_value(true)
            .value_name("IDS")
            .help("Target i915 card ids or pci ids, default all, ex. 0,1,3-5"))

        .arg(Arg::with_name(ARG_DRM_I915_MIN)
            .long(ARG_DRM_I915_MIN)
            .takes_value(true)
            .value_name("HERTZ")
            .help("Set i915 min frequency per --drm-i915, ex. 1200 or 1.2ghz"))

        .arg(Arg::with_name(ARG_DRM_I915_MAX)
            .long(ARG_DRM_I915_MAX)
            .takes_value(true)
            .value_name("HERTZ")
            .help("Set i915 max frequency per --drm-i915, ex. 1200 or 1.2ghz"))

        .arg(Arg::with_name(ARG_DRM_I915_BOOST)
            .long(ARG_DRM_I915_BOOST)
            .takes_value(true)
            .value_name("HERTZ")
            .help("Set i915 boost frequency per --drm-i915, ex. 1200 or 1.2ghz"));

    #[cfg(feature = "nvml")]
    let a = a

        .arg(Arg::with_name(ARG_NVML)
            .long(ARG_NVML)
            .takes_value(true)
            .value_name("IDS")
            .help("Target nvidia card ids or pci ids, default all, ex. 0,1,3-5"))

        .arg(Arg::with_name(ARG_NVML_GPU_MIN)
            .long(ARG_NVML_GPU_MIN)
            .takes_value(true)
            .value_name("HERTZ")
            .help("Set nvidia gpu min frequency per --nvml, ex. 1200 or 1.2ghz")
            .requires(ARG_NVML_GPU_MAX))

        .arg(Arg::with_name(ARG_NVML_GPU_MAX)
            .long(ARG_NVML_GPU_MAX)
            .takes_value(true)
            .value_name("HERTZ")
            .help("Set nvidia gpu max frequency per --nvml, ex. 1200 or 1.2ghz")
            .requires(ARG_NVML_GPU_MIN))

        .arg(Arg::with_name(ARG_NVML_GPU_RESET)
            .long(ARG_NVML_GPU_RESET)
            .takes_value(false)
            .conflicts_with("nvml-gpu-freq")
            .help("Reset nvidia gpu frequency to default per --nvml"))

        .arg(Arg::with_name(ARG_NVML_POWER_LIMIT)
            .long(ARG_NVML_POWER_LIMIT)
            .takes_value(true)
            .value_name("WATTS")
            .help("Set nvidia card power limit per --nvml"));

    let a = a

        .arg(Arg::with_name(ARG_PSTATE_EPB)
            .long(ARG_PSTATE_EPB)
            .takes_value(true)
            .value_name("0-15")
            .help("Set intel-pstate energy/performance bias per --cpu"))

        .arg(Arg::with_name(ARG_PSTATE_EPP)
            .long(ARG_PSTATE_EPP)
            .takes_value(true)
            .value_name("STR")
            .help("Set intel-pstate energy/performance pref per --cpu"))

        .arg(Arg::with_name(ARG_RAPL_PACKAGE)
            .short("P")
            .long(ARG_RAPL_PACKAGE)
            .takes_value(true)
            .value_name("INT")
            .help("Target intel-rapl package"))

        .arg(Arg::with_name(ARG_RAPL_ZONE)
            .short("Z")
            .long(ARG_RAPL_ZONE)
            .takes_value(true)
            .value_name("INT")
            .help("Target intel-rapl sub-zone"))

        .arg(Arg::with_name(ARG_RAPL_LONG_LIMIT)
            .short("L")
            .long(ARG_RAPL_LONG_LIMIT)
            .takes_value(true)
            .value_name("WATTS")
            .help("Set intel-rapl long_term power limit per --rapl-package/zone")
            .requires(ARG_RAPL_PACKAGE))

        .arg(Arg::with_name(ARG_RAPL_LONG_WINDOW)
            .long(ARG_RAPL_LONG_WINDOW)
            .takes_value(true)
            .value_name("SECS")
            .help("Set intel-rapl long_term time window per --rapl-package/zone")
            .requires(ARG_RAPL_PACKAGE))

        .arg(Arg::with_name(ARG_RAPL_SHORT_LIMIT)
            .short("S")
            .long(ARG_RAPL_SHORT_LIMIT)
            .takes_value(true)
            .value_name("WATTS")
            .help("Set intel-rapl short_term power limit per --rapl-package/zone")
            .requires(ARG_RAPL_PACKAGE))

        .arg(Arg::with_name(ARG_RAPL_SHORT_WINDOW)
            .long(ARG_RAPL_SHORT_WINDOW)
            .takes_value(true)
            .value_name("SECS")
            .help("Set intel-rapl short_term time window per --rapl-package/zone")
            .requires(ARG_RAPL_PACKAGE))

        .arg(Arg::with_name(ARG_PROFILE))

        .arg(Arg::with_name(ARG_CHAIN)
            .raw(true));

    a
}
