set command plot_progression.py

complete -c $command -f

complete -c $command \
    -s h \
    -l help \
    -d 'Show help'

complete -c $command \
    -l title \
    -d 'Specify a plot title'

complete -c $command \
    -s o \
    -l output \
    -d 'Specify an output file' \
    -F -r

complete -c $command \
    -s w \
    -l moving-average-width \
    -d 'Specify width of the moving-average window'

complete -c $command \
    -l no-moving-average \
    -d 'Do not show moving average curve'
