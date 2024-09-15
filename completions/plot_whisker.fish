set command plot_whisker.py

complete -c $command -f

complete -c $command \
    -s h \
    -l help \
    -d 'Show help'

complete -c $command \
    -l title \
    -d 'Specify a plot title'

complete -c $command \
    -a 'median\tdefault' \
    -l sort-by \
    -d 'Specify a sorting method'

complete -c $command \
    -l labels \
    -d 'Specify comma separated entries for the legend'

complete -c $command \
    -s o \
    -l output \
    -d 'Specify an output file' \
    -F -r
