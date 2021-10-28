# knobs

A command-line utility and library for viewing and setting Linux power and performance values.

Supported knobs:

- cpu: online/offline
- cpufreq: governor, min/max frequencies
- drm
  - i915: min/max/boost frequencies
- intel-pstate: epb, epp
- intel-rapl: power limit, time window - per zone/subzone/constraint
- nvml
  - nvidia gpu clock min/max frequency, power limit
  - enabled via the `nvml` feature flag
  - requires the nvidia management library at runtime, usually installed with the proprietary driver
    - knobs will work correcly with `nvml` enabled even if there is no driver/library or nvidia hardware present

## Output

A contrived example showing all available tables:

```
 CPU  Online  Governor   Cur      Min      Max      CPU min  CPU max
 ---  ------  --------   ---      ---      ---      -------  -------
 0    •       powersave  993 MHz  400 MHz  1.8 GHz  400 MHz  3.4 GHz
 1    true    powersave  629 MHz  400 MHz  1.8 GHz  400 MHz  3.4 GHz
 2    true    powersave  600 MHz  400 MHz  1.8 GHz  400 MHz  3.4 GHz
 3    true    powersave  600 MHz  400 MHz  1.8 GHz  400 MHz  3.4 GHz

 CPU  Available governors
 ---  -------------------
 all  performance powersave

 intel_pstate: active

 CPU  EP bias  EP preference
 ---  -------  -------------
 all  6        balance_performance

 CPU  Available EP preferences
 ---  ------------------------
 all  default performance balance_performance balance_power power

 Zone name  Zone  C0 limit  C1 limit  C0 window    C1 window  Energy
 ---------  ----  --------  --------  ---------    ---------  ------
 package-0  0     4 W       6 W       27983872 us  2440 us    17.763 kJ
 core       0:0   0 W       •         976 us       •          1.760 kJ
 uncore     0:1   0 W       •         976 us       •          944.211 mJ
 dram       0:2   0 W       •         976 us       •          2.704 kJ
 psys       1     0 W       0 W       27983872 us  976 us     1.812 kJ

 Card  Driver  Actual   Req'd    Min      Max      Boost    GPU min  GPU max
 ----  ------  ------   -----    ---      ---      -----    -------  -------
 0     i915    100 MHz  300 MHz  100 MHz  1.4 GHz  1.4 GHz  100 MHz  1.4 GHz

          Nvidia GPU  0
 -------------------  -------------------
                Name  GeForce RTX 2080 Ti
              PCI ID  00000000:02:00.0
    Graphics cur/max  690 MHz / 2.2 GHz
      Memory cur/max  810 MHz / 7.0 GHz
          SM cur/max  690 MHz / 2.2 GHz
       Video cur/max  630 MHz / 1.9 GHz
   Memory used/total  1.1 GB / 11.6 GB
    Power used/limit  23 W / 260 W
 Power limit min/max  100 W / 325 W
```

## Help

