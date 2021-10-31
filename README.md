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
 0    •       powersave  546 MHz  400 MHz  4.8 GHz  400 MHz  4.8 GHz
 1    true    powersave  680 MHz  400 MHz  4.8 GHz  400 MHz  4.8 GHz
 2    true    powersave  723 MHz  400 MHz  5.0 GHz  400 MHz  5.0 GHz
 3    true    powersave  553 MHz  400 MHz  5.0 GHz  400 MHz  5.0 GHz

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

 Zone name  Zone  Long lim  Short lim  Long win     Short win  Usage
 ---------  ----  --------  ---------  --------     ---------  -----
 package-0  0     28 W      35 W       27983872 us  2440 us    3.8 W/s
 core       0:0   0 W       •          976 us       •          1.4 W/s
 uncore     0:1   0 W       •          976 us       •          27.5 mW/s
 psys       1     0 W       0 W        27983872 us  976 us     98.9 mW/s

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
knobs 0.2.5

USAGE:
    knobs [OPTIONS] [-- <CHAIN>...]

OPTIONS:
    -q, --quiet                       Do not print values
        --show-cpu                    Print cpu and cpufreq values
        --show-drm                    Print drm values
        --show-nvml                   Print nvidia management values
        --show-pstate                 Print intel-pstate values
        --show-rapl                   Print intel-rapl values
    -c, --cpu <IDS>                   Target cpu ids, default all, ex. 0,1,3-5
    -o, --cpu-online <BOOL>           Set cpu online status per --cpu
    -g, --cpufreq-gov <STR>           Set cpufreq governor per --cpu
    -n, --cpufreq-min <HERTZ>         Set cpufreq min freq per --cpu, ex. 1200 or 1.2ghz
    -x, --cpufreq-max <HERTZ>         Set cpufreq max freq per --cpu, ex. 1200 or 1.2ghz
        --drm-i915 <IDS>              Target i915 card ids or pci ids, default all, ex. 0,1,3-5
        --drm-i915-min <HERTZ>        Set i915 min frequency per --drm-i915, ex. 1200 or 1.2ghz
        --drm-i915-max <HERTZ>        Set i915 max frequency per --drm-i915, ex. 1200 or 1.2ghz
        --drm-i915-boost <HERTZ>      Set i915 boost frequency per --drm-i915, ex. 1200 or 1.2ghz
        --nvml <IDS>                  Target nvidia card ids or pci ids, default all, ex. 0,1,3-5
        --nvml-gpu-min <HERTZ>        Set nvidia gpu min frequency per --nvml, ex. 1200 or 1.2ghz
        --nvml-gpu-max <HERTZ>        Set nvidia gpu max frequency per --nvml, ex. 1200 or 1.2ghz
        --nvml-gpu-reset              Reset nvidia gpu frequency to default per --nvml
        --nvml-power-limit <WATTS>    Set nvidia card power limit per --nvml
        --pstate-epb <0-15>           Set intel-pstate energy/performance bias per --cpu
        --pstate-epp <STR>            Set intel-pstate energy/performance pref per --cpu
    -P, --rapl-package <INT>          Target intel-rapl package
    -Z, --rapl-zone <INT>             Target intel-rapl sub-zone
    -L, --rapl-long-limit <WATTS>     Set intel-rapl long_term power limit per --rapl-package/zone
        --rapl-long-window <SECS>     Set intel-rapl long_term time window per --rapl-package/zone
    -S, --rapl-short-limit <WATTS>    Set intel-rapl short_term power limit per --rapl-package/zone
        --rapl-short-window <SECS>    Set intel-rapl short_term time window per --rapl-package/zone
    -h, --help                        Prints help information

ARGS:
    <CHAIN>...

            BOOL   0, 1, true, false
             IDS   comma-delimited sequence of integers and/or integer ranges
           HERTZ*  mhz when unspecified: hz/h - khz/k - mhz/m - ghz/g - thz/t
            SECS   ms when unspecified: ns/n - us/u - ms/m - s
           WATTS*  mw when unspecified: uw/u - mw/m - w - kw/k

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

### set intel-pstate energy/performance bias → 6
### set intel-pstate energy/performance preference → balance_performance

# for all cpus

knobs --pstate-epb 6 --pstate-epp balance_performance

# for the first 4 cpus only

knobs --cpu 0-3 --pstate-epb 6 --pstate-epp balance_performance

knobs -c 0-3 --pstate-epb 6 --pstate-epp balance_performance

### set intel-rapl package 0 constraint named 'long_term' → 28 watts
### set intel-rapl package 0 constraint named 'short-term' → 35 watts

knobs --rapl-package 0 --rapl-long-limit 28w --rapl-short-limit 35w

knobs -P 0 -L 28w -S 35w

### set nvidia gpu minimum frequency → 600 MHz
#### set nvidia gpu maximum frequency → 2.2 GHz

# for all gpus

knobs --nvml-gpu-min 600 --nvml-gpu-max 2.2ghz

# for the first 2 gpus only

knobs --nvml 0,1 --nvml-gpu-min 600 --nvml-gpu-max 2.2ghz

### knobs calls can be chained

# set cpus 1-3 online
# set cpus 4-7 offline

knobs --cpu 1-3 --cpu-online true -- --cpu 4-7 --cpu-online false

knobs -c 1-3 -o true -- -c 4-7 -o false

# set nvidia card 0 gpu min frequency → 1.8 GHz
# set nvidia card 0 gpu min frequency → 2.2 GHz
# set nvidia card 1 gpu min frequency → 600 MHz
# set nvidia card 1 gpu min frequency → 1000 MHz

knobs --nvml 0 --nvml-gpu-min 1800 --nvml-gpu-max 2.2ghz -- \
      --nvml 1 --nvml-gpu-min 600  --nvml-gpu-max 1000

### chains can be arbitarily long

knobs -c 0 -x 4.0ghz -- -c 1 -x 4.1ghz -- -c 2 -x 4.2ghz -- -c 3 -x 4.3ghz # and so on
```
