This folder contains scripts that can be used in combination with hyperfines `--export-json` option.

### Example:

```bash
hyperfine 'sleep 0.020' 'sleep 0.021' 'sleep 0.022' --export-json sleep.json
python plot_whisker.py sleep.json
```

### Pre-requisites

To make these scripts work, you will need to install `numpy`, `matplotlib` and `scipy`. Install them via
your package manager or `pip`:

```bash
pip install numpy matplotlib scipy  # pip3, if you are using python3
```
