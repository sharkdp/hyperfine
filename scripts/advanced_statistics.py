#!/usr/bin/env python

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

    print("Command '{}'".format(command))
    print("  runs:   {:8d}".format(len(ts)))
    print("  mean:   {:8.3f} s".format(np.mean(ts)))
    print("  stddev: {:8.3f} s".format(np.std(ts, ddof=1)))
    print("  median: {:8.3f} s".format(np.median(ts)))
    print("  min:    {:8.3f} s".format(np.min(ts)))
    print("  max:    {:8.3f} s".format(np.max(ts)))
    print()
    print("  percentiles:")
    print("     P_05 .. P_95:    {:.3f} s .. {:.3f} s".format(p05, p95))
    print(
        "     P_25 .. P_75:    {:.3f} s .. {:.3f} s  "
        "(IQR = {:.3f} s)".format(p25, p75, iqr)
    )
    print()
