_inbox() {
    local cur prev command command_index i kind ids
    local -a global_args candidates
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]:-}"
    command=""
    command_index=0
    global_args=()
    for ((i=1; i<COMP_CWORD; i++)); do
        case "${COMP_WORDS[i]}" in
            --dir|--lang)
                global_args+=("${COMP_WORDS[i]}")
                if ((i + 1 < COMP_CWORD)); then global_args+=("${COMP_WORDS[i+1]}"); ((i++)); fi
                ;;
            --dir=*|--lang=*) global_args+=("${COMP_WORDS[i]}") ;;
            add|edit|list|search|review|show|delete|trash|restore|backup|tags|doctor|help)
                if [[ -z "$command" ]]; then command="${COMP_WORDS[i]}"; command_index=$i; fi ;;
        esac
    done
    case "$prev" in
        --dir) COMPREPLY=( $(compgen -d -- "$cur") ); return ;;
        --from) COMPREPLY=( $(compgen -d -- "$cur") ); return ;;
        --lang) COMPREPLY=( $(compgen -W "auto en zh" -- "$cur") ); return ;;
        -t|--tag) COMPREPLY=( $(compgen -W "$(command inbox "${global_args[@]}" __complete tags 2>/dev/null)" -- "$cur") ); return ;;
        --sort) COMPREPLY=( $(compgen -W "time priority" -- "$cur") ); return ;;
        -n|--limit) COMPREPLY=( $(compgen -W "5 10 20 50 100" -- "$cur") ); return ;;
        -m) COMPREPLY=( $(compgen -W "1 2 3 4 5 6 7 8 9 10 11 12" -- "$cur") ); return ;;
        -d) COMPREPLY=( $(compgen -W "$(seq 1 31)" -- "$cur") ); return ;;
        -y)
            [[ "$command" == delete && "${COMP_WORDS[command_index+1]:-}" == range ]] && COMPREPLY=( $(compgen -W "$(date +%Y)" -- "$cur") )
            return ;;
    esac
    if [[ -z "$command" ]]; then
        candidates=(add edit list search review show delete trash restore backup tags doctor help --dir --lang -V --version)
        COMPREPLY=( $(compgen -W "${candidates[*]}" -- "$cur") ); return
    fi
    if [[ "$command" == delete && "${COMP_WORDS[command_index+1]:-}" == range ]]; then
        if [[ "$prev" == -h || "${COMP_WORDS[COMP_CWORD-2]:-}" == -h ]]; then
            COMPREPLY=( $(compgen -W "0 6 8 9 12 18 24" -- "$cur") )
        else
            COMPREPLY=( $(compgen -W "-y -m -d -h --yes --dir --lang" -- "$cur") )
        fi
        return
    fi
    if ((COMP_CWORD == command_index + 1)); then
        case "$command" in
            show|edit) kind=active-ids ;;
            restore)
                if [[ "$cur" == -* ]]; then
                    COMPREPLY=( $(compgen -W "--from --dir --lang" -- "$cur") ); return
                fi
                kind=trash-ids ;;
            delete)
                ids="$(command inbox "${global_args[@]}" __complete active-ids 2>/dev/null)"
                COMPREPLY=( $(compgen -W "$ids today range all" -- "$cur") ); return ;;
            trash) COMPREPLY=( $(compgen -W "empty" -- "$cur") ); return ;;
            backup) COMPREPLY=( $(compgen -d -- "$cur") ); return ;;
        esac
        if [[ -n "${kind:-}" ]]; then
            COMPREPLY=( $(compgen -W "$(command inbox "${global_args[@]}" __complete "$kind" 2>/dev/null)" -- "$cur") ); return
        fi
    fi
    case "$command" in
        add) candidates=(-t --tag --dir --lang) ;;
        edit) candidates=(-t --tag --clear-tags --dir --lang) ;;
        list|search) candidates=(-t --tag --any --sort -n --limit --dir --lang) ;;
        review) candidates=(-n --limit --dir --lang) ;;
        show) candidates=(--no-track --dir --lang) ;;
        delete)
            case "${COMP_WORDS[command_index+1]:-}" in
                today|all) candidates=(--yes --dir --lang) ;;
                *) candidates=(-t --tag --dir --lang) ;;
            esac ;;
        trash) [[ "${COMP_WORDS[command_index+1]:-}" == empty ]] && candidates=(--yes --dir --lang) || candidates=(--dir --lang) ;;
        restore) candidates=(--from --yes --dir --lang) ;;
        backup|tags|doctor|help) candidates=(--dir --lang) ;;
    esac
    COMPREPLY=( $(compgen -W "${candidates[*]}" -- "$cur") )
}
complete -o bashdefault -o default -F _inbox inbox
