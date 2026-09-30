function __inbox_recent_ids
    command inbox list -n 20 2>/dev/null | string split \n | string split -m1 ' '
end

complete -c inbox -f -n '__fish_use_subcommand' -a 'add edit list search review show delete trash restore tags doctor'
complete -c inbox -n '__fish_seen_subcommand_from show restore delete' -a '(__inbox_recent_ids)'
complete -c inbox -n '__fish_seen_subcommand_from delete' -a 'today range all'
complete -c inbox -n '__fish_seen_subcommand_from trash' -a 'empty'
