This folder contains scripts that can be used in combination with hyperfine's `--export-json` option.

### Run directly from GitHub

Install [`uv`](https://docs.astral.sh/uv/getting-started/installation/), then
run one of the scripts:

```bash
hyperfine 'sleep 0.020' 'sleep 0.021' 'sleep 0.022' --export-json sleep.json
uvx --from 'git+https://github.com/sharkdp/hyperfine' \
  plot_whisker sleep.json
```

Replace `plot_whisker` with any of the following commands.

| Command | Description |
| --- | --- |
| `plot_whisker` | Compare benchmarks with a box and whisker plot |
| `plot_histogram` | Show the distribution of measurements |
| `plot_parametrized` | Plot results from a parameter scan |
| `plot_progression` | Show measurements in run order |
| `plot_benchmark_comparison` | Compare commands across multiple JSON exports |
| `advanced_statistics` | Print additional statistics and percentiles |
| `welch_ttest` | Compare two benchmarks using Welch's t-test |

By default, the plotting and analysis scripts use the metric that was selected
first in the actual hyperfine run (`--metrics` option).
Use `--metric` to select another metric:

```bash
uvx --from 'git+https://github.com/sharkdp/hyperfine' \
  plot_histogram benchmark.json --metric instructions
```
