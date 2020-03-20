#!/usr/bin/python

"""This program shows `hyperfine` benchmark results as a box and whisker plot.

Quoting from the matplotlib documentation:
    The box extends from the lower to upper quartile values of the data, with
    a line at the median. The whiskers extend from the box to show the range
    of the data. Flier points are those past the end of the whiskers.
"""

import argparse
import json
import numpy as np
import matplotlib.pyplot as plt
import matplotlib.cm as cm

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("file", help="JSON file with benchmark results")
parser.add_argument("--title", help="Plot Title")
args = parser.parse_args()

with open(args.file) as f:
    results = json.load(f)["results"]

commands = [b["command"] for b in results]
times = [b["times"] for b in results]

boxplot = plt.boxplot(times, vert=True, patch_artist=True)
cmap = plt.cm.get_cmap("rainbow")
colors = [cmap(val / len(times)) for val in range(len(times))]

for patch, color in zip(boxplot["boxes"], colors):
    patch.set_facecolor(color)

plt.title(args.title)
plt.legend(handles=boxplot["boxes"], labels=commands, loc="best", fontsize="medium")
plt.ylabel("Time [s]")
plt.show()