```
knobs 0.2.4

USAGE:
    knobs [OPTIONS] [-- <chain>...]

OPTIONS:
    -q, --quiet                       Do not print values
        --show-cpu                    Print cpu and cpufreq values
        --show-drm                    Print drm values
        --show-nvml                   Print nvidia management values
        --show-pstate                 Print intel-pstate values
        --show-rapl                   Print intel-rapl values
    -c, --cpu <IDS>                   Target cpu ids, default all, ex. 0,1,3-5
    -o, --cpu-on <0|1>                Set cpu online status per --cpu
    -O, --cpus-on <TOGGLES>           Set cpu online status, ex. 10_1 → 0=ON 1=OFF 2=SKIP 3=ON
    -g, --cpufreq-gov <STR>           Set cpufreq governor per --cpu
    -n, --cpufreq-min <HZ>            Set cpufreq min freq per --cpu, ex. 1200 or 1.2ghz
    -x, --cpufreq-max <HZ>            Set cpufreq max freq per --cpu, ex. 1200 or 1.2ghz
        --drm-i915 <IDS>              Target i915 card ids or pci ids, default all, ex. 0,1,3-5
        --drm-i915-min <HZ>           Set i915 min frequency per --drm-i915, ex. 1200 or 1.2ghz
        --drm-i915-max <HZ>           Set i915 max frequency per --drm-i915, ex. 1200 or 1.2ghz
        --drm-i915-boost <HZ>         Set i915 boost frequency per --drm-i915, ex. 1200 or 1.2ghz
        --nvml <IDS>                  Target nvidia card ids or pci ids, default all, ex. 0,1,3-5
        --nvml-gpu-min <HZ>           Set nvidia gpu min frequency per --nvml, ex. 1200 or 1.2ghz
        --nvml-gpu-max <HZ>           Set nvidia gpu max frequency per --nvml, ex. 1200 or 1.2ghz
        --nvml-gpu-reset              Reset nvidia gpu frequency to default per --nvml
        --nvml-power-limit <WATTS>    Set nvidia card power limit per --nvml
        --pstate-epb <0-15>           Set intel-pstate energy/performance bias per --cpu
        --pstate-epp <STR>            Set intel-pstate energy/performance pref per --cpu
    -P, --rapl-package <INT>          Target intel-rapl package, default 0
    -Z, --rapl-zone <INT>             Target intel-rapl sub-zone, default none
    -0, --rapl-c0-limit <WATTS>       Set intel-rapl c0 power limit per --rapl-{package,zone}
    -1, --rapl-c1-limit <WATTS>       Set intel-rapl c1 power limit per --rapl-{package,zone}
        --rapl-c0-window <SECS>       Set intel-rapl c0 time window per --rapl-{package,zone}
        --rapl-c1-winodw <SECS>       Set intel-rapl c1 time window per --rapl-{package,zone}
    -h, --help                        Prints help information

ARGS:
    <chain>...

             IDS   A comma-delimited sequence of integers and/or integer ranges.
         TOGGLES   An sequence of 0 (off), 1 (on) or _ (skip) characters.
              HZ*  mhz when unspecified: hz/h - khz/k - mhz/m - ghz/g - thz/t
           WATTS*  mw when unspecified: uw/u - mw/m - w - kw/k
            SECS   ms when unspecified: ns/n - us/u - ms/m - s

        * Floating point values may be given for these units.

    Values for supported hardware are shown unless the --show-* or --quiet flags are used.

    All flags may be expressed as env vars. For example:

        --show-cpu                 → KNOBS_SHOW_CPU=1
        --cpu 1,3-5                → KNOBS_CPU=1,3-5
        --drm-i915-boost 1.2ghz    → KNOBS_DRM_I915_BOOST=1.2ghz

    The KNOBS_LOG env var may be set to trace, debug, info, warn, or error (default).

```

## Example usage

