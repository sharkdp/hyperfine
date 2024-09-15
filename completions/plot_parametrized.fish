set command plot_parametrized.py

complete -c $command -f

complete -c $command \
    -s h \
    -l help \
    -d 'Show help'

complete -c $command \
    -l log-x \
    -d 'Use a logarithmic x axis'

complete -c $command \
    -l log-time \
    -d 'Use a logarithmic time axis'

complete -c $command \
    -l titles \
    -d 'Specify comma separated titles for the legend'

complete -c $command \
    -s o \
    -l output \
    -d 'Specify an output file' \
    -F -r
