# knobs

Display and set Linux system tunables:

- cpu: online/offline
- cpufreq: governor, min/max frequencies
- intel-pstate: epb, epp
- intel-rapl: power limit, time window per zone/subzone/constraint
- drm
  - i915: min/max/boost frequencies
- nvml
  - nvidia gpu clock min/max frequency, power limit
  - requires nvidia management library at runtime, usually installed with the proprietary driver
  - enabled via the `nvml` feature flag

## Output

```
 CPU  Online  Governor   Cur      Min      Max      CPU min  CPU max
 ---  ------  --------   ---      ---      ---      -------  -------
 0    •       powersave  4.4 GHz  800 MHz  4.4 GHz  800 MHz  5.0 GHz
 1    true    powersave  4.4 GHz  800 MHz  4.4 GHz  800 MHz  5.0 GHz
 2    true    powersave  4.4 GHz  800 MHz  4.4 GHz  800 MHz  5.0 GHz
 3    true    powersave  4.3 GHz  800 MHz  4.4 GHz  800 MHz  5.0 GHz
 4    true    powersave  4.4 GHz  800 MHz  4.4 GHz  800 MHz  5.0 GHz
 5    true    powersave  4.3 GHz  800 MHz  4.4 GHz  800 MHz  5.0 GHz
 6    true    powersave  4.4 GHz  800 MHz  4.4 GHz  800 MHz  5.0 GHz
 7    true    powersave  4.4 GHz  800 MHz  4.4 GHz  800 MHz  5.0 GHz
 8    true    powersave  4.1 GHz  800 MHz  4.4 GHz  800 MHz  5.0 GHz
 9    true    powersave  4.1 GHz  800 MHz  4.4 GHz  800 MHz  5.0 GHz
 10   true    powersave  4.4 GHz  800 MHz  4.4 GHz  800 MHz  5.0 GHz
 11   true    powersave  4.2 GHz  800 MHz  4.4 GHz  800 MHz  5.0 GHz

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
knobs 0.2.0

USAGE:
    knobs [OPTIONS]

OPTIONS:
        --show-cpu                       Print cpu and cpufreq values
        --show-pstate                    Print intel_pstate values
        --show-drm                       Print drm values
        --show-nvml                      Print nvidia management values
    -q, --quiet                          Do not print values
    -c, --cpu <INDICES>                  Target cpu ids, default all, ex. 0,1,3-5
    -o, --cpu-on <0|1>                   Set cpu online status per --cpu
    -O, --cpu-on-each <TOGGLES>          Set cpu online status, ex. 10-1 → 0=ON 1=OFF 2=SKIP 3=ON
    -g, --cpufreq-gov <NAME>             Set cpufreq governor per --cpu
    -n, --cpufreq-min <FREQ>             Set cpufreq min freq per --cpu, ex. 1200 or 1.2ghz
    -x, --cpufreq-max <FREQ>             Set cpufreq max freq per --cpu, ex. 1200 or 1.2ghz
        --pstate-epb <0-15>              Set intel_pstate energy/performance bias per --cpu
        --pstate-epp <NAME>              Set intel_pstate energy/performance pref per --cpu
        --drm-i915 <INDICES>             Target i915 card ids or pci ids, default all, ex. 0,1,3-5
        --drm-i915-min <FREQ>            Set i915 min frequency per --drm-i915, ex. 1200 or 1.2ghz
        --drm-i915-max <FREQ>            Set i915 max frequency per --drm-i915, ex. 1200 or 1.2ghz
        --drm-i915-boost <FREQ>          Set i915 boost frequency per --drm-i915, ex. 1200 or 1.2ghz
        --nvml <INDICES>                 Target nvidia card ids or pci ids, default all, ex. 0,1,3-5
        --nvml-gpu-freq <FREQ[,FREQ]>    Set nvidia gpu min,max frequency per --nvml, ex. 800,1.2ghz
        --nvml-gpu-freq-reset            Reset nvidia gpu frequency to default per --nvml
        --nvml-power-limit <POWER>       Set nvidia card power limit per --nvml, ex. 260 or 0.26kw
    -h, --help                           Prints help information

    All present and supported subsystems are printed unless the --show-* or --quiet flags are used.

    The following special values and units are handled uniformly for all arguments.

        INDICES   A comma-delimited sequence of integers and/or integer ranges.

        TOGGLES   An enumeration of 0 (deactivate), 1 (activate) or - (skip) characters, where the
                  character is an action, and the character's position is an ID on which to act.

          FREQ*     Default: megahertz when unspecified
                  Supported: hz/h - khz/k - mhz/m - ghz/g - thz/t

         POWER*     Default: watts when unspecified
                  Supported: mw/m - w - kw/k

        * Floating point values may be given for these units.

    All flags may be expressed as env vars. For example:

        --show-cpu                 → KNOBS_SHOW_CPU=1
        --cpu 1,3-5                → KNOBS_CPU=1,3-5
        --nvml-gpu-freq 800,1.2ghz → KNOBS_NVML_GPU_FREQ=800,1.2ghz

    The KNOBS_LOG env var may be set to trace, debug, info, warn, or error (default).
```

## Example usage

---

### Set cpufreq min/max frequency

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

### Set intel_pstate epb

**For all CPUs**

```
knobs --pstate-epb 6
```

**For the first 4 CPUs only**

```
knobs -c 0-3 --pstate-epb 6
```
...or with long args...
```
knobs --cpu 0-3 --pstate-epb 6
```

---

### Set nvidia min/max GPU frequency

**For all GPUs**

```
knobs --nvml-gpu-clock 600,2.2ghz
```

**For the first 2 GPUs only**

```
knobs --nvml 0,1 --nvml-gpu-clock 600,2.2ghz
```

