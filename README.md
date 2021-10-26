# knobs

A command-line utility for viewing and setting Linux system power and performance values.

Supported knobs:

- cpu: online/offline
- cpufreq: governor, min/max frequencies
- intel-pstate: epb, epp
- intel-rapl: power limit, time window - per zone/subzone/constraint
- drm
  - i915: min/max/boost frequencies
- nvml
  - nvidia gpu clock min/max frequency, power limit
  - requires nvidia management library at runtime, usually installed with the proprietary driver
  - enabled via the `nvml` feature flag

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
knobs 0.2.3

USAGE:
    knobs [OPTIONS]

OPTIONS:
        --show-cpu                    Print cpu and cpufreq values
        --show-pstate                 Print intel-pstate values
        --show-rapl                   Print intel-rapl values
        --show-drm                    Print drm values
        --show-nvml                   Print nvidia management values
    -q, --quiet                       Do not print values
    -c, --cpu <IDS>                   Target cpu ids, default all, ex. 0,1,3-5
    -o, --cpu-on <0|1>                Set cpu online status per --cpu
    -O, --cpu-on-each <TOGGLES>       Set cpu online status, ex. 10_1 → 0=ON 1=OFF 2=SKIP 3=ON
    -g, --cpufreq-gov <STR>           Set cpufreq governor per --cpu
    -n, --cpufreq-min <HZ>            Set cpufreq min freq per --cpu, ex. 1200 or 1.2ghz
    -x, --cpufreq-max <HZ>            Set cpufreq max freq per --cpu, ex. 1200 or 1.2ghz
        --pstate-epb <0-15>           Set intel-pstate energy/performance bias per --cpu
        --pstate-epp <STR>            Set intel-pstate energy/performance pref per --cpu
    -P, --rapl-package <INT>          Target intel-rapl package, default 0
    -Z, --rapl-zone <INT>             Target intel-rapl sub-zone, default none
    -0, --rapl-c0-limit <WATTS>       Set intel-rapl c0 power limit per --rapl-{package,zone}
    -1, --rapl-c1-limit <WATTS>       Set intel-rapl c1 power limit per --rapl-{package,zone}
        --rapl-c0-window <SECS>       Set intel-rapl c0 time window per --rapl-{package,zone}
        --rapl-c1-winodw <SECS>       Set intel-rapl c1 time window per --rapl-{package,zone}
        --drm-i915 <IDS>              Target i915 card ids or pci ids, default all, ex. 0,1,3-5
        --drm-i915-min <HZ>           Set i915 min frequency per --drm-i915, ex. 1200 or 1.2ghz
        --drm-i915-max <HZ>           Set i915 max frequency per --drm-i915, ex. 1200 or 1.2ghz
        --drm-i915-boost <HZ>         Set i915 boost frequency per --drm-i915, ex. 1200 or 1.2ghz
        --nvml <IDS>                  Target nvidia card ids or pci ids, default all, ex. 0,1,3-5
        --nvml-gpu-freq <HZ[,HZ]>     Set nvidia gpu min,max frequency per --nvml, ex. 800,1.2ghz
        --nvml-gpu-freq-reset         Reset nvidia gpu frequency to default per --nvml
        --nvml-power-limit <WATTS>    Set nvidia card power limit per --nvml
    -h, --help                        Prints help information

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
        --nvml-gpu-freq 800,1.2ghz → KNOBS_NVML_GPU_FREQ=800,1.2ghz

    The KNOBS_LOG env var may be set to trace, debug, info, warn, or error (default).
```

## Example usage

---

### Set cpu min/max frequency

- set cpu minimum frequency → 800 MHz
- set cpu maximum frequency → 4.4 GHz

**For all CPUs**

```
knobs -n 800 -x 4.4ghz
```
...or with long args...
```
knobs --cpufreq-min 800 --cpufreq-max 4.4ghz
```

**For the first 4 CPUs only**

```
knobs -c 0-3 -n 800 -x 4.4ghz
```
...or with long args...
```
knobs --cpu 0-3 --cpufreq-min 800 --cpufreq-max 4.4ghz
```

---

### Set intel-pstate epb and epp

- set intel-pstate energy/performance bias → 6
- set intel-pstate energy/performance preference → balance_performance

**For all CPUs**

```
knobs --pstate-epb 6 --pstate-epp balance_performance
```

**For the first 4 CPUs only**

```
knobs -c 0-3 --pstate-epb 6 --pstate-epp balance_performance
```

---

### Set intel-rapl power constraints

- set package 0 constraint 0 (long-term) → 28 watts
- set package 0 constraint 1 (short-term) → 35 watts
- (0 is the default value for `--rapl-package`, while `--rapl-zone` has no default.)

```
knobs -0 28w -1 35w
```
...or with long args...
```
knobs --rapl-c0-limit 28w --rapl-c1-limit 35w
```

---

### Set nvidia min/max GPU frequency

- set gpu minimum frequency → 600 MHz
- set gpu maximum frequency - 2.2 GHz

**For all GPUs**

```
knobs --nvml-gpu-clock 600,2.2ghz
```

**For the first 2 GPUs only**

```
knobs --nvml 0,1 --nvml-gpu-clock 600,2.2ghz
```

