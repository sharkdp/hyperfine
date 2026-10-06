"""Shared metric selection and units for analysis and plotting scripts."""

import json
from typing import NamedTuple


class Metric(NamedTuple):
    label: str
    unit: str = ""
    scale: float = 1

    @property
    def axis_label(self):
        return f"{self.label} [{self.unit}]" if self.unit else self.label


METRICS = {
    "time_wall_clock": Metric("Wall-clock time", "s"),
    "time_cpu": Metric("Total CPU time", "s"),
    "time_user": Metric("User CPU time", "s"),
    "time_system": Metric("System CPU time", "s"),
    "memory_peak_resident": Metric("Peak resident memory", "MiB", 1024**2),
    "cpu_cycles": Metric("CPU cycles"),
    "instructions": Metric("Completed CPU instructions"),
    "cache_references": Metric("Cache references"),
    "cache_misses": Metric("Cache misses"),
    "branch_misses": Metric("Mispredicted branches"),
}


def add_metric_argument(parser):
    parser.add_argument(
        "--metric",
        choices=METRICS,
        help="Metric to analyze (default: primary_metric from the JSON export, "
        "or time_wall_clock if unspecified). "
        "Times are in seconds; memory is in MiB; hardware counters are unscaled counts.",
    )


def validate_metric(parser, results, metric):
    for result in results:
        if metric not in result["summary"]:
            parser.error(
                f"metric {metric!r} is unavailable for "
                f"{result.get('name', result['command'])!r}"
            )


def load_results(parser, filenames, metric=None):
    exports = []
    for filename in filenames:
        with open(filename, encoding="utf-8") as f:
            export = json.load(f)
        if not export["results"]:
            parser.error(f"no benchmark results in {filename}")
        exports.append(export)

    if metric is None:
        primary_metrics = {
            export.get("primary_metric", "time_wall_clock") for export in exports
        }
        if len(primary_metrics) != 1:
            parser.error(
                "input files use different primary metrics; select one with --metric"
            )
        metric = primary_metrics.pop()
    if metric not in METRICS:
        parser.error(f"unknown metric {metric!r}; select one with --metric")

    results = [export["results"] for export in exports]
    for benchmarks in results:
        validate_metric(parser, benchmarks, metric)
    return metric, results
