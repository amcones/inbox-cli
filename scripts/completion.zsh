#compdef inbox

_inbox_values() {
  local kind=$1
  local -a global_args
  local i
  for ((i=2; i<CURRENT; i++)); do
    case $words[i] in
      --dir|--lang)
        global_args+=($words[i])
        (( i + 1 < CURRENT )) && global_args+=($words[i+1]) && ((i++))
        ;;
      --dir=*|--lang=*) global_args+=($words[i]) ;;
    esac
  done
  reply=("${(@f)$(command inbox $global_args __complete $kind 2>/dev/null)}")
}

_inbox() {
  local -a commands common range_common
  local command target
  commands=(
    'add:add an idea' 'edit:edit an idea' 'list:list ideas'
    'search:search ideas' 'review:review priority candidates' 'show:show an idea'
    'delete:delete ideas or tags' 'trash:list or empty trash' 'restore:restore an idea or backup'
    'backup:create a complete backup'
    'tags:list tags' 'doctor:check stored data' 'help:show help'
  )
  common=('--dir[use a data directory]:directory:_directories' '--lang[select language]:language:(auto en zh)')
  range_common=('--dir[use a data directory]:directory:_directories' '--lang[select language]:language:(auto en zh)')
  if (( CURRENT == 2 )); then
    _describe 'command' commands
    _values 'global option' --dir --lang -V --version
    return
  fi
  command=$words[2]
  # _arguments counts positions from the command name. Remove the subcommand
  # so its first positional specification applies to the word after it.
  words=("$words[1]" "${(@)words[3,-1]}")
  (( CURRENT-- ))
  case $command in
    add) _arguments $common '*'{-t,--tag}'[add a tag]:tag:_inbox_tags' '1:content:' ;;
    edit) _arguments $common '*'{-t,--tag}'[replace tags]:tag:_inbox_tags' '--clear-tags[remove all tags]' '1:idea ID:_inbox_active' '2:new content:' ;;
    list) _arguments $common '*'{-t,--tag}'[filter by tag]:tag:_inbox_tags' '--any[match any tag]' '--sort[sort order]:order:(time priority)' '(-n --limit)'{-n,--limit}'[maximum results]:count:(5 10 20 50 100)' ;;
    search) _arguments $common '*'{-t,--tag}'[filter by tag]:tag:_inbox_tags' '--any[match any tag]' '--sort[sort order]:order:(time priority)' '(-n --limit)'{-n,--limit}'[maximum results]:count:(5 10 20 50 100)' '1:query:' ;;
    review) _arguments $common '(-n --limit)'{-n,--limit}'[maximum results]:count:(5 10 20 50 100)' ;;
    show) _arguments $common '--no-track[do not count a view]' '1:idea ID:_inbox_active' ;;
    restore) _arguments $common '--from[restore a complete backup]:backup directory:_directories' '--yes[skip full restore confirmation]' '1:trashed idea ID:_inbox_trash' ;;
    backup) _arguments $common '1:backup directory:_directories' ;;
    trash)
      if [[ $words[2] == empty ]]; then
        _arguments $common '(-y --yes)'{-y,--yes}'[skip confirmation]' '1:action:(empty)'
      else
        _arguments $common '1:action:(empty)'
      fi ;;
    delete)
      target=$words[2]
      if [[ $target == range ]]; then
        _arguments $range_common '--yes[skip confirmation]' '-y[year]:year:' '-m[month]:month:({1..12})' '-d[day]:day:({1..31})' '-h[hour range]:start hour:({0..23}):end hour:({1..24})' '1:target:(range)'
      elif [[ $target == today || $target == all ]]; then
        _arguments $common '(-y --yes)'{-y,--yes}'[skip bulk confirmation]' '1:target:_inbox_delete_targets'
      else
        _arguments $common '*'{-t,--tag}'[remove a tag]:tag:_inbox_tags' '1:target:_inbox_delete_targets'
      fi ;;
    tags|doctor|help) _arguments $common ;;
    *) _describe 'command' commands ;;
  esac
}

_inbox_active() { local -a reply; _inbox_values active-ids; _describe 'active idea ID' reply }
_inbox_trash() { local -a reply; _inbox_values trash-ids; _describe 'trashed idea ID' reply }
_inbox_tags() { local -a reply; _inbox_values tags; _describe 'tag' reply }
_inbox_delete_targets() { local -a reply; _inbox_values active-ids; reply+=(today range all); _describe 'delete target' reply }

compdef _inbox inbox
