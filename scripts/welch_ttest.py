#!/usr/bin/python

import argparse
import json
import sys
from scipy import stats

parser = argparse.ArgumentParser()
parser.add_argument("file", help="JSON file with two benchmark results")
args = parser.parse_args()

with open(args.file) as f:
    results = json.load(f)["results"]

if len(results) != 2:
    print("The input file has to contain exactly two benchmarks")
    sys.exit(1)

a, b = [x["command"] for x in results[:2]]
X, Y = [x["times"] for x in results[:2]]

print("Command 1: {}".format(a))
print("Command 2: {}\n".format(b))

t, p = stats.ttest_ind(X, Y, equal_var=False)
th = 0.05
dispose = p < th
print("t = {}, p = {}".format(t, p))
print("p < {} = {}\n".format(th, dispose))
if dispose:
    print("There is a difference between the two benchmarks.")
else:
    print("The two benchmarks are almost the same.")
