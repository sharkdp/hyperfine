This folder contains scripts that can be used in combination with hyperfine's `--export-json` option.

### Example:

```bash
hyperfine 'sleep 0.020' 'sleep 0.021' 'sleep 0.022' --export-json sleep.json
uv run plot_whisker.py sleep.json
```

All plotting and analysis scripts use the JSON export's `primary_metric` by default.
Pass `--metric` to select a different metric. Supported metrics are `time_wall_clock`,
`time_cpu`, `time_user`, `time_system`, `memory_peak_resident`, `cpu_cycles`,
`instructions`, `cache_references`, `cache_misses`, and `branch_misses`, provided the
selected metric is available in every benchmark being analyzed. For example:

```bash
uv run plot_histogram.py benchmark.json --metric instructions
```

Time is displayed in seconds, memory in MiB, and counters as unscaled counts.
Multi-file plots require matching primary metrics unless `--metric` explicitly selects
one metric across all files. Exports without `primary_metric` fall back to wall-clock
time; the per-run `measurements` JSON schema is still required.

For descriptive statistics or a Welch's t-test comparing exactly two benchmark results:

```bash
uv run advanced_statistics.py benchmark.json --metric time_cpu --time-unit millisecond
uv run welch_ttest.py comparison.json --metric memory_peak_resident
```

`advanced_statistics.py` accepts `--time-unit second` or `--time-unit millisecond`
for time metrics only. Both analysis scripts also accept the default primary metric
without `--metric`.

### Prerequisites

Install [`uv`](https://docs.astral.sh/uv/getting-started/installation/) and run the
commands above from this directory.

The scripts declare their dependencies using [PEP 723](https://peps.python.org/pep-0723/)
inline script metadata. `uv run` automatically installs these dependencies in an
isolated environment before running the script.
