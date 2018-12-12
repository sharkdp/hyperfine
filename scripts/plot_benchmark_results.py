#!/usr/bin/python

import argparse
import json
import matplotlib.pyplot as plt

parser = argparse.ArgumentParser()
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
