# Go Pre-commit Checks

**Detection:** `go.mod` in project root.

## Check Commands

| Check | Command | Fix Command |
|-------|---------|-------------|
| format | `gofmt -l .` | `gofmt -w .` |
| lint | `go vet ./...` | — |
| test | `go test ./... -race -count=1` | — |

## Notes

- If `golangci-lint` is installed, use `golangci-lint run ./...` instead of `go vet`
- Fix commands require user confirmation before execution
