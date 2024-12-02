#!/usr/bin/env python
# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "numpy",
# ]
# ///

import argparse
import json
from enum import Enum

import numpy as np


class Unit(Enum):
    SECOND = 1
    MILLISECOND = 2

    def factor(self):
        match self:
            case Unit.SECOND:
                return 1
            case Unit.MILLISECOND:
                return 1e3

    def __str__(self):
        match self:
            case Unit.SECOND:
                return "s"
            case Unit.MILLISECOND:
                return "ms"


parser = argparse.ArgumentParser()
parser.add_argument("file", help="JSON file with benchmark results")
parser.add_argument(
    "--time-unit",
    help="The unit of time.",
    default="second",
    action="store",
    choices=["second", "millisecond"],
    dest="unit",
)
args = parser.parse_args()

unit = Unit.MILLISECOND if args.unit == "millisecond" else Unit.SECOND
unit_str = str(unit)

with open(args.file) as f:
    results = json.load(f)["results"]

commands = [b["command"] for b in results]
times = [b["times"] for b in results]

for command, ts in zip(commands, times):
    ts = [t * unit.factor() for t in ts]

    p05 = np.percentile(ts, 5)
    p25 = np.percentile(ts, 25)
    p75 = np.percentile(ts, 75)
    p95 = np.percentile(ts, 95)

    iqr = p75 - p25

    print(f"Command '{command}'")
    print(f"  runs:   {len(ts):8d}")
    print(f"  mean:   {np.mean(ts):8.3f} {unit_str}")
    print(f"  stddev: {np.std(ts, ddof=1):8.3f} {unit_str}")
    print(f"  median: {np.median(ts):8.3f} {unit_str}")
    print(f"  min:    {np.min(ts):8.3f} {unit_str}")
    print(f"  max:    {np.max(ts):8.3f} {unit_str}")
    print()
    print("  percentiles:")
    print(f"     P_05 .. P_95:    {p05:.3f} {unit_str} .. {p95:.3f} {unit_str}")
    print(
        f"     P_25 .. P_75:    {p25:.3f} {unit_str} .. {p75:.3f} {unit_str}  (IQR = {iqr:.3f} {unit_str})"
    )
    print()
