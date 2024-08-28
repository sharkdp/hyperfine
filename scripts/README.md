This folder contains scripts that can be used in combination with hyperfines `--export-json` option.

### Example:

```bash
hyperfine 'sleep 0.020' 'sleep 0.021' 'sleep 0.022' --export-json sleep.json
./plot_whisker.py sleep.json
```

### Pre-requisites

To make these scripts work, you will need `numpy`, `matplotlib` and `scipy`.

If you have a Python package manager that understands [PEP-723](https://peps.python.org/pep-0723/)
inline script requirements like [`uv`](https://github.com/astral-sh/uv) or [`pipx`](https://github.com/pypa/pipx),
you can directly run the scripts using

```bash
uv run plot_whisker.py sleep.json
```

Otherwise, install the dependencies via your system package manager or using `pip`:

```bash
pip install numpy matplotlib scipy  # pip3, if you are using python3
```
