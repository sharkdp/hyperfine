This folder contains scripts that can be used in combination with hyperfines `--export-json` option.

### Example:

```bash
hyperfine 'sleep 0.020' 'sleep 0.021' 'sleep 0.022' --export-json sleep.json
uv run plot_whisker.py sleep.json
```

### Pre-requisites

Install [`uv`](https://docs.astral.sh/uv/getting-started/installation/) and run the
commands above from this directory.

The scripts declare their dependencies using [PEP 723](https://peps.python.org/pep-0723/)
inline script metadata. `uv run` automatically installs these dependencies in an
isolated environment before running the script.
