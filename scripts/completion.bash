_inbox() {
    local cur command ids
    cur="${COMP_WORDS[COMP_CWORD]}"
    command="${COMP_WORDS[1]:-}"
    if [[ $COMP_CWORD -eq 1 ]]; then
        COMPREPLY=( $(compgen -W "add edit list search review show delete trash restore tags doctor" -- "$cur") )
        return
    fi
    if [[ $COMP_CWORD -eq 2 && "$command" == delete ]]; then
        ids="$(command inbox list -n 20 2>/dev/null | awk '{print $1}') today range all"
        COMPREPLY=( $(compgen -W "$ids" -- "$cur") )
    elif [[ $COMP_CWORD -eq 2 && ( "$command" == show || "$command" == restore ) ]]; then
        ids="$(command inbox list -n 20 2>/dev/null | awk '{print $1}')"
        COMPREPLY=( $(compgen -W "$ids" -- "$cur") )
    fi
}
complete -F _inbox inbox
