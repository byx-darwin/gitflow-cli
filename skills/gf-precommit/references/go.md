# Go Pre-commit Checks

**Shared language profile:** `gf-quality/references/profiles/go.md`. Read it for tools, version sources, and scan exclusions.


## Check Commands

| Check | Command | Fix Command |
|-------|---------|-------------|
| format | `gofmt -l .` | `gofmt -w .` |
| lint | `go vet ./...` | — |
| test | `go test ./... -race -count=1` | — |

## Notes

- If `golangci-lint` is installed, use `golangci-lint run ./...` instead of `go vet`
- Fix commands require user confirmation before execution
