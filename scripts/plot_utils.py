"""Shared metric options for the plotting scripts."""

METRICS = {
    "time_wall_clock": ("Time [s]", 1),
    "time_user": ("User CPU time [s]", 1),
    "time_system": ("System CPU time [s]", 1),
    "memory_peak_resident": ("Peak resident memory [MiB]", 1024**2),
    "cpu_cycles": ("CPU cycles", 1),
    "instructions": ("Completed CPU instructions", 1),
    "cache_references": ("Cache references", 1),
    "cache_misses": ("Cache misses", 1),
    "branch_misses": ("Mispredicted branches", 1),
}


def add_metric_argument(parser):
    parser.add_argument(
        "--metric",
        choices=METRICS,
        default="time_wall_clock",
        help="Metric to plot (default: time_wall_clock). "
        "Times are in seconds; memory is in MiB; hardware counters are unscaled counts.",
    )


def validate_metric(parser, results, metric):
    for result in results:
        if metric not in result["summary"]:
            parser.error(
                f"metric {metric!r} is unavailable for "
                f"{result.get('name', result['command'])!r}"
            )
