#!/usr/bin/python

"""This program shows `hyperfine` benchmark results as a histogram."""

import argparse
import json
import numpy as np
import matplotlib.pyplot as plt

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("file", help="JSON file with benchmark results")
parser.add_argument("--title", help="Plot title")
args = parser.parse_args()

with open(args.file) as f:
    results = json.load(f)["results"]

commands = [b["command"] for b in results]
all_times = [b["times"] for b in results]

plt.hist(all_times,
         label=commands,
         bins="auto",
         histtype="bar",
         range=(np.min(all_times), np.max(all_times)))
plt.legend(prop={'family': ['Source Code Pro', 'Fira Mono', 'Courier New']})

plt.xlabel("Time [s]")
if args.title:
    plt.title(args.title)

plt.show()
