#!/usr/bin/python

"""This program shows `hyperfine` benchmark results as a box and whisker plot.

Quoting from the matplotlib documentation:
    The box extends from the lower to upper quartile values of the data, with
    a line at the median. The whiskers extend from the box to show the range
    of the data. Flier points are those past the end of the whiskers.
"""

import argparse
import json
import matplotlib.pyplot as plt

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("file", help="JSON file with benchmark results")
args = parser.parse_args()

with open(args.file) as f:
    results = json.load(f)["results"]

commands = [b["command"] for b in results]
times = [b["times"] for b in results]

plt.boxplot(times, labels=commands)
plt.ylabel("Time [s]")
plt.ylim(0, None)
plt.show()
