#!/usr/bin/env python
# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "numpy",
# ]
# ///

import argparse

import numpy as np

from plot_utils import METRICS, add_metric_argument, load_results


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("file", help="JSON file with benchmark results")
    add_metric_argument(parser)
    parser.add_argument(
        "--time-unit",
        help="Display unit for time metrics (default: second).",
        choices=["second", "millisecond"],
    )
    args = parser.parse_args()

    metric, [results] = load_results(parser, [args.file], args.metric)
    description = METRICS[metric]
    unit, scale = description.unit, description.scale
    if args.time_unit is not None:
        if not metric.startswith("time_"):
            parser.error("--time-unit can only be used with a time metric")
        if args.time_unit == "millisecond":
            unit, scale = "ms", 0.001

    def format_value(value):
        suffix = f" {unit}" if unit else ""
        return f"{value:.3f}{suffix}"

    print(f"Metric: {description.label}\n")
    for result in results:
        summary = result["summary"][metric]
        values = [m[metric]["value"] / scale for m in result["measurements"]]

        p05, p25, p75, p95 = np.percentile(values, [5, 25, 75, 95])

        iqr = p75 - p25

        print(f"Command '{result.get('name', result['command'])}'")
        print(f"  runs:   {summary['count']:8d}")
        for statistic in ["mean", "stddev", "median", "min", "max"]:
            value = summary[statistic]
            formatted = "N/A" if value is None else format_value(value / scale)
            print(f"  {statistic + ':':<8}{formatted:>8}")
        print()
        print("  percentiles:")
        print(f"     P_05 .. P_95:    {format_value(p05)} .. {format_value(p95)}")
        print(
            f"     P_25 .. P_75:    {format_value(p25)} .. {format_value(p75)}"
            f"  (IQR = {format_value(iqr)})"
        )
        print()


if __name__ == "__main__":
    main()
