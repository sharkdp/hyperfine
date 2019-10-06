This folder contains scripts that can be used in combination with hyperfines `--export-json` option.

### Pre-requisites

To make theses scripts work, you will need matplotlib installed. Just execute the following commands:

```bash
pip install matplotlib

# If you're using python 3
pip3 install matplotlib
```

### Example:

``` bash
> hyperfine 'sleep 0.020' 'sleep 0.021' 'sleep 0.022' --export-json sleep.json
> python plot_benchmark_results.py sleep.json
```
