#!/usr/bin/env python
# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "scipy",
# ]
# ///

"""This script performs Welch's t-test on a JSON export file with two
benchmark results to test for a difference in their population means."""

import argparse
import math

from scipy import stats

from plot_utils import METRICS, add_metric_argument, load_results


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("file", help="JSON file with two benchmark results")
    add_metric_argument(parser)
    args = parser.parse_args()

    metric, [results] = load_results(parser, [args.file], args.metric)

    if len(results) != 2:
        parser.error("the input file must contain exactly two benchmarks")

    a, b = (x.get("name", x["command"]) for x in results[:2])
    X, Y = ([m[metric]["value"] for m in x["measurements"]] for x in results[:2])
    if min(len(X), len(Y)) < 2:
        parser.error("Welch's t-test requires at least two measurements per benchmark")

    print(f"Metric: {METRICS[metric].label}")
    print(f"Command 1: {a}")
    print(f"Command 2: {b}\n")

    t, p = stats.ttest_ind(X, Y, equal_var=False)
    if not math.isfinite(p):
        parser.error("Welch's t-test is undefined for these measurements")
    th = 0.05
    dispose = p < th
    print(f"t = {t:.3}, p = {p:.3}")
    print()

    if dispose:
        print(
            f"A statistically significant difference in mean value was detected (p < {th})."
        )
    else:
        print(
            f"No statistically significant difference in mean value was detected (p >= {th})."
        )


if __name__ == "__main__":
    main()
