# Go Quality Toolchain

**Detection:** `go.mod` in project root.

## Gate Commands

| # | Gate | Command | Pass Criteria |
|---|------|---------|---------------|
| 1 | build | `go build ./...` | exit 0 |
| 2 | test | `go test ./... -race -count=1` | all pass |
| 3 | coverage | `go test ./... -coverprofile=coverage.out && go tool cover -func=coverage.out \| awk -v t="${COV_THRESHOLD:-80}" '/^total:/ {gsub(/%/,"",$3); exit ($3+0 < t+0)}'` | exit 0 (total **statement** coverage ≥ threshold); N/A if no `.go` in change set |
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
- Gate 3: total **statement** coverage ≥ `COV_THRESHOLD` (default 80%); N/A if no `.go` in change set
- Gate 3 reports SKIPPED if `go tool cover` is unavailable — N/A is reserved for a change set with no `.go` file
- Go's coverage unit is the **statement**, not the line: `go tool cover -func` prints `total:\t(statements)\t50.0%`, and the Go toolchain has no line-coverage mode. The number is therefore not directly comparable to the line-coverage figures reported for Rust, Python, Java, Ruby and Node.js — state the unit whenever a Go coverage value appears in a report
- Gate 4: report the `gofmt -l .` file list — never run `gofmt -w .`
- Gate 5: `staticcheck ./...` as fallback if golangci-lint unavailable

## Forbidden Actions

- ❌ Never run `go clean -modcache`
- ❌ Never auto-fix with `gofmt -w .` — report only

## Configuration

### Tool Setup

| Tool | Install | Config File | Required |
|------|---------|-------------|----------|
| golangci-lint | `go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest` | `.golangci.yml` | Gate 5 |
| gofmt | Included with Go | — | Gate 4 |
| go vet | Included with Go | — | Gate 5 |

### Config File Examples

#### .golangci.yml

```yaml
linters:
  enable:
    - gofmt
    - govet
    - staticcheck
    - errcheck
  disable:
    - gocyclo
  fast: true

linters-settings:
  gofmt:
    simplify: true
```

#### go.mod

```go
module github.com/example/project

go 1.21

require (
    github.com/stretchr/testify v1.8.4
)
```

### Environment Variables

| Variable | Effect | Default |
|----------|--------|---------|
| `COV_THRESHOLD` / `COVERAGE_THRESHOLD` | Override coverage threshold (compared against Go's **statement** coverage) | 80% |
| `GOPROXY` | Go module proxy | `https://proxy.golang.org` |
| `GONOSUMCHECK` | Skip checksum verification | — |
| `GOFLAGS` | Default go command flags | — |

### Language-Specific Notes

- Gate 2 includes `-race` for race condition detection
- Gate 3: total **statement** coverage ≥ `COV_THRESHOLD` (default 80%); N/A if no `.go` in change set
- Gate 3 reports SKIPPED if `go tool cover` is unavailable — N/A is reserved for a change set with no `.go` file
- Go's coverage unit is the **statement**, not the line: `go tool cover -func` prints `total:\t(statements)\t50.0%`, and the Go toolchain has no line-coverage mode. The number is therefore not directly comparable to the line-coverage figures reported for Rust, Python, Java, Ruby and Node.js — state the unit whenever a Go coverage value appears in a report
- Gate 4: report the `gofmt -l .` file list — never run `gofmt -w .`
- Gate 5: `staticcheck ./...` as fallback if golangci-lint unavailable

## Troubleshooting

### Common Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `golangci-lint: command not found` | Tool not installed | `go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest` |
| `go: downloading: module not found` | Module proxy issue | Check `GOPROXY` or use `go mod vendor` |
| `FAIL: TestX (0.00s)` | Test failure | Run `go test -v ./...` for details |
| `race detected` | Race condition | Fix concurrent access patterns |

### Exit Code Reference

| Code | Meaning | Action |
|------|---------|--------|
| 0 | Success | Continue to next gate |
| 1 | Build/test failure | Fix errors and retry |
| 2 | Vet errors | Fix vet warnings |

### FAQ

**Q: Why does `go test` hang?**
A: Check for deadlocks or infinite loops. Use `go test -timeout 30s` to set a timeout.

**Q: How to update dependencies?**
A: Run `go get -u ./...` then `go mod tidy`.

**Q: Module proxy issues?**
A: Set `GOPROXY=https://goproxy.cn` (China) or use `go mod vendor` for offline builds.

### Performance Tips

- Use `go test -parallel 4` for parallel test execution
- Enable build caching: Go caches builds automatically
- Use `go mod vendor` for offline builds and faster CI
- Run `go clean -testcache` to clear test cache if needed
