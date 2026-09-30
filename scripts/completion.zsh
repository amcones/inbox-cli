#compdef inbox
_inbox() {
  local -a commands ids
  commands=(
    'add:add an idea' 'edit:edit an idea' 'list:list ideas'
    'search:search ideas' 'review:review priority candidates' 'show:show an idea'
    'delete:move ideas to trash' 'trash:list or empty trash' 'restore:restore an idea'
    'tags:list tags' 'doctor:check stored data'
  )
  if (( CURRENT == 2 )); then
    _describe 'command' commands
    return
  fi
  case $words[2] in
    show|restore)
      if (( CURRENT == 3 )); then
        ids=("${(@f)$(command inbox list -n 20 2>/dev/null | awk '{print $1}')}" )
        _values 'recent idea' $ids
      fi
      ;;
    delete)
      if (( CURRENT == 3 )); then
        ids=(today range all "${(@f)$(command inbox list -n 20 2>/dev/null | awk '{print $1}')}" )
        _values 'target' $ids
      fi
      ;;
    trash)
      (( CURRENT == 3 )) && _values 'action' empty
      ;;
  esac
}
compdef _inbox inbox
