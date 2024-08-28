#!/usr/bin/env python
# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "pyqt6",
#     "matplotlib",
#     "numpy",
# ]
# ///

"""This program shows `hyperfine` benchmark results in a sequential way
in order to debug possible background interference, caching effects,
thermal throttling and similar effects.
"""

import argparse
import json

import matplotlib.pyplot as plt
import numpy as np


def moving_average(times, num_runs):
    times_padded = np.pad(
        times, (num_runs // 2, num_runs - 1 - num_runs // 2), mode="edge"
    )
    kernel = np.ones(num_runs) / num_runs
    return np.convolve(times_padded, kernel, mode="valid")


parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("file", help="JSON file with benchmark results")
parser.add_argument("--title", help="Plot Title")
parser.add_argument("-o", "--output", help="Save image to the given filename.")
parser.add_argument(
    "-w",
    "--moving-average-width",
    type=int,
    metavar="num_runs",
    help="Width of the moving-average window (default: N/5)",
)
parser.add_argument(
    "--no-moving-average",
    action="store_true",
    help="Do not show moving average curve",
)


args = parser.parse_args()

with open(args.file) as f:
    results = json.load(f)["results"]

for result in results:
    label = result["command"]
    times = result["times"]
    num = len(times)
    nums = range(num)

    plt.scatter(x=nums, y=times, marker=".")
    plt.ylim([0, None])
    plt.xlim([-1, num])

    if not args.no_moving_average:
        moving_average_width = (
            num // 5 if args.moving_average_width is None else args.moving_average_width
        )

        average = moving_average(times, moving_average_width)
        plt.plot(nums, average, "-")

if args.title:
    plt.title(args.title)

legend = []
for result in results:
    legend.append(result["command"])
    if not args.no_moving_average:
        legend.append("moving average")
plt.legend(legend)

plt.ylabel("Time [s]")

if args.output:
    plt.savefig(args.output)
else:
    plt.show()
