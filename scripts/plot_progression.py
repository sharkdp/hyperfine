#!/usr/bin/env python

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

args = parser.parse_args()

with open(args.file) as f:
    results = json.load(f)["results"]

label = results[0]["command"]
times = results[0]["times"]
num = len(times)
nums = range(num)

plt.scatter(x=nums, y=times, marker=".", color="orange")
plt.ylim([0, None])
plt.xlim([-1, num])

moving_average_width = (
    num // 5 if args.moving_average_width is None else args.moving_average_width
)

moving_average = moving_average(times, moving_average_width)
plt.plot(nums, moving_average, "-", color="blue")

if args.title:
    plt.title(args.title)
plt.legend(labels=[label], loc="best", fontsize="medium")
plt.ylabel("Time [s]")
if args.output:
    plt.savefig(args.output)
else:
    plt.show()
