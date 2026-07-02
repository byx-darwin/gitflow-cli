_gitflow-cli() {
    local i cur prev opts cmd
    COMPREPLY=()
    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
        cur="$2"
    else
        cur="${COMP_WORDS[COMP_CWORD]}"
    fi
    prev="$3"
    cmd=""
    opts=""

    for i in "${COMP_WORDS[@]:0:COMP_CWORD}"
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="gitflow__cli"
                ;;
            gitflow__cli,auth)
                cmd="gitflow__cli__subcmd__auth"
                ;;
            gitflow__cli,commit)
                cmd="gitflow__cli__subcmd__commit"
                ;;
            gitflow__cli,completions)
                cmd="gitflow__cli__subcmd__completions"
                ;;
            gitflow__cli,help)
                cmd="gitflow__cli__subcmd__help"
                ;;
            gitflow__cli,issue)
                cmd="gitflow__cli__subcmd__issue"
                ;;
            gitflow__cli,label)
                cmd="gitflow__cli__subcmd__label"
                ;;
            gitflow__cli,milestone)
                cmd="gitflow__cli__subcmd__milestone"
                ;;
            gitflow__cli,pipeline)
                cmd="gitflow__cli__subcmd__pipeline"
                ;;
            gitflow__cli,pr)
                cmd="gitflow__cli__subcmd__pr"
                ;;
            gitflow__cli,release)
                cmd="gitflow__cli__subcmd__release"
                ;;
            gitflow__cli,review)
                cmd="gitflow__cli__subcmd__review"
                ;;
            gitflow__cli,run)
                cmd="gitflow__cli__subcmd__run"
                ;;
            gitflow__cli,skills)
                cmd="gitflow__cli__subcmd__skills"
                ;;
            gitflow__cli__subcmd__auth,help)
                cmd="gitflow__cli__subcmd__auth__subcmd__help"
                ;;
            gitflow__cli__subcmd__auth,login)
                cmd="gitflow__cli__subcmd__auth__subcmd__login"
                ;;
            gitflow__cli__subcmd__auth,logout)
                cmd="gitflow__cli__subcmd__auth__subcmd__logout"
                ;;
            gitflow__cli__subcmd__auth,status)
                cmd="gitflow__cli__subcmd__auth__subcmd__status"
                ;;
            gitflow__cli__subcmd__auth,token)
                cmd="gitflow__cli__subcmd__auth__subcmd__token"
                ;;
            gitflow__cli__subcmd__auth__subcmd__help,help)
                cmd="gitflow__cli__subcmd__auth__subcmd__help__subcmd__help"
                ;;
            gitflow__cli__subcmd__auth__subcmd__help,login)
                cmd="gitflow__cli__subcmd__auth__subcmd__help__subcmd__login"
                ;;
            gitflow__cli__subcmd__auth__subcmd__help,logout)
                cmd="gitflow__cli__subcmd__auth__subcmd__help__subcmd__logout"
                ;;
            gitflow__cli__subcmd__auth__subcmd__help,status)
                cmd="gitflow__cli__subcmd__auth__subcmd__help__subcmd__status"
                ;;
            gitflow__cli__subcmd__auth__subcmd__help,token)
                cmd="gitflow__cli__subcmd__auth__subcmd__help__subcmd__token"
                ;;
            gitflow__cli__subcmd__commit,comment)
                cmd="gitflow__cli__subcmd__commit__subcmd__comment"
                ;;
            gitflow__cli__subcmd__commit,diff)
                cmd="gitflow__cli__subcmd__commit__subcmd__diff"
                ;;
            gitflow__cli__subcmd__commit,help)
                cmd="gitflow__cli__subcmd__commit__subcmd__help"
                ;;
            gitflow__cli__subcmd__commit,patch)
                cmd="gitflow__cli__subcmd__commit__subcmd__patch"
                ;;
            gitflow__cli__subcmd__commit,view)
                cmd="gitflow__cli__subcmd__commit__subcmd__view"
                ;;
            gitflow__cli__subcmd__commit__subcmd__help,comment)
                cmd="gitflow__cli__subcmd__commit__subcmd__help__subcmd__comment"
                ;;
            gitflow__cli__subcmd__commit__subcmd__help,diff)
                cmd="gitflow__cli__subcmd__commit__subcmd__help__subcmd__diff"
                ;;
            gitflow__cli__subcmd__commit__subcmd__help,help)
                cmd="gitflow__cli__subcmd__commit__subcmd__help__subcmd__help"
                ;;
            gitflow__cli__subcmd__commit__subcmd__help,patch)
                cmd="gitflow__cli__subcmd__commit__subcmd__help__subcmd__patch"
                ;;
            gitflow__cli__subcmd__commit__subcmd__help,view)
                cmd="gitflow__cli__subcmd__commit__subcmd__help__subcmd__view"
                ;;
            gitflow__cli__subcmd__help,auth)
                cmd="gitflow__cli__subcmd__help__subcmd__auth"
                ;;
            gitflow__cli__subcmd__help,commit)
                cmd="gitflow__cli__subcmd__help__subcmd__commit"
                ;;
            gitflow__cli__subcmd__help,completions)
                cmd="gitflow__cli__subcmd__help__subcmd__completions"
                ;;
            gitflow__cli__subcmd__help,help)
                cmd="gitflow__cli__subcmd__help__subcmd__help"
                ;;
            gitflow__cli__subcmd__help,issue)
                cmd="gitflow__cli__subcmd__help__subcmd__issue"
                ;;
            gitflow__cli__subcmd__help,label)
                cmd="gitflow__cli__subcmd__help__subcmd__label"
                ;;
            gitflow__cli__subcmd__help,milestone)
                cmd="gitflow__cli__subcmd__help__subcmd__milestone"
                ;;
            gitflow__cli__subcmd__help,pipeline)
                cmd="gitflow__cli__subcmd__help__subcmd__pipeline"
                ;;
            gitflow__cli__subcmd__help,pr)
                cmd="gitflow__cli__subcmd__help__subcmd__pr"
                ;;
            gitflow__cli__subcmd__help,release)
                cmd="gitflow__cli__subcmd__help__subcmd__release"
                ;;
            gitflow__cli__subcmd__help,review)
                cmd="gitflow__cli__subcmd__help__subcmd__review"
                ;;
            gitflow__cli__subcmd__help,run)
                cmd="gitflow__cli__subcmd__help__subcmd__run"
                ;;
            gitflow__cli__subcmd__help,skills)
                cmd="gitflow__cli__subcmd__help__subcmd__skills"
                ;;
            gitflow__cli__subcmd__help__subcmd__auth,login)
                cmd="gitflow__cli__subcmd__help__subcmd__auth__subcmd__login"
                ;;
            gitflow__cli__subcmd__help__subcmd__auth,logout)
                cmd="gitflow__cli__subcmd__help__subcmd__auth__subcmd__logout"
                ;;
            gitflow__cli__subcmd__help__subcmd__auth,status)
                cmd="gitflow__cli__subcmd__help__subcmd__auth__subcmd__status"
                ;;
            gitflow__cli__subcmd__help__subcmd__auth,token)
                cmd="gitflow__cli__subcmd__help__subcmd__auth__subcmd__token"
                ;;
            gitflow__cli__subcmd__help__subcmd__commit,comment)
                cmd="gitflow__cli__subcmd__help__subcmd__commit__subcmd__comment"
                ;;
            gitflow__cli__subcmd__help__subcmd__commit,diff)
                cmd="gitflow__cli__subcmd__help__subcmd__commit__subcmd__diff"
                ;;
            gitflow__cli__subcmd__help__subcmd__commit,patch)
                cmd="gitflow__cli__subcmd__help__subcmd__commit__subcmd__patch"
                ;;
            gitflow__cli__subcmd__help__subcmd__commit,view)
                cmd="gitflow__cli__subcmd__help__subcmd__commit__subcmd__view"
                ;;
            gitflow__cli__subcmd__help__subcmd__issue,add-label)
                cmd="gitflow__cli__subcmd__help__subcmd__issue__subcmd__add__subcmd__label"
                ;;
            gitflow__cli__subcmd__help__subcmd__issue,close)
                cmd="gitflow__cli__subcmd__help__subcmd__issue__subcmd__close"
                ;;
            gitflow__cli__subcmd__help__subcmd__issue,comment)
                cmd="gitflow__cli__subcmd__help__subcmd__issue__subcmd__comment"
                ;;
            gitflow__cli__subcmd__help__subcmd__issue,create)
                cmd="gitflow__cli__subcmd__help__subcmd__issue__subcmd__create"
                ;;
            gitflow__cli__subcmd__help__subcmd__issue,list)
                cmd="gitflow__cli__subcmd__help__subcmd__issue__subcmd__list"
                ;;
            gitflow__cli__subcmd__help__subcmd__issue,remove-label)
                cmd="gitflow__cli__subcmd__help__subcmd__issue__subcmd__remove__subcmd__label"
                ;;
            gitflow__cli__subcmd__help__subcmd__issue,reopen)
                cmd="gitflow__cli__subcmd__help__subcmd__issue__subcmd__reopen"
                ;;
            gitflow__cli__subcmd__help__subcmd__issue,view)
                cmd="gitflow__cli__subcmd__help__subcmd__issue__subcmd__view"
                ;;
            gitflow__cli__subcmd__help__subcmd__label,create)
                cmd="gitflow__cli__subcmd__help__subcmd__label__subcmd__create"
                ;;
            gitflow__cli__subcmd__help__subcmd__label,delete)
                cmd="gitflow__cli__subcmd__help__subcmd__label__subcmd__delete"
                ;;
            gitflow__cli__subcmd__help__subcmd__label,edit)
                cmd="gitflow__cli__subcmd__help__subcmd__label__subcmd__edit"
                ;;
            gitflow__cli__subcmd__help__subcmd__label,list)
                cmd="gitflow__cli__subcmd__help__subcmd__label__subcmd__list"
                ;;
            gitflow__cli__subcmd__help__subcmd__milestone,close)
                cmd="gitflow__cli__subcmd__help__subcmd__milestone__subcmd__close"
                ;;
            gitflow__cli__subcmd__help__subcmd__milestone,create)
                cmd="gitflow__cli__subcmd__help__subcmd__milestone__subcmd__create"
                ;;
            gitflow__cli__subcmd__help__subcmd__milestone,edit)
                cmd="gitflow__cli__subcmd__help__subcmd__milestone__subcmd__edit"
                ;;
            gitflow__cli__subcmd__help__subcmd__milestone,list)
                cmd="gitflow__cli__subcmd__help__subcmd__milestone__subcmd__list"
                ;;
            gitflow__cli__subcmd__help__subcmd__milestone,reopen)
                cmd="gitflow__cli__subcmd__help__subcmd__milestone__subcmd__reopen"
                ;;
            gitflow__cli__subcmd__help__subcmd__pipeline,jobs)
                cmd="gitflow__cli__subcmd__help__subcmd__pipeline__subcmd__jobs"
                ;;
            gitflow__cli__subcmd__help__subcmd__pipeline,logs)
                cmd="gitflow__cli__subcmd__help__subcmd__pipeline__subcmd__logs"
                ;;
            gitflow__cli__subcmd__help__subcmd__pipeline,report)
                cmd="gitflow__cli__subcmd__help__subcmd__pipeline__subcmd__report"
                ;;
            gitflow__cli__subcmd__help__subcmd__pipeline,status)
                cmd="gitflow__cli__subcmd__help__subcmd__pipeline__subcmd__status"
                ;;
            gitflow__cli__subcmd__help__subcmd__pr,checkout)
                cmd="gitflow__cli__subcmd__help__subcmd__pr__subcmd__checkout"
                ;;
            gitflow__cli__subcmd__help__subcmd__pr,close)
                cmd="gitflow__cli__subcmd__help__subcmd__pr__subcmd__close"
                ;;
            gitflow__cli__subcmd__help__subcmd__pr,comment)
                cmd="gitflow__cli__subcmd__help__subcmd__pr__subcmd__comment"
                ;;
            gitflow__cli__subcmd__help__subcmd__pr,create)
                cmd="gitflow__cli__subcmd__help__subcmd__pr__subcmd__create"
                ;;
            gitflow__cli__subcmd__help__subcmd__pr,list)
                cmd="gitflow__cli__subcmd__help__subcmd__pr__subcmd__list"
                ;;
            gitflow__cli__subcmd__help__subcmd__pr,merge)
                cmd="gitflow__cli__subcmd__help__subcmd__pr__subcmd__merge"
                ;;
            gitflow__cli__subcmd__help__subcmd__pr,ready)
                cmd="gitflow__cli__subcmd__help__subcmd__pr__subcmd__ready"
                ;;
            gitflow__cli__subcmd__help__subcmd__pr,reopen)
                cmd="gitflow__cli__subcmd__help__subcmd__pr__subcmd__reopen"
                ;;
            gitflow__cli__subcmd__help__subcmd__pr,sync)
                cmd="gitflow__cli__subcmd__help__subcmd__pr__subcmd__sync"
                ;;
            gitflow__cli__subcmd__help__subcmd__pr,view)
                cmd="gitflow__cli__subcmd__help__subcmd__pr__subcmd__view"
                ;;
            gitflow__cli__subcmd__help__subcmd__pr,wip)
                cmd="gitflow__cli__subcmd__help__subcmd__pr__subcmd__wip"
                ;;
            gitflow__cli__subcmd__help__subcmd__release,create)
                cmd="gitflow__cli__subcmd__help__subcmd__release__subcmd__create"
                ;;
            gitflow__cli__subcmd__help__subcmd__release,delete)
                cmd="gitflow__cli__subcmd__help__subcmd__release__subcmd__delete"
                ;;
            gitflow__cli__subcmd__help__subcmd__release,download)
                cmd="gitflow__cli__subcmd__help__subcmd__release__subcmd__download"
                ;;
            gitflow__cli__subcmd__help__subcmd__release,edit)
                cmd="gitflow__cli__subcmd__help__subcmd__release__subcmd__edit"
                ;;
            gitflow__cli__subcmd__help__subcmd__release,list)
                cmd="gitflow__cli__subcmd__help__subcmd__release__subcmd__list"
                ;;
            gitflow__cli__subcmd__help__subcmd__release,upload)
                cmd="gitflow__cli__subcmd__help__subcmd__release__subcmd__upload"
                ;;
            gitflow__cli__subcmd__help__subcmd__release,view)
                cmd="gitflow__cli__subcmd__help__subcmd__release__subcmd__view"
                ;;
            gitflow__cli__subcmd__help__subcmd__review,approve)
                cmd="gitflow__cli__subcmd__help__subcmd__review__subcmd__approve"
                ;;
            gitflow__cli__subcmd__help__subcmd__review,comment)
                cmd="gitflow__cli__subcmd__help__subcmd__review__subcmd__comment"
                ;;
            gitflow__cli__subcmd__help__subcmd__review,request-changes)
                cmd="gitflow__cli__subcmd__help__subcmd__review__subcmd__request__subcmd__changes"
                ;;
            gitflow__cli__subcmd__help__subcmd__review,submit)
                cmd="gitflow__cli__subcmd__help__subcmd__review__subcmd__submit"
                ;;
            gitflow__cli__subcmd__issue,add-label)
                cmd="gitflow__cli__subcmd__issue__subcmd__add__subcmd__label"
                ;;
            gitflow__cli__subcmd__issue,close)
                cmd="gitflow__cli__subcmd__issue__subcmd__close"
                ;;
            gitflow__cli__subcmd__issue,comment)
                cmd="gitflow__cli__subcmd__issue__subcmd__comment"
                ;;
            gitflow__cli__subcmd__issue,create)
                cmd="gitflow__cli__subcmd__issue__subcmd__create"
                ;;
            gitflow__cli__subcmd__issue,help)
                cmd="gitflow__cli__subcmd__issue__subcmd__help"
                ;;
            gitflow__cli__subcmd__issue,list)
                cmd="gitflow__cli__subcmd__issue__subcmd__list"
                ;;
            gitflow__cli__subcmd__issue,remove-label)
                cmd="gitflow__cli__subcmd__issue__subcmd__remove__subcmd__label"
                ;;
            gitflow__cli__subcmd__issue,reopen)
                cmd="gitflow__cli__subcmd__issue__subcmd__reopen"
                ;;
            gitflow__cli__subcmd__issue,view)
                cmd="gitflow__cli__subcmd__issue__subcmd__view"
                ;;
            gitflow__cli__subcmd__issue__subcmd__help,add-label)
                cmd="gitflow__cli__subcmd__issue__subcmd__help__subcmd__add__subcmd__label"
                ;;
            gitflow__cli__subcmd__issue__subcmd__help,close)
                cmd="gitflow__cli__subcmd__issue__subcmd__help__subcmd__close"
                ;;
            gitflow__cli__subcmd__issue__subcmd__help,comment)
                cmd="gitflow__cli__subcmd__issue__subcmd__help__subcmd__comment"
                ;;
            gitflow__cli__subcmd__issue__subcmd__help,create)
                cmd="gitflow__cli__subcmd__issue__subcmd__help__subcmd__create"
                ;;
            gitflow__cli__subcmd__issue__subcmd__help,help)
                cmd="gitflow__cli__subcmd__issue__subcmd__help__subcmd__help"
                ;;
            gitflow__cli__subcmd__issue__subcmd__help,list)
                cmd="gitflow__cli__subcmd__issue__subcmd__help__subcmd__list"
                ;;
            gitflow__cli__subcmd__issue__subcmd__help,remove-label)
                cmd="gitflow__cli__subcmd__issue__subcmd__help__subcmd__remove__subcmd__label"
                ;;
            gitflow__cli__subcmd__issue__subcmd__help,reopen)
                cmd="gitflow__cli__subcmd__issue__subcmd__help__subcmd__reopen"
                ;;
            gitflow__cli__subcmd__issue__subcmd__help,view)
                cmd="gitflow__cli__subcmd__issue__subcmd__help__subcmd__view"
                ;;
            gitflow__cli__subcmd__label,create)
                cmd="gitflow__cli__subcmd__label__subcmd__create"
                ;;
            gitflow__cli__subcmd__label,delete)
                cmd="gitflow__cli__subcmd__label__subcmd__delete"
                ;;
            gitflow__cli__subcmd__label,edit)
                cmd="gitflow__cli__subcmd__label__subcmd__edit"
                ;;
            gitflow__cli__subcmd__label,help)
                cmd="gitflow__cli__subcmd__label__subcmd__help"
                ;;
            gitflow__cli__subcmd__label,list)
                cmd="gitflow__cli__subcmd__label__subcmd__list"
                ;;
            gitflow__cli__subcmd__label__subcmd__help,create)
                cmd="gitflow__cli__subcmd__label__subcmd__help__subcmd__create"
                ;;
            gitflow__cli__subcmd__label__subcmd__help,delete)
                cmd="gitflow__cli__subcmd__label__subcmd__help__subcmd__delete"
                ;;
            gitflow__cli__subcmd__label__subcmd__help,edit)
                cmd="gitflow__cli__subcmd__label__subcmd__help__subcmd__edit"
                ;;
            gitflow__cli__subcmd__label__subcmd__help,help)
                cmd="gitflow__cli__subcmd__label__subcmd__help__subcmd__help"
                ;;
            gitflow__cli__subcmd__label__subcmd__help,list)
                cmd="gitflow__cli__subcmd__label__subcmd__help__subcmd__list"
                ;;
            gitflow__cli__subcmd__milestone,close)
                cmd="gitflow__cli__subcmd__milestone__subcmd__close"
                ;;
            gitflow__cli__subcmd__milestone,create)
                cmd="gitflow__cli__subcmd__milestone__subcmd__create"
                ;;
            gitflow__cli__subcmd__milestone,edit)
                cmd="gitflow__cli__subcmd__milestone__subcmd__edit"
                ;;
            gitflow__cli__subcmd__milestone,help)
                cmd="gitflow__cli__subcmd__milestone__subcmd__help"
                ;;
            gitflow__cli__subcmd__milestone,list)
                cmd="gitflow__cli__subcmd__milestone__subcmd__list"
                ;;
            gitflow__cli__subcmd__milestone,reopen)
                cmd="gitflow__cli__subcmd__milestone__subcmd__reopen"
                ;;
            gitflow__cli__subcmd__milestone__subcmd__help,close)
                cmd="gitflow__cli__subcmd__milestone__subcmd__help__subcmd__close"
                ;;
            gitflow__cli__subcmd__milestone__subcmd__help,create)
                cmd="gitflow__cli__subcmd__milestone__subcmd__help__subcmd__create"
                ;;
            gitflow__cli__subcmd__milestone__subcmd__help,edit)
                cmd="gitflow__cli__subcmd__milestone__subcmd__help__subcmd__edit"
                ;;
            gitflow__cli__subcmd__milestone__subcmd__help,help)
                cmd="gitflow__cli__subcmd__milestone__subcmd__help__subcmd__help"
                ;;
            gitflow__cli__subcmd__milestone__subcmd__help,list)
                cmd="gitflow__cli__subcmd__milestone__subcmd__help__subcmd__list"
                ;;
            gitflow__cli__subcmd__milestone__subcmd__help,reopen)
                cmd="gitflow__cli__subcmd__milestone__subcmd__help__subcmd__reopen"
                ;;
            gitflow__cli__subcmd__pipeline,help)
                cmd="gitflow__cli__subcmd__pipeline__subcmd__help"
                ;;
            gitflow__cli__subcmd__pipeline,jobs)
                cmd="gitflow__cli__subcmd__pipeline__subcmd__jobs"
                ;;
            gitflow__cli__subcmd__pipeline,logs)
                cmd="gitflow__cli__subcmd__pipeline__subcmd__logs"
                ;;
            gitflow__cli__subcmd__pipeline,report)
                cmd="gitflow__cli__subcmd__pipeline__subcmd__report"
                ;;
            gitflow__cli__subcmd__pipeline,status)
                cmd="gitflow__cli__subcmd__pipeline__subcmd__status"
                ;;
            gitflow__cli__subcmd__pipeline__subcmd__help,help)
                cmd="gitflow__cli__subcmd__pipeline__subcmd__help__subcmd__help"
                ;;
            gitflow__cli__subcmd__pipeline__subcmd__help,jobs)
                cmd="gitflow__cli__subcmd__pipeline__subcmd__help__subcmd__jobs"
                ;;
            gitflow__cli__subcmd__pipeline__subcmd__help,logs)
                cmd="gitflow__cli__subcmd__pipeline__subcmd__help__subcmd__logs"
                ;;
            gitflow__cli__subcmd__pipeline__subcmd__help,report)
                cmd="gitflow__cli__subcmd__pipeline__subcmd__help__subcmd__report"
                ;;
            gitflow__cli__subcmd__pipeline__subcmd__help,status)
                cmd="gitflow__cli__subcmd__pipeline__subcmd__help__subcmd__status"
                ;;
            gitflow__cli__subcmd__pr,checkout)
                cmd="gitflow__cli__subcmd__pr__subcmd__checkout"
                ;;
            gitflow__cli__subcmd__pr,close)
                cmd="gitflow__cli__subcmd__pr__subcmd__close"
                ;;
            gitflow__cli__subcmd__pr,comment)
                cmd="gitflow__cli__subcmd__pr__subcmd__comment"
                ;;
            gitflow__cli__subcmd__pr,create)
                cmd="gitflow__cli__subcmd__pr__subcmd__create"
                ;;
            gitflow__cli__subcmd__pr,help)
                cmd="gitflow__cli__subcmd__pr__subcmd__help"
                ;;
            gitflow__cli__subcmd__pr,list)
                cmd="gitflow__cli__subcmd__pr__subcmd__list"
                ;;
            gitflow__cli__subcmd__pr,merge)
                cmd="gitflow__cli__subcmd__pr__subcmd__merge"
                ;;
            gitflow__cli__subcmd__pr,ready)
                cmd="gitflow__cli__subcmd__pr__subcmd__ready"
                ;;
            gitflow__cli__subcmd__pr,reopen)
                cmd="gitflow__cli__subcmd__pr__subcmd__reopen"
                ;;
            gitflow__cli__subcmd__pr,sync)
                cmd="gitflow__cli__subcmd__pr__subcmd__sync"
                ;;
            gitflow__cli__subcmd__pr,view)
                cmd="gitflow__cli__subcmd__pr__subcmd__view"
                ;;
            gitflow__cli__subcmd__pr,wip)
                cmd="gitflow__cli__subcmd__pr__subcmd__wip"
                ;;
            gitflow__cli__subcmd__pr__subcmd__help,checkout)
                cmd="gitflow__cli__subcmd__pr__subcmd__help__subcmd__checkout"
                ;;
            gitflow__cli__subcmd__pr__subcmd__help,close)
                cmd="gitflow__cli__subcmd__pr__subcmd__help__subcmd__close"
                ;;
            gitflow__cli__subcmd__pr__subcmd__help,comment)
                cmd="gitflow__cli__subcmd__pr__subcmd__help__subcmd__comment"
                ;;
            gitflow__cli__subcmd__pr__subcmd__help,create)
                cmd="gitflow__cli__subcmd__pr__subcmd__help__subcmd__create"
                ;;
            gitflow__cli__subcmd__pr__subcmd__help,help)
                cmd="gitflow__cli__subcmd__pr__subcmd__help__subcmd__help"
                ;;
            gitflow__cli__subcmd__pr__subcmd__help,list)
                cmd="gitflow__cli__subcmd__pr__subcmd__help__subcmd__list"
                ;;
            gitflow__cli__subcmd__pr__subcmd__help,merge)
                cmd="gitflow__cli__subcmd__pr__subcmd__help__subcmd__merge"
                ;;
            gitflow__cli__subcmd__pr__subcmd__help,ready)
                cmd="gitflow__cli__subcmd__pr__subcmd__help__subcmd__ready"
                ;;
            gitflow__cli__subcmd__pr__subcmd__help,reopen)
                cmd="gitflow__cli__subcmd__pr__subcmd__help__subcmd__reopen"
                ;;
            gitflow__cli__subcmd__pr__subcmd__help,sync)
                cmd="gitflow__cli__subcmd__pr__subcmd__help__subcmd__sync"
                ;;
            gitflow__cli__subcmd__pr__subcmd__help,view)
                cmd="gitflow__cli__subcmd__pr__subcmd__help__subcmd__view"
                ;;
            gitflow__cli__subcmd__pr__subcmd__help,wip)
                cmd="gitflow__cli__subcmd__pr__subcmd__help__subcmd__wip"
                ;;
            gitflow__cli__subcmd__release,create)
                cmd="gitflow__cli__subcmd__release__subcmd__create"
                ;;
            gitflow__cli__subcmd__release,delete)
                cmd="gitflow__cli__subcmd__release__subcmd__delete"
                ;;
            gitflow__cli__subcmd__release,download)
                cmd="gitflow__cli__subcmd__release__subcmd__download"
                ;;
            gitflow__cli__subcmd__release,edit)
                cmd="gitflow__cli__subcmd__release__subcmd__edit"
                ;;
            gitflow__cli__subcmd__release,help)
                cmd="gitflow__cli__subcmd__release__subcmd__help"
                ;;
            gitflow__cli__subcmd__release,list)
                cmd="gitflow__cli__subcmd__release__subcmd__list"
                ;;
            gitflow__cli__subcmd__release,upload)
                cmd="gitflow__cli__subcmd__release__subcmd__upload"
                ;;
            gitflow__cli__subcmd__release,view)
                cmd="gitflow__cli__subcmd__release__subcmd__view"
                ;;
            gitflow__cli__subcmd__release__subcmd__help,create)
                cmd="gitflow__cli__subcmd__release__subcmd__help__subcmd__create"
                ;;
            gitflow__cli__subcmd__release__subcmd__help,delete)
                cmd="gitflow__cli__subcmd__release__subcmd__help__subcmd__delete"
                ;;
            gitflow__cli__subcmd__release__subcmd__help,download)
                cmd="gitflow__cli__subcmd__release__subcmd__help__subcmd__download"
                ;;
            gitflow__cli__subcmd__release__subcmd__help,edit)
                cmd="gitflow__cli__subcmd__release__subcmd__help__subcmd__edit"
                ;;
            gitflow__cli__subcmd__release__subcmd__help,help)
                cmd="gitflow__cli__subcmd__release__subcmd__help__subcmd__help"
                ;;
            gitflow__cli__subcmd__release__subcmd__help,list)
                cmd="gitflow__cli__subcmd__release__subcmd__help__subcmd__list"
                ;;
            gitflow__cli__subcmd__release__subcmd__help,upload)
                cmd="gitflow__cli__subcmd__release__subcmd__help__subcmd__upload"
                ;;
            gitflow__cli__subcmd__release__subcmd__help,view)
                cmd="gitflow__cli__subcmd__release__subcmd__help__subcmd__view"
                ;;
            gitflow__cli__subcmd__review,approve)
                cmd="gitflow__cli__subcmd__review__subcmd__approve"
                ;;
            gitflow__cli__subcmd__review,comment)
                cmd="gitflow__cli__subcmd__review__subcmd__comment"
                ;;
            gitflow__cli__subcmd__review,help)
                cmd="gitflow__cli__subcmd__review__subcmd__help"
                ;;
            gitflow__cli__subcmd__review,request-changes)
                cmd="gitflow__cli__subcmd__review__subcmd__request__subcmd__changes"
                ;;
            gitflow__cli__subcmd__review,submit)
                cmd="gitflow__cli__subcmd__review__subcmd__submit"
                ;;
            gitflow__cli__subcmd__review__subcmd__help,approve)
                cmd="gitflow__cli__subcmd__review__subcmd__help__subcmd__approve"
                ;;
            gitflow__cli__subcmd__review__subcmd__help,comment)
                cmd="gitflow__cli__subcmd__review__subcmd__help__subcmd__comment"
                ;;
            gitflow__cli__subcmd__review__subcmd__help,help)
                cmd="gitflow__cli__subcmd__review__subcmd__help__subcmd__help"
                ;;
            gitflow__cli__subcmd__review__subcmd__help,request-changes)
                cmd="gitflow__cli__subcmd__review__subcmd__help__subcmd__request__subcmd__changes"
                ;;
            gitflow__cli__subcmd__review__subcmd__help,submit)
                cmd="gitflow__cli__subcmd__review__subcmd__help__subcmd__submit"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        gitflow__cli)
            opts="-v -h --platform --output --verbose --help issue pr release review auth label milestone commit pipeline skills run completions help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__auth)
            opts="-v -h --platform --output --verbose --help login logout status token help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__auth__subcmd__help)
            opts="login logout status token help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__auth__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__auth__subcmd__help__subcmd__login)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__auth__subcmd__help__subcmd__logout)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__auth__subcmd__help__subcmd__status)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__auth__subcmd__help__subcmd__token)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__auth__subcmd__login)
            opts="-v -h --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__auth__subcmd__logout)
            opts="-v -h --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__auth__subcmd__status)
            opts="-v -h --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__auth__subcmd__token)
            opts="-v -h --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__commit)
            opts="-v -h --platform --output --verbose --help view diff patch comment help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__commit__subcmd__comment)
            opts="-v -h --body --body-file --path --line --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --body)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --body-file)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --path)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --line)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__commit__subcmd__diff)
            opts="-v -h --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__commit__subcmd__help)
            opts="view diff patch comment help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__commit__subcmd__help__subcmd__comment)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__commit__subcmd__help__subcmd__diff)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__commit__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__commit__subcmd__help__subcmd__patch)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__commit__subcmd__help__subcmd__view)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__commit__subcmd__patch)
            opts="-v -h --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__commit__subcmd__view)
            opts="-v -h --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__completions)
            opts="-v -h --install --uninstall --platform --output --verbose --help bash zsh fish"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help)
            opts="issue pr release review auth label milestone commit pipeline skills run completions help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__auth)
            opts="login logout status token"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__auth__subcmd__login)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__auth__subcmd__logout)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__auth__subcmd__status)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__auth__subcmd__token)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__commit)
            opts="view diff patch comment"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__commit__subcmd__comment)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__commit__subcmd__diff)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__commit__subcmd__patch)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__commit__subcmd__view)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__completions)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__issue)
            opts="create list view close reopen comment add-label remove-label"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__issue__subcmd__add__subcmd__label)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__issue__subcmd__close)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__issue__subcmd__comment)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__issue__subcmd__create)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__issue__subcmd__list)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__issue__subcmd__remove__subcmd__label)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__issue__subcmd__reopen)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__issue__subcmd__view)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__label)
            opts="create list edit delete"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__label__subcmd__create)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__label__subcmd__delete)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__label__subcmd__edit)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__label__subcmd__list)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__milestone)
            opts="create list edit close reopen"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__milestone__subcmd__close)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__milestone__subcmd__create)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__milestone__subcmd__edit)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__milestone__subcmd__list)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__milestone__subcmd__reopen)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__pipeline)
            opts="status logs jobs report"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__pipeline__subcmd__jobs)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__pipeline__subcmd__logs)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__pipeline__subcmd__report)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__pipeline__subcmd__status)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__pr)
            opts="create list view close reopen comment merge checkout ready wip sync"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__pr__subcmd__checkout)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__pr__subcmd__close)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__pr__subcmd__comment)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__pr__subcmd__create)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__pr__subcmd__list)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__pr__subcmd__merge)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__pr__subcmd__ready)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__pr__subcmd__reopen)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__pr__subcmd__sync)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__pr__subcmd__view)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__pr__subcmd__wip)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__release)
            opts="create list view edit upload download delete"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__release__subcmd__create)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__release__subcmd__delete)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__release__subcmd__download)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__release__subcmd__edit)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__release__subcmd__list)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__release__subcmd__upload)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__release__subcmd__view)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__review)
            opts="comment approve request-changes submit"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__review__subcmd__approve)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__review__subcmd__comment)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__review__subcmd__request__subcmd__changes)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__review__subcmd__submit)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__run)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__help__subcmd__skills)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__issue)
            opts="-v -h --platform --output --verbose --help create list view close reopen comment add-label remove-label help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__issue__subcmd__add__subcmd__label)
            opts="-v -h --label --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --label)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__issue__subcmd__close)
            opts="-v -h --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__issue__subcmd__comment)
            opts="-v -h --body --body-file --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --body)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --body-file)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__issue__subcmd__create)
            opts="-v -h --title --body --body-file --label --assignee --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --title)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --body)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --body-file)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --label)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --assignee)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__issue__subcmd__help)
            opts="create list view close reopen comment add-label remove-label help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__issue__subcmd__help__subcmd__add__subcmd__label)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__issue__subcmd__help__subcmd__close)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__issue__subcmd__help__subcmd__comment)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__issue__subcmd__help__subcmd__create)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__issue__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__issue__subcmd__help__subcmd__list)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__issue__subcmd__help__subcmd__remove__subcmd__label)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__issue__subcmd__help__subcmd__reopen)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__issue__subcmd__help__subcmd__view)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__issue__subcmd__list)
            opts="-v -h --state --search --label --limit --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --state)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --search)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --label)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__issue__subcmd__remove__subcmd__label)
            opts="-v -h --label --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --label)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__issue__subcmd__reopen)
            opts="-v -h --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__issue__subcmd__view)
            opts="-v -h --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__label)
            opts="-v -h --platform --output --verbose --help create list edit delete help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__label__subcmd__create)
            opts="-v -h --color --description --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --color)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --description)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__label__subcmd__delete)
            opts="-y -v -h --yes --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__label__subcmd__edit)
            opts="-v -h --color --description --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --color)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --description)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__label__subcmd__help)
            opts="create list edit delete help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__label__subcmd__help__subcmd__create)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__label__subcmd__help__subcmd__delete)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__label__subcmd__help__subcmd__edit)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__label__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__label__subcmd__help__subcmd__list)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__label__subcmd__list)
            opts="-v -h --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__milestone)
            opts="-v -h --platform --output --verbose --help create list edit close reopen help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__milestone__subcmd__close)
            opts="-v -h --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__milestone__subcmd__create)
            opts="-v -h --title --description --due-on --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --title)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --description)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --due-on)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__milestone__subcmd__edit)
            opts="-v -h --title --description --due-on --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --title)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --description)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --due-on)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__milestone__subcmd__help)
            opts="create list edit close reopen help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__milestone__subcmd__help__subcmd__close)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__milestone__subcmd__help__subcmd__create)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__milestone__subcmd__help__subcmd__edit)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__milestone__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__milestone__subcmd__help__subcmd__list)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__milestone__subcmd__help__subcmd__reopen)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__milestone__subcmd__list)
            opts="-v -h --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__milestone__subcmd__reopen)
            opts="-v -h --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pipeline)
            opts="-v -h --platform --output --verbose --help status logs jobs report help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pipeline__subcmd__help)
            opts="status logs jobs report help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pipeline__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pipeline__subcmd__help__subcmd__jobs)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pipeline__subcmd__help__subcmd__logs)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pipeline__subcmd__help__subcmd__report)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pipeline__subcmd__help__subcmd__status)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pipeline__subcmd__jobs)
            opts="-v -h --pipeline-id --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --pipeline-id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pipeline__subcmd__logs)
            opts="-v -h --pipeline-id --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --pipeline-id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pipeline__subcmd__report)
            opts="-v -h --branch --days --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --branch)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --days)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pipeline__subcmd__status)
            opts="-v -h --branch --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --branch)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pr)
            opts="-v -h --platform --output --verbose --help create list view close reopen comment merge checkout ready wip sync help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pr__subcmd__checkout)
            opts="-v -h --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pr__subcmd__close)
            opts="-v -h --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pr__subcmd__comment)
            opts="-v -h --body --body-file --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --body)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --body-file)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pr__subcmd__create)
            opts="-v -h --title --head --base --body --body-file --draft --repo --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --title)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --head)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --base)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --body)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --body-file)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --repo)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pr__subcmd__help)
            opts="create list view close reopen comment merge checkout ready wip sync help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pr__subcmd__help__subcmd__checkout)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pr__subcmd__help__subcmd__close)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pr__subcmd__help__subcmd__comment)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pr__subcmd__help__subcmd__create)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pr__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pr__subcmd__help__subcmd__list)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pr__subcmd__help__subcmd__merge)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pr__subcmd__help__subcmd__ready)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pr__subcmd__help__subcmd__reopen)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pr__subcmd__help__subcmd__sync)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pr__subcmd__help__subcmd__view)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pr__subcmd__help__subcmd__wip)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pr__subcmd__list)
            opts="-v -h --state --limit --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --state)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pr__subcmd__merge)
            opts="-v -h --strategy --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --strategy)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pr__subcmd__ready)
            opts="-v -h --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pr__subcmd__reopen)
            opts="-v -h --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pr__subcmd__sync)
            opts="-v -h --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pr__subcmd__view)
            opts="-v -h --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__pr__subcmd__wip)
            opts="-v -h --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__release)
            opts="-v -h --platform --output --verbose --help create list view edit upload download delete help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__release__subcmd__create)
            opts="-v -h --tag-name --name --body --body-file --draft --prerelease --target-commitish --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --tag-name)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --name)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --body)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --body-file)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --target-commitish)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__release__subcmd__delete)
            opts="-y -v -h --yes --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__release__subcmd__download)
            opts="-v -h --pattern --dir --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --pattern)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --dir)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__release__subcmd__edit)
            opts="-v -h --name --body --body-file --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --name)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --body)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --body-file)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__release__subcmd__help)
            opts="create list view edit upload download delete help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__release__subcmd__help__subcmd__create)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__release__subcmd__help__subcmd__delete)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__release__subcmd__help__subcmd__download)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__release__subcmd__help__subcmd__edit)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__release__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__release__subcmd__help__subcmd__list)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__release__subcmd__help__subcmd__upload)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__release__subcmd__help__subcmd__view)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__release__subcmd__list)
            opts="-v -h --limit --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__release__subcmd__upload)
            opts="-v -h --file --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --file)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__release__subcmd__view)
            opts="-v -h --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__review)
            opts="-v -h --platform --output --verbose --help comment approve request-changes submit help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__review__subcmd__approve)
            opts="-v -h --body --body-file --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --body)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --body-file)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__review__subcmd__comment)
            opts="-v -h --body --body-file --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --body)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --body-file)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__review__subcmd__help)
            opts="comment approve request-changes submit help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__review__subcmd__help__subcmd__approve)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__review__subcmd__help__subcmd__comment)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__review__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__review__subcmd__help__subcmd__request__subcmd__changes)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__review__subcmd__help__subcmd__submit)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__review__subcmd__request__subcmd__changes)
            opts="-v -h --body --body-file --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --body)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --body-file)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__review__subcmd__submit)
            opts="-v -h --body --body-file --event --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --body)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --body-file)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --event)
                    COMPREPLY=($(compgen -W "approved changes_requested commented" -- "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__run)
            opts="-n -v -h --name --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --name)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        gitflow__subcmd__cli__subcmd__skills)
            opts="-v -h --platform --output --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -W "github gitlab gitcode" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json text" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
    esac
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _gitflow-cli -o nosort -o bashdefault -o default gitflow-cli
else
    complete -F _gitflow-cli -o bashdefault -o default gitflow-cli
fi
