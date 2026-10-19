#!/usr/bin/env python
# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "matplotlib",
#     "pyqt6",
# ]
# ///

"""This program shows parametrized `hyperfine` benchmark results as an
errorbar plot."""

import argparse
import sys

import matplotlib.pyplot as plt

from plot_utils import METRICS, add_metric_argument, load_results


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    add_metric_argument(parser)
    parser.add_argument("file", help="JSON file with benchmark results", nargs="+")
    parser.add_argument(
        "--parameter-name",
        metavar="name",
        type=str,
        help="Deprecated; parameter names are now inferred from benchmark files",
    )
    parser.add_argument(
        "--log-x", help="Use a logarithmic x (parameter) axis", action="store_true"
    )
    parser.add_argument(
        "--log-y",
        "--log-time",
        dest="log_y",
        help="Use a logarithmic metric axis",
        action="store_true",
    )
    parser.add_argument(
        "--titles", help="Comma-separated list of titles for the plot legend"
    )
    parser.add_argument("-o", "--output", help="Save image to the given filename.")

    args = parser.parse_args()
    metric, datasets = load_results(parser, args.file, args.metric)
    metric_info = METRICS[metric]
    if args.parameter_name is not None:
        sys.stderr.write(
            "warning: --parameter-name is deprecated; names are inferred from "
            "benchmark results\n"
        )

    def die(msg):
        sys.stderr.write(f"fatal: {msg}\n")
        sys.exit(1)

    def extract_parameters(results):
        """Return `(parameter_name: str, parameter_values: List[float])`."""
        if not results:
            die("no benchmark data to plot")
        (names, values) = zip(*(unique_parameter(b) for b in results))
        names = frozenset(names)
        if len(names) != 1:
            die(
                f"benchmarks must all have the same parameter name, but found: {sorted(names)}"
            )
        return (next(iter(names)), list(values))

    def unique_parameter(benchmark):
        """Return the unique parameter `(name: str, value: float)`, or die."""
        params_dict = benchmark.get("parameters", {})
        if not params_dict:
            die("benchmarks must have exactly one parameter, but found none")
        if len(params_dict) > 1:
            die(
                f"benchmarks must have exactly one parameter, but found multiple: {sorted(params_dict)}"
            )
        [(name, value)] = params_dict.items()
        return (name, float(value["value"]))

    parameter_name = None

    for results in datasets:
        (this_parameter_name, parameter_values) = extract_parameters(results)
        if parameter_name is not None and this_parameter_name != parameter_name:
            die(
                f"files must all have the same parameter name, but found {parameter_name!r} vs. {this_parameter_name!r}"
            )
        parameter_name = this_parameter_name

        summaries = [b["summary"][metric] for b in results]
        means = [s["mean"] / metric_info.scale for s in summaries]
        stddevs = [
            s["stddev"] / metric_info.scale if s["stddev"] is not None else float("nan")
            for s in summaries
        ]

        plt.errorbar(x=parameter_values, y=means, yerr=stddevs, capsize=2)

    plt.xlabel(parameter_name)
    plt.ylabel(metric_info.axis_label)

    if args.log_y:
        plt.yscale("log")
    else:
        plt.ylim(0, None)

    if args.log_x:
        plt.xscale("log")

    if args.titles:
        plt.legend(args.titles.split(","))

    if args.output:
        plt.savefig(args.output)
    else:
        plt.show()


if __name__ == "__main__":
    main()
