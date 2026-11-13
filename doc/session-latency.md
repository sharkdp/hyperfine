# Measuring latency inside a session

A search client can handle dozens of keystrokes in a single process. Use
Hyperfine to measure the whole session, including startup. To measure the time
from a keystroke to its accepted response, record both timestamps in the client
using the same monotonic clock.

## Collecting both measurements

Save this small example as `session.py`. The sleep stands in for a request.
Replace it with the operation whose response the client waits for.

```python
import json
import time

for request in range(12):
    start = time.perf_counter_ns()
    time.sleep(0.005)
    elapsed_ms = (time.perf_counter_ns() - start) / 1_000_000
    print(json.dumps({"request": request, "latency_ms": elapsed_ms}))
```

Run this in a fresh directory with a Unix shell and Python 3. Each of the six
sessions writes a separate request log. Shell redirection is explicit here.

```sh
hyperfine --runs 6 --export-json sessions.json -S \
  'python3 session.py > requests-${HYPERFINE_ITERATION}.jsonl'
```

The wall-clock measurements in `sessions.json` describe six whole sessions.
Compute the nearest-rank p95 from the 72 individual request durations:

```python
import json
import math
from pathlib import Path

latencies = sorted(
    json.loads(line)["latency_ms"]
    for run in range(6)
    for line in Path(f"requests-{run}.jsonl").read_text().splitlines()
)
assert len(latencies) == 72
print(f"Request p95: {latencies[math.ceil(0.95 * len(latencies)) - 1]:.2f} ms")
```

## Example from a local search client

A typing replay compared two policies for one persistent worker: immediately
submit up to eight requests, or allow one request in flight and keep only the
latest pending input. Both used pipes with zero debounce. Twelve held-out input
sequences ran six times per policy, alternating policy order between rounds.
Each final response had to match an independent search, including result order.

| Metric | Up to eight in flight | One in flight |
| --- | ---: | ---: |
| Median whole session, including typing intervals | 3.326 s | 3.248 s |
| Median final keystroke to client acceptance | 5.33 ms | 3.88 ms |
| p95 final keystroke to client acceptance | 48.89 ms | 9.86 ms |

The whole-session median fell by about 2%, while the client p95 fell by 80%.
The timings end when the client accepts the result, before any screen drawing.
These figures describe one application index and one host. The sleep example
above reproduces the collection method.
