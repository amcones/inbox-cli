function __inbox_using_command
    set -l words (commandline -opc)
    contains -- $argv[1] $words[2..-1]
end

function __inbox_no_command
    set -l words (commandline -opc)
    for word in $words[2..-1]
        if contains -- $word add edit list search review show delete trash restore backup tags info doctor help
            return 1
        end
    end
    return 0
end

function __inbox_complete_values
    set -l words (commandline -opc)
    set -l global_args
    for i in (seq 2 (count $words))
        if contains -- $words[$i] --dir --lang
            if test (math $i + 1) -le (count $words)
                set -a global_args $words[$i] $words[(math $i + 1)]
            end
        else if string match -qr '^--(dir|lang)=' -- $words[$i]
            set -a global_args $words[$i]
        end
    end
    command inbox $global_args __complete $argv[1] 2>/dev/null
end

complete -c inbox -f
complete -c inbox -n __inbox_no_command -a 'add edit list search review show delete trash restore backup tags info doctor help'
complete -c inbox -n __inbox_no_command -l dir -r -d 'Data directory'
complete -c inbox -n __inbox_no_command -l lang -r -a 'auto en zh' -d 'Language'
complete -c inbox -n __inbox_no_command -s V -l version -d 'Show version'

complete -c inbox -n '__inbox_using_command show' -a '(__inbox_complete_values active-ids)'
complete -c inbox -n '__inbox_using_command edit' -a '(__inbox_complete_values active-ids)'
complete -c inbox -n '__inbox_using_command delete' -a '(__inbox_complete_values active-ids) today range all'
complete -c inbox -n '__inbox_using_command restore' -a '(__inbox_complete_values trash-ids)'
complete -c inbox -n '__inbox_using_command restore' -l from -r -d 'Backup directory'
complete -c inbox -n '__inbox_using_command restore' -l yes -d 'Skip full restore confirmation'
complete -c inbox -n '__inbox_using_command trash' -a 'empty'
complete -c inbox -n '__inbox_using_command backup; and not contains -- verify (commandline -opc)' -a 'verify' -F
complete -c inbox -n '__inbox_using_command backup; and contains -- verify (commandline -opc)' -F

for cmd in add edit list search delete
    complete -c inbox -n "__inbox_using_command $cmd" -s t -l tag -r -a '(__inbox_complete_values tags)' -d 'Tag'
end
for cmd in list search
    complete -c inbox -n "__inbox_using_command $cmd" -l any -d 'Match any tag'
    complete -c inbox -n "__inbox_using_command $cmd" -l sort -r -a 'time priority' -d 'Sort order'
end
for cmd in list search review
    complete -c inbox -n "__inbox_using_command $cmd" -s n -l limit -r -a '5 10 20 50 100' -d 'Result limit'
end
complete -c inbox -n '__inbox_using_command edit' -l clear-tags -d 'Remove all tags'
complete -c inbox -n '__inbox_using_command show' -l no-track -d 'Do not count a view'
complete -c inbox -n '__inbox_using_command delete' -l yes -d 'Skip confirmation'
complete -c inbox -n '__inbox_using_command trash' -l yes -d 'Skip confirmation'
complete -c inbox -n '__inbox_using_command delete; and contains -- range (commandline -opc)' -s y -r -d 'Year'
complete -c inbox -n '__inbox_using_command delete; and contains -- range (commandline -opc)' -s m -r -a '1 2 3 4 5 6 7 8 9 10 11 12' -d 'Month'
complete -c inbox -n '__inbox_using_command delete; and contains -- range (commandline -opc)' -s d -r -a '(seq 1 31)' -d 'Day'
complete -c inbox -n '__inbox_using_command delete; and contains -- range (commandline -opc)' -s h -r -a '0 6 8 9 12 18 24' -d 'Hour'

for cmd in add edit list search review show trash restore backup tags info doctor help
    complete -c inbox -n "__inbox_using_command $cmd" -l dir -r -d 'Data directory'
    complete -c inbox -n "__inbox_using_command $cmd" -l lang -r -a 'auto en zh' -d 'Language'
end
complete -c inbox -n '__inbox_using_command delete' -l dir -r -d 'Data directory'
complete -c inbox -n '__inbox_using_command delete' -l lang -r -a 'auto en zh' -d 'Language'
