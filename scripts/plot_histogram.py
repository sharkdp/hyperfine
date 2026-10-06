#!/usr/bin/env python
# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "matplotlib",
#     "pyqt6",
#     "numpy",
# ]
# ///

"""This program shows `hyperfine` benchmark results as a histogram."""

import argparse
import json

import matplotlib.pyplot as plt
import numpy as np

from plot_utils import METRICS, add_metric_argument, validate_metric

parser = argparse.ArgumentParser(description=__doc__)
add_metric_argument(parser)
parser.add_argument("file", help="JSON file with benchmark results")
parser.add_argument("--title", help="Plot title")
parser.add_argument(
    "--labels", help="Comma-separated list of entries for the plot legend"
)
parser.add_argument("--bins", help="Number of bins (default: auto)")
parser.add_argument(
    "--legend-location",
    help="Location of the legend on plot (default: upper center)",
    choices=[
        "upper center",
        "lower center",
        "right",
        "left",
        "best",
        "upper left",
        "upper right",
        "lower left",
        "lower right",
        "center left",
        "center right",
        "center",
    ],
    default="upper center",
)
parser.add_argument(
    "--type", help="Type of histogram (*bar*, barstacked, step, stepfilled)"
)
parser.add_argument("-o", "--output", help="Save image to the given filename.")
parser.add_argument(
    "--min",
    "--t-min",
    dest="value_min",
    type=float,
    help="Minimum metric value to display",
)
parser.add_argument(
    "--max",
    "--t-max",
    dest="value_max",
    type=float,
    help="Maximum metric value to display",
)
parser.add_argument(
    "--log-count",
    help="Use a logarithmic y-axis for the event count",
    action="store_true",
)

args = parser.parse_args()
metric_label, metric_scale = METRICS[args.metric]

with open(args.file) as f:
    results = json.load(f)["results"]
validate_metric(parser, results, args.metric)

if args.labels:
    labels = args.labels.split(",")
else:
    labels = [b.get("name", b["command"]) for b in results]
all_values = [
    [m[args.metric]["value"] / metric_scale for m in b["measurements"]] for b in results
]

value_min = (
    args.value_min
    if args.value_min is not None
    else np.min(list(map(np.min, all_values)))
)
value_max = (
    args.value_max
    if args.value_max is not None
    else np.max(list(map(np.max, all_values)))
)

bins = int(args.bins) if args.bins else "auto"
histtype = args.type if args.type else "bar"

plt.figure(figsize=(10, 5))
plt.hist(
    all_values,
    label=labels,
    bins=bins,
    histtype=histtype,
    range=(value_min, value_max),
)
plt.legend(
    loc=args.legend_location,
    fancybox=True,
    shadow=True,
    prop={"size": 10, "family": ["Source Code Pro", "Fira Mono", "Courier New"]},
)

plt.xlabel(metric_label)
if args.title:
    plt.title(args.title)

if args.log_count:
    plt.yscale("log")
else:
    plt.ylim(0, None)

if args.output:
    plt.savefig(args.output, dpi=600)
else:
    plt.show()
