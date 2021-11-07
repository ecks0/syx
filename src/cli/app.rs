use clap::{crate_version, App, AppSettings, Arg};

use crate::NAME;

pub(super) const ARG_QUIET: &str = "quiet";

pub(super) const ARG_SHOW_CPU: &str = "show-cpu";
pub(super) const ARG_SHOW_DRM: &str = "show-drm";
#[cfg(feature = "nvml")]
pub(super) const ARG_SHOW_NVML: &str = "show-nvml";
pub(super) const ARG_SHOW_PSTATE: &str = "show-pstate";
pub(super) const ARG_SHOW_RAPL: &str = "show-rapl";

pub(super) const ARG_CPU: &str = "cpu";
pub(super) const ARG_CPU_ON: &str = "cpu-on";
pub(super) const ARG_CPU_ON_EACH: &str = "cpu-on-each";

pub(super) const ARG_CPUFREQ_GOV: &str = "cpufreq-gov";
pub(super) const ARG_CPUFREQ_MIN: &str = "cpufreq-min";
pub(super) const ARG_CPUFREQ_MAX: &str = "cpufreq-max";

pub(super) const ARG_DRM_I915: &str = "drm-i915";
pub(super) const ARG_DRM_I915_MIN: &str = "drm-i915-min";
pub(super) const ARG_DRM_I915_MAX: &str = "drm-i915-max";
pub(super) const ARG_DRM_I915_BOOST: &str = "drm-i915-boost";

#[cfg(feature = "nvml")]
pub(super) const ARG_NVML: &str = "nvml";
#[cfg(feature = "nvml")]
pub(super) const ARG_NVML_GPU_MIN: &str = "nvml-gpu-min";
#[cfg(feature = "nvml")]
pub(super) const ARG_NVML_GPU_MAX: &str = "nvml-gpu-max";
#[cfg(feature = "nvml")]
pub(super) const ARG_NVML_GPU_RESET: &str = "nvml-gpu-reset";
#[cfg(feature = "nvml")]
pub(super) const ARG_NVML_POWER_LIMIT: &str = "nvml-power-limit";

pub(super) const ARG_PSTATE_EPB: &str = "pstate-epb";
pub(super) const ARG_PSTATE_EPP: &str = "pstate-epp";

pub(super) const ARG_RAPL_PACKAGE: &str = "rapl-package";
pub(super) const ARG_RAPL_ZONE: &str = "rapl-zone";
pub(super) const ARG_RAPL_LONG_LIMIT: &str = "rapl-long-limit";
pub(super) const ARG_RAPL_LONG_WINDOW: &str = "rapl-long-window";
pub(super) const ARG_RAPL_SHORT_LIMIT: &str = "rapl-short-limit";
pub(super) const ARG_RAPL_SHORT_WINDOW: &str = "rapl-short-window";

pub(super) const ARG_PROFILE: &str = "PROFILE";

pub(super) const ARG_CHAIN: &str = "CHAIN";

const AFTER_HELP: &str = r#"            BOOL   0, 1, true, false
             IDS   comma-delimited sequence of integers and/or integer ranges
           HERTZ*  mhz when unspecified: hz/h - khz/k - mhz/m - ghz/g - thz/t
            SECS   ms when unspecified: ns/n - us/u - ms/m - s
         TOGGLES   sequence of 0 (off), 1 (on), or _ (skip), where position denotes id
           WATTS*  mw when unspecified: uw/u - mw/m - w - kw/k

        * Floating point values may be given for these units.

    All supported values are shown by default unless the --show-* or --quiet flags are used.

    All flags may be expressed as env vars. For example:

        --show-cpu              → KNOBS_SHOW_CPU=1
        --cpu 1,3-5             → KNOBS_CPU=1,3-5
        --drm-i915-boost 1.2ghz → KNOBS_DRM_I915_BOOST=1.2ghz

    The KNOBS_LOG env var may be set to trace, debug, info, warn, or error (default).
"#;