```bash
### set cpu minimum frequency → 800 MHz
### set cpu maximum frequency → 4.4 GHz

# for all cpus

knobs --cpufreq-min 800 --cpufreq-max 4.4ghz

knobs -n 800 -x 4.4ghz

# for the first 4 cpus only

knobs --cpu 0-3 --cpufreq-min 800 --cpufreq-max 4.4ghz

knobs -c 0-3 -n 800 -x 4.4ghz

### set cpus 1-3 online
### set cpus 4-7 offline

knobs --cpus-on _1110000

knobs -O _1110000

### set intel-pstate energy/performance bias → 6
### set intel-pstate energy/performance preference → balance_performance

# for all cpus

knobs --pstate-epb 6 --pstate-epp balance_performance

# for the first 4 cpus only

knobs --cpu 0-3 --pstate-epb 6 --pstate-epp balance_performance

knobs -c 0-3 --pstate-epb 6 --pstate-epp balance_performance

### set intel-rapl package 0 constraint 0 (long-term) → 28 watts
### set intel-rapl package 0 constraint 1 (short-term) → 35 watts
### (0 is the default value for --rapl-package, while --rapl-zone has no default.)

knobs --rapl-c0-limit 28w --rapl-c1-limit 35w

knobs -0 28w -1 35w

# set nvidia gpu minimum frequency → 600 MHz
# set nvidia gpu maximum frequency → 2.2 GHz

# for all gpus

knobs --nvml-gpu-min 600 --nvml-gpu-max 2.2ghz

# for the first 2 gpus only

knobs --nvml 0,1 --nvml-gpu-min 600 --nvml-gpu-max 2.2ghz

### knobs calls can be chained

### set cpus 1-3 online
### set cpus 4-7 offline

knobs --cpu 1-3 --cpu-on true -- --cpu 4-7 --cpu-on false

knobs -c 1-3 -o true -- -c 4-7 -o false

### set nvidia card 0 gpu min frequency → 1.8 GHz
### set nvidia card 0 gpu min frequency → 2.2 GHz
### set nvidia card 1 gpu min frequency → 600 MHz
### set nvidia card 1 gpu min frequency → 1000 MHz

knobs --nvml 0 --nvml-gpu-min 1800 --nvml-gpu-max 2.2ghz -- \
      --nvml 1 --nvml-gpu-min 600  --nvml-gpu-max 1000

### chains can be arbitarily long

knobs -c 0 -x 4.0ghz -- -c 1 -x 4.1ghz -- -c 2 -x 4.2ghz -- -c 3 -x 4.3ghz # and so on

### enable debug logging with -q/--quiet to see what commands are doing

KNOBS_LOG=debug knobs -q -x 4400
Chain 0
OK sysfs w /sys/devices/system/cpu/cpufreq/policy0/scaling_max_freq 4400000
OK sysfs w /sys/devices/system/cpu/cpufreq/policy1/scaling_max_freq 4400000
OK sysfs w /sys/devices/system/cpu/cpufreq/policy2/scaling_max_freq 4400000
OK sysfs w /sys/devices/system/cpu/cpufreq/policy3/scaling_max_freq 4400000

KNOBS_LOG=debug knobs -q \
    --rapl-c0-limit 10w --rapl-c1-limit 13w -- \
    --drm-i915-min 300mhz --drm-i915-max 900mhz
OK sysfs l /sys/class/drm/card0/device/driver ../../../bus/pci/drivers/i915
Chain 0
OK sysfs w /sys/devices/virtual/powercap/intel-rapl/intel-rapl:0/constraint_0_power_limit_uw 10000000
OK sysfs w /sys/devices/virtual/powercap/intel-rapl/intel-rapl:0/constraint_1_power_limit_uw 13000000
Chain 1
OK sysfs w /sys/class/drm/card0/gt_max_freq_mhz 900
OK sysfs w /sys/class/drm/card0/gt_min_freq_mhz 300

### each chain's instance is printed to the trace logging channel

KNOBS_LOG=trace knobs -q -x 4400
Chain 0
Knobs {
    cpu: Some(
        [
            0,
            1,
            2,
            3,
        ],
    ),
    cpu_on: None,
    cpus_on: None,
    cpufreq_gov: None,
    cpufreq_min: None,
    cpufreq_max: Some(
        Frequency {
            hertz: 4400000000.0,
        },
    ),
    drm_i915: None,
    drm_i915_min: None,
    drm_i915_max: None,
    drm_i915_boost: None,
    nvml: None,
    nvml_gpu_min: None,
    nvml_gpu_max: None,
    nvml_gpu_reset: None,
    nvml_power_limit: None,
    pstate_epb: None,
    pstate_epp: None,
    rapl_package: Some(
        0,
    ),
    rapl_zone: None,
    rapl_c0_limit: None,
    rapl_c1_limit: None,
    rapl_c0_window: None,
    rapl_c1_window: None,
}
OK sysfs w /sys/devices/system/cpu/cpufreq/policy0/scaling_max_freq 4400000
OK sysfs w /sys/devices/system/cpu/cpufreq/policy1/scaling_max_freq 4400000
OK sysfs w /sys/devices/system/cpu/cpufreq/policy2/scaling_max_freq 4400000
OK sysfs w /sys/devices/system/cpu/cpufreq/policy3/scaling_max_freq 4400000
```
