#!/usr/bin/env python
# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "numpy",
# ]
# ///

import argparse
import json

import numpy as np

parser = argparse.ArgumentParser()
parser.add_argument("file", help="JSON file with benchmark results")
args = parser.parse_args()

with open(args.file) as f:
    results = json.load(f)["results"]

commands = [b["command"] for b in results]
times = [b["times"] for b in results]

for command, ts in zip(commands, times):
    p05 = np.percentile(ts, 5)
    p25 = np.percentile(ts, 25)
    p75 = np.percentile(ts, 75)
    p95 = np.percentile(ts, 95)

    iqr = p75 - p25

    print(f"Command '{command}'")
    print(f"  runs:   {len(ts):8d}")
    print(f"  mean:   {np.mean(ts):8.3f} s")
    print(f"  stddev: {np.std(ts, ddof=1):8.3f} s")
    print(f"  median: {np.median(ts):8.3f} s")
    print(f"  min:    {np.min(ts):8.3f} s")
    print(f"  max:    {np.max(ts):8.3f} s")
    print()
    print("  percentiles:")
    print(f"     P_05 .. P_95:    {p05:.3f} s .. {p95:.3f} s")
    print(f"     P_25 .. P_75:    {p25:.3f} s .. {p75:.3f} s  (IQR = {iqr:.3f} s)")
    print()