// Build and return a clap app.
pub(super) fn build<'a, 'b>() -> clap::App<'a, 'b> {
    let a = App::new(NAME)
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::DisableVersion)
        .setting(AppSettings::TrailingVarArg)
        .setting(AppSettings::UnifiedHelpMessage)
        .version(crate_version!())
        .after_help(AFTER_HELP)
        .arg(
            Arg::with_name(ARG_QUIET)
                .short("q")
                .long(ARG_QUIET)
                .takes_value(false)
                .help("Do not print values"),
        )
        .arg(
            Arg::with_name(ARG_SHOW_CPU)
                .long(ARG_SHOW_CPU)
                .takes_value(false)
                .help("Print cpu and cpufreq values"),
        )
        .arg(
            Arg::with_name(ARG_SHOW_DRM)
                .long(ARG_SHOW_DRM)
                .takes_value(false)
                .help("Print drm values"),
        );

    #[cfg(feature = "nvml")]
    let a = a.arg(
        Arg::with_name(ARG_SHOW_NVML)
            .long(ARG_SHOW_NVML)
            .takes_value(false)
            .help("Print nvidia management values"),
    );

    let a = a
        .arg(
            Arg::with_name(ARG_SHOW_PSTATE)
                .long(ARG_SHOW_PSTATE)
                .takes_value(false)
                .help("Print intel-pstate values"),
        )
        .arg(
            Arg::with_name(ARG_SHOW_RAPL)
                .long(ARG_SHOW_RAPL)
                .takes_value(false)
                .help("Print intel-rapl values"),
        )
        .arg(
            Arg::with_name(ARG_CPU)
                .short("c")
                .long(ARG_CPU)
                .takes_value(true)
                .value_name("IDS")
                .help("Target cpu ids, default all, ex. 0,1,3-5"),
        )
        .arg(
            Arg::with_name(ARG_CPU_ON)
                .short("o")
                .long(ARG_CPU_ON)
                .takes_value(true)
                .value_name("BOOL")
                .help("Set cpu online status per --cpu"),
        )
        .arg(
            Arg::with_name(ARG_CPU_ON_EACH)
                .short("O")
                .long(ARG_CPU_ON_EACH)
                .takes_value(true)
                .value_name("TOGGLES")
                .help("Set cpu online status, ex. 10_1 → 0:ON 1:OFF 2:SKIP 3:ON"),
        )
        .arg(
            Arg::with_name(ARG_CPUFREQ_GOV)
                .short("g")
                .long(ARG_CPUFREQ_GOV)
                .takes_value(true)
                .value_name("STR")
                .help("Set cpufreq governor per --cpu"),
        )
        .arg(
            Arg::with_name(ARG_CPUFREQ_MIN)
                .short("n")
                .long(ARG_CPUFREQ_MIN)
                .takes_value(true)
                .value_name("HERTZ")
                .help("Set cpufreq min freq per --cpu, ex. 1200 or 1.2ghz"),
        )
        .arg(
            Arg::with_name(ARG_CPUFREQ_MAX)
                .short("x")
                .long(ARG_CPUFREQ_MAX)
                .takes_value(true)
                .value_name("HERTZ")
                .help("Set cpufreq max freq per --cpu, ex. 1200 or 1.2ghz"),
        )
        .arg(
            Arg::with_name(ARG_DRM_I915)
                .long(ARG_DRM_I915)
                .takes_value(true)
                .value_name("IDS")
                .help("Target i915 card ids or pci ids, default all, ex. 0,1,3-5"),
        )
        .arg(
            Arg::with_name(ARG_DRM_I915_MIN)
                .long(ARG_DRM_I915_MIN)
                .takes_value(true)
                .value_name("HERTZ")
                .help("Set i915 min frequency per --drm-i915, ex. 1200 or 1.2ghz"),
        )
        .arg(
            Arg::with_name(ARG_DRM_I915_MAX)
                .long(ARG_DRM_I915_MAX)
                .takes_value(true)
                .value_name("HERTZ")
                .help("Set i915 max frequency per --drm-i915, ex. 1200 or 1.2ghz"),
        )
        .arg(
            Arg::with_name(ARG_DRM_I915_BOOST)
                .long(ARG_DRM_I915_BOOST)
                .takes_value(true)
                .value_name("HERTZ")
                .help("Set i915 boost frequency per --drm-i915, ex. 1200 or 1.2ghz"),
        );

    #[cfg(feature = "nvml")]
    let a = a
        .arg(
            Arg::with_name(ARG_NVML)
                .long(ARG_NVML)
                .takes_value(true)
                .value_name("IDS")
                .help("Target nvidia card ids or pci ids, default all, ex. 0,1,3-5"),
        )
        .arg(
            Arg::with_name(ARG_NVML_GPU_MIN)
                .long(ARG_NVML_GPU_MIN)
                .takes_value(true)
                .value_name("HERTZ")
                .help("Set nvidia gpu min frequency per --nvml, ex. 1200 or 1.2ghz")
                .requires(ARG_NVML_GPU_MAX),
        )
        .arg(
            Arg::with_name(ARG_NVML_GPU_MAX)
                .long(ARG_NVML_GPU_MAX)
                .takes_value(true)
                .value_name("HERTZ")
                .help("Set nvidia gpu max frequency per --nvml, ex. 1200 or 1.2ghz")
                .requires(ARG_NVML_GPU_MIN),
        )
        .arg(
            Arg::with_name(ARG_NVML_GPU_RESET)
                .long(ARG_NVML_GPU_RESET)
                .takes_value(false)
                .conflicts_with("nvml-gpu-freq")
                .help("Reset nvidia gpu frequency to default per --nvml"),
        )
        .arg(
            Arg::with_name(ARG_NVML_POWER_LIMIT)
                .long(ARG_NVML_POWER_LIMIT)
                .takes_value(true)
                .value_name("WATTS")
                .help("Set nvidia card power limit per --nvml"),
        );

    let a = a
        .arg(
            Arg::with_name(ARG_PSTATE_EPB)
                .long(ARG_PSTATE_EPB)
                .takes_value(true)
                .value_name("0-15")
                .help("Set intel-pstate energy/performance bias per --cpu"),
        )
        .arg(
            Arg::with_name(ARG_PSTATE_EPP)
                .long(ARG_PSTATE_EPP)
                .takes_value(true)
                .value_name("STR")
                .help("Set intel-pstate energy/performance pref per --cpu"),
        )
        .arg(
            Arg::with_name(ARG_RAPL_PACKAGE)
                .short("P")
                .long(ARG_RAPL_PACKAGE)
                .takes_value(true)
                .value_name("INT")
                .help("Target intel-rapl package"),
        )
        .arg(
            Arg::with_name(ARG_RAPL_ZONE)
                .short("Z")
                .long(ARG_RAPL_ZONE)
                .takes_value(true)
                .value_name("INT")
                .help("Target intel-rapl sub-zone"),
        )
        .arg(
            Arg::with_name(ARG_RAPL_LONG_LIMIT)
                .short("L")
                .long(ARG_RAPL_LONG_LIMIT)
                .takes_value(true)
                .value_name("WATTS")
                .help("Set intel-rapl long term power limit per --rapl-package/zone")
                .requires(ARG_RAPL_PACKAGE),
        )
        .arg(
            Arg::with_name(ARG_RAPL_LONG_WINDOW)
                .long(ARG_RAPL_LONG_WINDOW)
                .takes_value(true)
                .value_name("SECS")
                .help("Set intel-rapl long term time window per --rapl-package/zone")
                .requires(ARG_RAPL_PACKAGE),
        )
        .arg(
            Arg::with_name(ARG_RAPL_SHORT_LIMIT)
                .short("S")
                .long(ARG_RAPL_SHORT_LIMIT)
                .takes_value(true)
                .value_name("WATTS")
                .help("Set intel-rapl short term power limit per --rapl-package/zone")
                .requires(ARG_RAPL_PACKAGE),
        )
        .arg(
            Arg::with_name(ARG_RAPL_SHORT_WINDOW)
                .long(ARG_RAPL_SHORT_WINDOW)
                .takes_value(true)
                .value_name("SECS")
                .help("Set intel-rapl short term time window per --rapl-package/zone")
                .requires(ARG_RAPL_PACKAGE),
        )
        .arg(Arg::with_name(ARG_PROFILE))
        .arg(Arg::with_name(ARG_CHAIN).raw(true));

    a
}
