# Go Language Profile

Shared facts for the Go language layers in `gf-quality`, `gf-precommit`, and `gf-smell`.

## Version Source

- Read the `go` directive from the target project's `go.mod` and compare it with `go version`; do not assume a fixed version.

## Tools and Installation

| Tool | Source or installation guidance | Used by |
|---|---|---|
| gofmt, go vet, go test, go tool cover | Go toolchain | format, static checks, tests, coverage |
| golangci-lint | Recommend `go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest` when absent | quality static gate |
| gocyclo | Recommend `go install github.com/fzipp/gocyclo/cmd/gocyclo@latest` when absent | smell detection |
| staticcheck | Recommend `go install honnef.co/go/tools/cmd/staticcheck@latest` when absent | static or dead-code analysis |

Never install tools automatically. Relevant module environment variables include `GOPROXY`, `GOPRIVATE`, `GONOSUMDB`, and `GOSUMDB`; read effective values with `go env` without printing credentials.
