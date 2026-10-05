#!/usr/bin/env python
# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "matplotlib",
#     "pyqt6",
#     "numpy",
# ]
# ///

"""
This script shows `hyperfine` benchmark results as a bar plot grouped by command.
Note all the input files must contain results for all commands.
"""

import argparse
import json
import pathlib

import matplotlib.pyplot as plt
import numpy as np

from plot_utils import METRICS, add_metric_argument, validate_metric

parser = argparse.ArgumentParser(description=__doc__)
add_metric_argument(parser)
parser.add_argument(
    "files", nargs="+", type=pathlib.Path, help="JSON files with benchmark results"
)
parser.add_argument("--title", help="Plot Title")
parser.add_argument(
    "--benchmark-names", nargs="+", help="Names of the benchmark groups"
)
parser.add_argument("-o", "--output", help="Save image to the given filename")

args = parser.parse_args()
metric_label, metric_scale = METRICS[args.metric]

commands = None
data = []
inputs = []

if args.benchmark_names:
    assert len(args.files) == len(args.benchmark_names), (
        "Number of benchmark names must match the number of input files."
    )

for i, filename in enumerate(args.files):
    with open(filename) as f:
        results = json.load(f)["results"]
    validate_metric(parser, results, args.metric)
    benchmark_commands = [b.get("name", b["command"]) for b in results]
    if commands is None:
        commands = benchmark_commands
    else:
        assert commands == benchmark_commands, (
            f"Unexpected commands in {filename}: {benchmark_commands}, expected: {commands}"
        )
    data.append([b["summary"][args.metric]["mean"] / metric_scale for b in results])
    if args.benchmark_names:
        inputs.append(args.benchmark_names[i])
    else:
        inputs.append(filename.stem)

data = np.transpose(data)
x = np.arange(len(inputs))  # the label locations
width = 0.25  # the width of the bars

fig, ax = plt.subplots(layout="constrained")
fig.set_figheight(5)
fig.set_figwidth(10)
for i, command in enumerate(commands):
    offset = width * (i + 1)
    rects = ax.bar(x + offset, data[i], width, label=command)

ax.set_xticks(x + 0.5, inputs)
ax.grid(visible=True, axis="y")

if args.title:
    plt.title(args.title)
plt.xlabel("Benchmark")
plt.ylabel(metric_label)
plt.legend(title="Command")

if args.output:
    plt.savefig(args.output)
else:
    plt.show()
