This folder contains scripts that can be used in combination with hyperfines `--export-json` option.

### Example:

``` bash
> hyperfine 'sleep 0.020' 'sleep 0.021' 'sleep 0.022' --export-json sleep.json
> python plot_benchmark_results.py sleep.json
```
