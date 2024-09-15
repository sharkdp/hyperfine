set command plot_histogram.py

complete -c $command -f

complete -c $command \
    -s h \
    -l help \
    -d 'Show help'

complete -c $command \
    -l title \
    -d 'Specify a plot title'

complete -c $command \
    -l labels \
    -d 'Specify comma separated entries for the legend'

complete -c $command \
    -a 'auto\tdefault' \
    -l bins \
    -d 'Specify a number of bins'

complete -c $command \
    -a '"upper center"\tdefault
        "lower center"
        "right"
        "left"
        "best"
        "upper left"
        "upper right"
        "lower left"
        "lower right"
        "center left"
        "center right"
        "center"' \
    -l legend-location \
    -d 'Specify a legend location' \
    -x

complete -c $command \
    -a 'bar\tdefault
            barstacked
            step
            stepfilled' \
    -l type \
    -d 'Specify a type of the histogram' \
    -x

complete -c $command \
    -s o \
    -l output \
    -d 'Specify an output file' \
    -F -r

complete -c $command \
    -l t-min \
    -d 'Specify a minimum time'

complete -c $command \
    -l t-max \
    -d 'Specify a maximum time'

complete -c $command \
    -l log-count \
    -d 'Specify a logarithmic y-axis for the event count'
