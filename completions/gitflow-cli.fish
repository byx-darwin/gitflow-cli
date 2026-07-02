# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_gitflow_cli_global_optspecs
    string join \n platform= output= v/verbose h/help
end

function __fish_gitflow_cli_needs_command
    # Figure out if the current invocation already has a command.
    set -l cmd (commandline -opc)
    set -e cmd[1]
    argparse -s (__fish_gitflow_cli_global_optspecs) -- $cmd 2>/dev/null
    or return
    if set -q argv[1]
        # Also print the command, so this can be used to figure out what it is.
        echo $argv[1]
        return 1
    end
    return 0
end

function __fish_gitflow_cli_using_subcommand
    set -l cmd (__fish_gitflow_cli_needs_command)
    test -z "$cmd"
    and return 1
    contains -- $cmd[1] $argv
end

complete -c gitflow-cli -n "__fish_gitflow_cli_needs_command" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_needs_command" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_needs_command" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_needs_command" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_needs_command" -f -a "issue" -d 'Issue operations (create, list, view)'
complete -c gitflow-cli -n "__fish_gitflow_cli_needs_command" -f -a "pr" -d 'Pull request operations (create, list, view)'
complete -c gitflow-cli -n "__fish_gitflow_cli_needs_command" -f -a "release" -d 'Release operations (create, list, view, edit, upload, download, delete)'
complete -c gitflow-cli -n "__fish_gitflow_cli_needs_command" -f -a "review" -d 'Code review operations (comment, approve, request-changes, submit)'
complete -c gitflow-cli -n "__fish_gitflow_cli_needs_command" -f -a "auth" -d 'Authentication operations (login, logout, status, token)'
complete -c gitflow-cli -n "__fish_gitflow_cli_needs_command" -f -a "label" -d 'Label operations (create, list, edit, delete)'
complete -c gitflow-cli -n "__fish_gitflow_cli_needs_command" -f -a "milestone" -d 'Milestone operations (create, list, edit, close, reopen)'
complete -c gitflow-cli -n "__fish_gitflow_cli_needs_command" -f -a "commit" -d 'Commit operations (view, diff, patch, comment)'
complete -c gitflow-cli -n "__fish_gitflow_cli_needs_command" -f -a "skills" -d 'Install gitflow skills to `~/.claude/skills/`'
complete -c gitflow-cli -n "__fish_gitflow_cli_needs_command" -f -a "run" -d 'Run the main application workflow (deprecated)'
complete -c gitflow-cli -n "__fish_gitflow_cli_needs_command" -f -a "completions" -d 'Generate shell completion scripts'
complete -c gitflow-cli -n "__fish_gitflow_cli_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and not __fish_seen_subcommand_from create list view close reopen comment add-label remove-label help" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and not __fish_seen_subcommand_from create list view close reopen comment add-label remove-label help" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and not __fish_seen_subcommand_from create list view close reopen comment add-label remove-label help" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and not __fish_seen_subcommand_from create list view close reopen comment add-label remove-label help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and not __fish_seen_subcommand_from create list view close reopen comment add-label remove-label help" -f -a "create" -d '创建一个新的 Issue。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and not __fish_seen_subcommand_from create list view close reopen comment add-label remove-label help" -f -a "list" -d '列出 Issue。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and not __fish_seen_subcommand_from create list view close reopen comment add-label remove-label help" -f -a "view" -d '查看单个 Issue 详情。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and not __fish_seen_subcommand_from create list view close reopen comment add-label remove-label help" -f -a "close" -d '关闭 Issue。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and not __fish_seen_subcommand_from create list view close reopen comment add-label remove-label help" -f -a "reopen" -d '重新打开 Issue。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and not __fish_seen_subcommand_from create list view close reopen comment add-label remove-label help" -f -a "comment" -d '评论 Issue。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and not __fish_seen_subcommand_from create list view close reopen comment add-label remove-label help" -f -a "add-label" -d '为 Issue 添加标签。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and not __fish_seen_subcommand_from create list view close reopen comment add-label remove-label help" -f -a "remove-label" -d '从 Issue 移除标签。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and not __fish_seen_subcommand_from create list view close reopen comment add-label remove-label help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from create" -l title -d 'Issue 标题（必填）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from create" -l body -d 'Issue 正文（可选，与 `--body-file` 二选一）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from create" -l body-file -d '从文件读取 Issue 正文（可选）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from create" -l label -d '标签列表（可多次指定）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from create" -l assignee -d '指派人列表（可多次指定）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from create" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from create" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from create" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from create" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from list" -l state -d '按状态过滤（`open` 或 `closed`）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from list" -l search -d '搜索关键词。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from list" -l label -d '按标签过滤（可多次指定）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from list" -l limit -d '返回数量上限。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from list" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from list" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from list" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from view" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from view" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from view" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from view" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from close" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from close" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from close" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from close" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from reopen" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from reopen" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from reopen" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from reopen" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from comment" -l body -d '评论正文（可选，与 `--body-file` 二选一）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from comment" -l body-file -d '从文件读取评论正文（可选）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from comment" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from comment" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from comment" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from comment" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from add-label" -l label -d '要添加的标签（至少一个）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from add-label" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from add-label" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from add-label" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from add-label" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from remove-label" -l label -d '要移除的标签名称。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from remove-label" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from remove-label" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from remove-label" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from remove-label" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from help" -f -a "create" -d '创建一个新的 Issue。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from help" -f -a "list" -d '列出 Issue。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from help" -f -a "view" -d '查看单个 Issue 详情。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from help" -f -a "close" -d '关闭 Issue。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from help" -f -a "reopen" -d '重新打开 Issue。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from help" -f -a "comment" -d '评论 Issue。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from help" -f -a "add-label" -d '为 Issue 添加标签。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from help" -f -a "remove-label" -d '从 Issue 移除标签。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand issue; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and not __fish_seen_subcommand_from create list view close reopen comment merge checkout ready wip sync help" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and not __fish_seen_subcommand_from create list view close reopen comment merge checkout ready wip sync help" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and not __fish_seen_subcommand_from create list view close reopen comment merge checkout ready wip sync help" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and not __fish_seen_subcommand_from create list view close reopen comment merge checkout ready wip sync help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and not __fish_seen_subcommand_from create list view close reopen comment merge checkout ready wip sync help" -f -a "create" -d '创建一条新的 Pull Request。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and not __fish_seen_subcommand_from create list view close reopen comment merge checkout ready wip sync help" -f -a "list" -d '列出 Pull Request。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and not __fish_seen_subcommand_from create list view close reopen comment merge checkout ready wip sync help" -f -a "view" -d '查看单个 Pull Request 详情。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and not __fish_seen_subcommand_from create list view close reopen comment merge checkout ready wip sync help" -f -a "close" -d '关闭 Pull Request。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and not __fish_seen_subcommand_from create list view close reopen comment merge checkout ready wip sync help" -f -a "reopen" -d '重新打开 Pull Request。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and not __fish_seen_subcommand_from create list view close reopen comment merge checkout ready wip sync help" -f -a "comment" -d '评论 Pull Request。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and not __fish_seen_subcommand_from create list view close reopen comment merge checkout ready wip sync help" -f -a "merge" -d '合并 Pull Request。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and not __fish_seen_subcommand_from create list view close reopen comment merge checkout ready wip sync help" -f -a "checkout" -d '在本地检出 Pull Request 的分支。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and not __fish_seen_subcommand_from create list view close reopen comment merge checkout ready wip sync help" -f -a "ready" -d '将草稿 PR 标记为可审查状态。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and not __fish_seen_subcommand_from create list view close reopen comment merge checkout ready wip sync help" -f -a "wip" -d '将 PR 标记为草稿状态。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and not __fish_seen_subcommand_from create list view close reopen comment merge checkout ready wip sync help" -f -a "sync" -d '同步 PR 的分支（将 base 分支的最新变更合入 head 分支）。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and not __fish_seen_subcommand_from create list view close reopen comment merge checkout ready wip sync help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from create" -l title -d 'PR 标题（必填）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from create" -l head -d '来源分支（可选，默认为当前 git 分支）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from create" -l base -d '目标分支（可选，默认为 `main`）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from create" -l body -d 'PR 正文（可选，与 `--body-file` 二选一）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from create" -l body-file -d '从文件读取 PR 正文（可选）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from create" -l repo -d '目标仓库（`owner/name` 格式，可选，默认为当前仓库）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from create" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from create" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from create" -l draft -d '是否以草稿方式创建。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from create" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from create" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from list" -l state -d '按状态过滤（`open` 或 `closed`）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from list" -l limit -d '返回数量上限。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from list" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from list" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from list" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from view" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from view" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from view" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from view" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from close" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from close" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from close" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from close" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from reopen" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from reopen" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from reopen" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from reopen" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from comment" -l body -d '评论正文（可选，与 `--body-file` 二选一）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from comment" -l body-file -d '从文件读取评论正文（可选）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from comment" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from comment" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from comment" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from comment" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from merge" -l strategy -d '合并策略（`merge`、`squash` 或 `rebase`，默认为 `merge`）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from merge" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from merge" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from merge" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from merge" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from checkout" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from checkout" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from checkout" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from checkout" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from ready" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from ready" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from ready" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from ready" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from wip" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from wip" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from wip" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from wip" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from sync" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from sync" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from sync" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from sync" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from help" -f -a "create" -d '创建一条新的 Pull Request。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from help" -f -a "list" -d '列出 Pull Request。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from help" -f -a "view" -d '查看单个 Pull Request 详情。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from help" -f -a "close" -d '关闭 Pull Request。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from help" -f -a "reopen" -d '重新打开 Pull Request。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from help" -f -a "comment" -d '评论 Pull Request。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from help" -f -a "merge" -d '合并 Pull Request。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from help" -f -a "checkout" -d '在本地检出 Pull Request 的分支。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from help" -f -a "ready" -d '将草稿 PR 标记为可审查状态。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from help" -f -a "wip" -d '将 PR 标记为草稿状态。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from help" -f -a "sync" -d '同步 PR 的分支（将 base 分支的最新变更合入 head 分支）。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand pr; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and not __fish_seen_subcommand_from create list view edit upload download delete help" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and not __fish_seen_subcommand_from create list view edit upload download delete help" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and not __fish_seen_subcommand_from create list view edit upload download delete help" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and not __fish_seen_subcommand_from create list view edit upload download delete help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and not __fish_seen_subcommand_from create list view edit upload download delete help" -f -a "create" -d '创建一个新的 Release。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and not __fish_seen_subcommand_from create list view edit upload download delete help" -f -a "list" -d '列出 Release。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and not __fish_seen_subcommand_from create list view edit upload download delete help" -f -a "view" -d '查看指定 tag 的 Release 详情。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and not __fish_seen_subcommand_from create list view edit upload download delete help" -f -a "edit" -d '编辑指定 Release 的元数据。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and not __fish_seen_subcommand_from create list view edit upload download delete help" -f -a "upload" -d '上传资源文件到 Release。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and not __fish_seen_subcommand_from create list view edit upload download delete help" -f -a "download" -d '下载 Release 的资源文件。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and not __fish_seen_subcommand_from create list view edit upload download delete help" -f -a "delete" -d '删除指定 Release。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and not __fish_seen_subcommand_from create list view edit upload download delete help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from create" -l tag-name -d 'Git tag 名称（必填）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from create" -l name -d 'Release 标题（可选）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from create" -l body -d 'Release 正文（可选，与 `--body-file` 二选一）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from create" -l body-file -d '从文件读取 Release 正文（可选）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from create" -l target-commitish -d '目标 commitish（可选，默认为当前分支 HEAD）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from create" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from create" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from create" -l draft -d '以草稿方式创建。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from create" -l prerelease -d '以预发布方式创建。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from create" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from create" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from list" -l limit -d '返回数量上限（可选）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from list" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from list" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from list" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from view" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from view" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from view" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from view" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from edit" -l name -d 'Release 标题（可选）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from edit" -l body -d 'Release 正文（可选，与 `--body-file` 二选一）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from edit" -l body-file -d '从文件读取 Release 正文（可选）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from edit" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from edit" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from edit" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from edit" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from upload" -l file -d '本地文件路径。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from upload" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from upload" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from upload" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from upload" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from download" -l pattern -d '文件名匹配模式（可选）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from download" -l dir -d '下载目录（可选）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from download" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from download" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from download" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from download" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from delete" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from delete" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from delete" -s y -l yes -d '跳过确认提示。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from delete" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from delete" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from help" -f -a "create" -d '创建一个新的 Release。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from help" -f -a "list" -d '列出 Release。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from help" -f -a "view" -d '查看指定 tag 的 Release 详情。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from help" -f -a "edit" -d '编辑指定 Release 的元数据。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from help" -f -a "upload" -d '上传资源文件到 Release。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from help" -f -a "download" -d '下载 Release 的资源文件。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from help" -f -a "delete" -d '删除指定 Release。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand release; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and not __fish_seen_subcommand_from comment approve request-changes submit help" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and not __fish_seen_subcommand_from comment approve request-changes submit help" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and not __fish_seen_subcommand_from comment approve request-changes submit help" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and not __fish_seen_subcommand_from comment approve request-changes submit help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and not __fish_seen_subcommand_from comment approve request-changes submit help" -f -a "comment" -d '在 PR 上发表评论。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and not __fish_seen_subcommand_from comment approve request-changes submit help" -f -a "approve" -d '批准 PR。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and not __fish_seen_subcommand_from comment approve request-changes submit help" -f -a "request-changes" -d '要求对 PR 进行修改。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and not __fish_seen_subcommand_from comment approve request-changes submit help" -f -a "submit" -d '提交一次完整的 Review。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and not __fish_seen_subcommand_from comment approve request-changes submit help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and __fish_seen_subcommand_from comment" -l body -d '评论正文（可选，与 `--body-file` 二选一）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and __fish_seen_subcommand_from comment" -l body-file -d '从文件读取评论正文（可选）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and __fish_seen_subcommand_from comment" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and __fish_seen_subcommand_from comment" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and __fish_seen_subcommand_from comment" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and __fish_seen_subcommand_from comment" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and __fish_seen_subcommand_from approve" -l body -d '批准说明（可选，与 `--body-file` 二选一）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and __fish_seen_subcommand_from approve" -l body-file -d '从文件读取批准说明（可选）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and __fish_seen_subcommand_from approve" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and __fish_seen_subcommand_from approve" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and __fish_seen_subcommand_from approve" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and __fish_seen_subcommand_from approve" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and __fish_seen_subcommand_from request-changes" -l body -d '修改要求说明（可选，与 `--body-file` 二选一）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and __fish_seen_subcommand_from request-changes" -l body-file -d '从文件读取修改要求（可选）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and __fish_seen_subcommand_from request-changes" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and __fish_seen_subcommand_from request-changes" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and __fish_seen_subcommand_from request-changes" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and __fish_seen_subcommand_from request-changes" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and __fish_seen_subcommand_from submit" -l body -d 'Review 总结说明（可选，与 `--body-file` 二选一）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and __fish_seen_subcommand_from submit" -l body-file -d '从文件读取 Review 总结（可选）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and __fish_seen_subcommand_from submit" -l event -d 'Review 结论（`approved`、`changes_requested`、`commented`）。' -r -f -a "approved\t'审查通过，可以合并。'
changes_requested\t'要求修改后才能合并。'
commented\t'仅发表评论，不表态。'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and __fish_seen_subcommand_from submit" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and __fish_seen_subcommand_from submit" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and __fish_seen_subcommand_from submit" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and __fish_seen_subcommand_from submit" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and __fish_seen_subcommand_from help" -f -a "comment" -d '在 PR 上发表评论。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and __fish_seen_subcommand_from help" -f -a "approve" -d '批准 PR。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and __fish_seen_subcommand_from help" -f -a "request-changes" -d '要求对 PR 进行修改。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and __fish_seen_subcommand_from help" -f -a "submit" -d '提交一次完整的 Review。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand review; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand auth; and not __fish_seen_subcommand_from login logout status token help" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand auth; and not __fish_seen_subcommand_from login logout status token help" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand auth; and not __fish_seen_subcommand_from login logout status token help" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand auth; and not __fish_seen_subcommand_from login logout status token help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand auth; and not __fish_seen_subcommand_from login logout status token help" -f -a "login" -d '执行登录流程。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand auth; and not __fish_seen_subcommand_from login logout status token help" -f -a "logout" -d '执行登出流程，清除本地凭据。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand auth; and not __fish_seen_subcommand_from login logout status token help" -f -a "status" -d '查询当前认证状态。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand auth; and not __fish_seen_subcommand_from login logout status token help" -f -a "token" -d '获取当前有效的访问 Token。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand auth; and not __fish_seen_subcommand_from login logout status token help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand auth; and __fish_seen_subcommand_from login" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand auth; and __fish_seen_subcommand_from login" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand auth; and __fish_seen_subcommand_from login" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand auth; and __fish_seen_subcommand_from login" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand auth; and __fish_seen_subcommand_from logout" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand auth; and __fish_seen_subcommand_from logout" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand auth; and __fish_seen_subcommand_from logout" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand auth; and __fish_seen_subcommand_from logout" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand auth; and __fish_seen_subcommand_from status" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand auth; and __fish_seen_subcommand_from status" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand auth; and __fish_seen_subcommand_from status" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand auth; and __fish_seen_subcommand_from status" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand auth; and __fish_seen_subcommand_from token" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand auth; and __fish_seen_subcommand_from token" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand auth; and __fish_seen_subcommand_from token" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand auth; and __fish_seen_subcommand_from token" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "login" -d '执行登录流程。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "logout" -d '执行登出流程，清除本地凭据。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "status" -d '查询当前认证状态。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "token" -d '获取当前有效的访问 Token。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and not __fish_seen_subcommand_from create list edit delete help" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and not __fish_seen_subcommand_from create list edit delete help" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and not __fish_seen_subcommand_from create list edit delete help" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and not __fish_seen_subcommand_from create list edit delete help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and not __fish_seen_subcommand_from create list edit delete help" -f -a "create" -d '创建一个新的标签。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and not __fish_seen_subcommand_from create list edit delete help" -f -a "list" -d '列出仓库中的所有标签。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and not __fish_seen_subcommand_from create list edit delete help" -f -a "edit" -d '编辑一个已有的标签。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and not __fish_seen_subcommand_from create list edit delete help" -f -a "delete" -d '删除一个标签。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and not __fish_seen_subcommand_from create list edit delete help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and __fish_seen_subcommand_from create" -l color -d '标签颜色（必填，十六进制格式，如 `d73a4a`）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and __fish_seen_subcommand_from create" -l description -d '标签描述（可选）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and __fish_seen_subcommand_from create" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and __fish_seen_subcommand_from create" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and __fish_seen_subcommand_from create" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and __fish_seen_subcommand_from create" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and __fish_seen_subcommand_from list" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and __fish_seen_subcommand_from list" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and __fish_seen_subcommand_from list" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and __fish_seen_subcommand_from edit" -l color -d '新的标签颜色（可选）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and __fish_seen_subcommand_from edit" -l description -d '新的标签描述（可选）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and __fish_seen_subcommand_from edit" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and __fish_seen_subcommand_from edit" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and __fish_seen_subcommand_from edit" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and __fish_seen_subcommand_from edit" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and __fish_seen_subcommand_from delete" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and __fish_seen_subcommand_from delete" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and __fish_seen_subcommand_from delete" -s y -l yes -d '跳过确认提示（默认跳过确认直接删除）。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and __fish_seen_subcommand_from delete" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and __fish_seen_subcommand_from delete" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and __fish_seen_subcommand_from help" -f -a "create" -d '创建一个新的标签。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and __fish_seen_subcommand_from help" -f -a "list" -d '列出仓库中的所有标签。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and __fish_seen_subcommand_from help" -f -a "edit" -d '编辑一个已有的标签。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and __fish_seen_subcommand_from help" -f -a "delete" -d '删除一个标签。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand label; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and not __fish_seen_subcommand_from create list edit close reopen help" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and not __fish_seen_subcommand_from create list edit close reopen help" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and not __fish_seen_subcommand_from create list edit close reopen help" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and not __fish_seen_subcommand_from create list edit close reopen help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and not __fish_seen_subcommand_from create list edit close reopen help" -f -a "create" -d '创建一个新的里程碑。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and not __fish_seen_subcommand_from create list edit close reopen help" -f -a "list" -d '列出仓库中的所有里程碑。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and not __fish_seen_subcommand_from create list edit close reopen help" -f -a "edit" -d '编辑一个已有的里程碑。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and not __fish_seen_subcommand_from create list edit close reopen help" -f -a "close" -d '关闭一个里程碑。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and not __fish_seen_subcommand_from create list edit close reopen help" -f -a "reopen" -d '重新打开一个已关闭的里程碑。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and not __fish_seen_subcommand_from create list edit close reopen help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from create" -l title -d '里程碑标题（必填）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from create" -l description -d '里程碑描述（可选）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from create" -l due-on -d '截止日期（可选，RFC 3339 格式）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from create" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from create" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from create" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from create" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from list" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from list" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from list" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from edit" -l title -d '新的标题（可选）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from edit" -l description -d '新的描述（可选）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from edit" -l due-on -d '新的截止日期（可选，RFC 3339 格式）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from edit" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from edit" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from edit" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from edit" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from close" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from close" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from close" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from close" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from reopen" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from reopen" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from reopen" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from reopen" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from help" -f -a "create" -d '创建一个新的里程碑。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from help" -f -a "list" -d '列出仓库中的所有里程碑。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from help" -f -a "edit" -d '编辑一个已有的里程碑。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from help" -f -a "close" -d '关闭一个里程碑。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from help" -f -a "reopen" -d '重新打开一个已关闭的里程碑。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand milestone; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and not __fish_seen_subcommand_from view diff patch comment help" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and not __fish_seen_subcommand_from view diff patch comment help" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and not __fish_seen_subcommand_from view diff patch comment help" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and not __fish_seen_subcommand_from view diff patch comment help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and not __fish_seen_subcommand_from view diff patch comment help" -f -a "view" -d '查看 Commit 详情。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and not __fish_seen_subcommand_from view diff patch comment help" -f -a "diff" -d '获取 Commit 的 unified diff 输出。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and not __fish_seen_subcommand_from view diff patch comment help" -f -a "patch" -d '获取 Commit 的原始 patch 内容。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and not __fish_seen_subcommand_from view diff patch comment help" -f -a "comment" -d '评论 Commit 中的特定文件行。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and not __fish_seen_subcommand_from view diff patch comment help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and __fish_seen_subcommand_from view" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and __fish_seen_subcommand_from view" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and __fish_seen_subcommand_from view" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and __fish_seen_subcommand_from view" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and __fish_seen_subcommand_from diff" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and __fish_seen_subcommand_from diff" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and __fish_seen_subcommand_from diff" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and __fish_seen_subcommand_from diff" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and __fish_seen_subcommand_from patch" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and __fish_seen_subcommand_from patch" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and __fish_seen_subcommand_from patch" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and __fish_seen_subcommand_from patch" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and __fish_seen_subcommand_from comment" -l body -d '评论内容（必填，与 `--body-file` 二选一）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and __fish_seen_subcommand_from comment" -l body-file -d '从文件读取评论内容（可选）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and __fish_seen_subcommand_from comment" -l path -d '文件路径（相对于仓库根目录，必填）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and __fish_seen_subcommand_from comment" -l line -d '评论的行号（1-based，必填）。' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and __fish_seen_subcommand_from comment" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and __fish_seen_subcommand_from comment" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and __fish_seen_subcommand_from comment" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and __fish_seen_subcommand_from comment" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and __fish_seen_subcommand_from help" -f -a "view" -d '查看 Commit 详情。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and __fish_seen_subcommand_from help" -f -a "diff" -d '获取 Commit 的 unified diff 输出。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and __fish_seen_subcommand_from help" -f -a "patch" -d '获取 Commit 的原始 patch 内容。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and __fish_seen_subcommand_from help" -f -a "comment" -d '评论 Commit 中的特定文件行。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand commit; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand skills" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand skills" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand skills" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand skills" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand run" -s n -l name -d 'Override the configuration name (defaults to the name from the layered config or `"gitflow-cli"`)' -r
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand run" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand run" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand run" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand run" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand completions" -l platform -d 'Override platform auto-detection' -r -f -a "github\t'GitHub (github.com or Enterprise)'
gitlab\t'GitLab (gitlab.com or self-hosted)'
gitcode\t'`GitCode` (gitcode.com or self-hosted)'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand completions" -l output -d 'Output format (json or text)' -r -f -a "json\t'Structured JSON output (default; for machine consumption by skills)'
text\t'Human-readable plain text output'"
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand completions" -s v -l verbose -d 'Enable verbose output'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand completions" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and not __fish_seen_subcommand_from issue pr release review auth label milestone commit skills run completions help" -f -a "issue" -d 'Issue operations (create, list, view)'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and not __fish_seen_subcommand_from issue pr release review auth label milestone commit skills run completions help" -f -a "pr" -d 'Pull request operations (create, list, view)'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and not __fish_seen_subcommand_from issue pr release review auth label milestone commit skills run completions help" -f -a "release" -d 'Release operations (create, list, view, edit, upload, download, delete)'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and not __fish_seen_subcommand_from issue pr release review auth label milestone commit skills run completions help" -f -a "review" -d 'Code review operations (comment, approve, request-changes, submit)'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and not __fish_seen_subcommand_from issue pr release review auth label milestone commit skills run completions help" -f -a "auth" -d 'Authentication operations (login, logout, status, token)'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and not __fish_seen_subcommand_from issue pr release review auth label milestone commit skills run completions help" -f -a "label" -d 'Label operations (create, list, edit, delete)'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and not __fish_seen_subcommand_from issue pr release review auth label milestone commit skills run completions help" -f -a "milestone" -d 'Milestone operations (create, list, edit, close, reopen)'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and not __fish_seen_subcommand_from issue pr release review auth label milestone commit skills run completions help" -f -a "commit" -d 'Commit operations (view, diff, patch, comment)'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and not __fish_seen_subcommand_from issue pr release review auth label milestone commit skills run completions help" -f -a "skills" -d 'Install gitflow skills to `~/.claude/skills/`'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and not __fish_seen_subcommand_from issue pr release review auth label milestone commit skills run completions help" -f -a "run" -d 'Run the main application workflow (deprecated)'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and not __fish_seen_subcommand_from issue pr release review auth label milestone commit skills run completions help" -f -a "completions" -d 'Generate shell completion scripts'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and not __fish_seen_subcommand_from issue pr release review auth label milestone commit skills run completions help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from issue" -f -a "create" -d '创建一个新的 Issue。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from issue" -f -a "list" -d '列出 Issue。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from issue" -f -a "view" -d '查看单个 Issue 详情。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from issue" -f -a "close" -d '关闭 Issue。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from issue" -f -a "reopen" -d '重新打开 Issue。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from issue" -f -a "comment" -d '评论 Issue。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from issue" -f -a "add-label" -d '为 Issue 添加标签。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from issue" -f -a "remove-label" -d '从 Issue 移除标签。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from pr" -f -a "create" -d '创建一条新的 Pull Request。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from pr" -f -a "list" -d '列出 Pull Request。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from pr" -f -a "view" -d '查看单个 Pull Request 详情。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from pr" -f -a "close" -d '关闭 Pull Request。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from pr" -f -a "reopen" -d '重新打开 Pull Request。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from pr" -f -a "comment" -d '评论 Pull Request。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from pr" -f -a "merge" -d '合并 Pull Request。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from pr" -f -a "checkout" -d '在本地检出 Pull Request 的分支。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from pr" -f -a "ready" -d '将草稿 PR 标记为可审查状态。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from pr" -f -a "wip" -d '将 PR 标记为草稿状态。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from pr" -f -a "sync" -d '同步 PR 的分支（将 base 分支的最新变更合入 head 分支）。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from release" -f -a "create" -d '创建一个新的 Release。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from release" -f -a "list" -d '列出 Release。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from release" -f -a "view" -d '查看指定 tag 的 Release 详情。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from release" -f -a "edit" -d '编辑指定 Release 的元数据。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from release" -f -a "upload" -d '上传资源文件到 Release。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from release" -f -a "download" -d '下载 Release 的资源文件。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from release" -f -a "delete" -d '删除指定 Release。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from review" -f -a "comment" -d '在 PR 上发表评论。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from review" -f -a "approve" -d '批准 PR。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from review" -f -a "request-changes" -d '要求对 PR 进行修改。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from review" -f -a "submit" -d '提交一次完整的 Review。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "login" -d '执行登录流程。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "logout" -d '执行登出流程，清除本地凭据。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "status" -d '查询当前认证状态。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "token" -d '获取当前有效的访问 Token。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from label" -f -a "create" -d '创建一个新的标签。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from label" -f -a "list" -d '列出仓库中的所有标签。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from label" -f -a "edit" -d '编辑一个已有的标签。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from label" -f -a "delete" -d '删除一个标签。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from milestone" -f -a "create" -d '创建一个新的里程碑。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from milestone" -f -a "list" -d '列出仓库中的所有里程碑。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from milestone" -f -a "edit" -d '编辑一个已有的里程碑。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from milestone" -f -a "close" -d '关闭一个里程碑。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from milestone" -f -a "reopen" -d '重新打开一个已关闭的里程碑。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from commit" -f -a "view" -d '查看 Commit 详情。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from commit" -f -a "diff" -d '获取 Commit 的 unified diff 输出。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from commit" -f -a "patch" -d '获取 Commit 的原始 patch 内容。'
complete -c gitflow-cli -n "__fish_gitflow_cli_using_subcommand help; and __fish_seen_subcommand_from commit" -f -a "comment" -d '评论 Commit 中的特定文件行。'
