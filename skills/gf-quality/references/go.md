# Go Quality Toolchain

**Detection:** `go.mod` in project root.

## Gate Commands

| # | Gate | Command | Pass Criteria |
|---|------|---------|---------------|
| 1 | build | `go build ./...` | exit 0 |
| 2 | test | `go test ./... -race -count=1` | all pass |
| 3 | coverage | `go test ./... -coverprofile=coverage.out && go tool cover -func=coverage.out \| grep total` | incremental ≥ 80% |
| 4 | format | `gofmt -l .` | no output (all formatted) |
| 5 | static | `go vet ./...` then `golangci-lint run ./...` | exit 0 |
| 6 | pre-commit | `pre-commit run --all-files` | all hooks pass (or N/A) |

## Tool Installation

| Tool | Install Command | Required By |
|------|----------------|-------------|
| golangci-lint | `go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest` | Gate 5 |

If golangci-lint is missing, fall back to `go vet ./...` only.

## Notes

- Gate 2 includes `-race` for race condition detection
- Gate 3: compare against previous run; incremental coverage ≥ 80%
- Gate 4: auto-fix with `gofmt -w .` only after user confirmation
- Gate 5: `staticcheck ./...` as fallback if golangci-lint unavailable

## Forbidden Actions

- ❌ Never run `go clean -modcache`
- ❌ Never auto-fix without showing diff first
