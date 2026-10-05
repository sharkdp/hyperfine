#!/usr/bin/env python
# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "scipy",
# ]
# ///

"""This script performs Welch's t-test on a JSON export file with two
benchmark results to test for a difference in their population mean runtimes."""

import argparse
import json
import sys

from scipy import stats

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("file", help="JSON file with two benchmark results")
args = parser.parse_args()

with open(args.file) as f:
    results = json.load(f)["results"]

if len(results) != 2:
    print("The input file has to contain exactly two benchmarks")
    sys.exit(1)

a, b = (x.get("name", x["command"]) for x in results[:2])
X, Y = ([m["time_wall_clock"]["value"] for m in x["measurements"]] for x in results[:2])

print(f"Command 1: {a}")
print(f"Command 2: {b}\n")

t, p = stats.ttest_ind(X, Y, equal_var=False)
th = 0.05
dispose = p < th
print(f"t = {t:.3}, p = {p:.3}")
print()

if dispose:
    print(
        f"A statistically significant difference in mean runtime was detected (p < {th})."
    )
else:
    print(
        f"No statistically significant difference in mean runtime was detected (p >= {th})."
    )
