# Language Detection

Detect project language by scanning for marker files in the project root.

## Detection Rules

Check these files **in order**. First match wins:

| Marker File | Language | Load Reference |
|-------------|----------|----------------|
| `Cargo.toml` | Rust | `references/rust.md` |
| `go.mod` | Go | `references/go.md` |
| `pom.xml` | Java (Maven) | `references/java.md` |
| `build.gradle` / `build.gradle.kts` | Java (Gradle) | `references/java.md` |
| `pyproject.toml` | Python | `references/python.md` |
| `setup.py` / `setup.cfg` | Python | `references/python.md` |
| `package.json` | Node.js / TypeScript | `references/node.md` |
| `Gemfile` | Ruby | `references/ruby.md` |

## Multi-Language Projects

If multiple marker files exist (e.g., a Rust project with a Node.js frontend):

1. List all detected languages to the user
2. Ask which one to run quality checks for
3. If the user says "all" → run each language's gates in sequence

## No Marker File (Generic)

If no marker file is found:

1. Check for `.pre-commit-config.yaml` → run `pre-commit run --all-files` only
2. If no pre-commit config either → report "No project detected, no quality gates to run"

## Detection Command

```bash
# One-liner for quick detection
for f in Cargo.toml go.mod pom.xml build.gradle build.gradle.kts pyproject.toml setup.py setup.cfg package.json Gemfile; do
  [ -f "$f" ] && echo "DETECTED: $f"
done
```
