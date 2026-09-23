# Ruby Quality Toolchain

**Shared language profile:** `gf-quality/references/profiles/ruby.md`. Read it before running these gates.

## Gate Commands

| # | Gate | Command | Pass Criteria |
|---|------|---------|---------------|
| 1 | build | `bundle check` | exit 0 (declared dependencies available; no install) |
| 2 | test | `bundle exec rspec` | all pass |
| 3 | coverage | `bundle exec rspec` with SimpleCov `minimum_coverage ${COV_THRESHOLD:-80}` in `spec/spec_helper.rb` | exit 0 (total line coverage ≥ threshold); N/A if no `.rb` in change set |
| 4 | format | `bundle exec rubocop --only Layout` | exit 0, no offenses |
| 5 | static | `bundle exec rubocop` | exit 0, no offenses |
| 6 | pre-commit | `pre-commit run --all-files` | all hooks pass (or N/A if no `.pre-commit-config.yaml`) |

## Gate Notes

- Run gates from the directory containing `Gemfile`.
- Gate 3 requires SimpleCov in `spec_helper.rb`; report SKIPPED if absent and N/A only when the change set has no `.rb` file.
- SimpleCov reports line coverage.

### Makefile-First Rule

If project root contains a `Makefile` with matching targets, prefer `make` commands over direct tool invocations:

| Gate | Preferred Command | Fallback |
|------|-------------------|----------|
| build | `make build` | `bundle check` |
| test | `make test` | `bundle exec rspec` |
| format | `make fmt` | `bundle exec rubocop --only Layout` |
| static | `make lint` | `bundle exec rubocop` |

Detection: `make -n <target> >/dev/null 2>&1` returns 0 → target exists.


## Forbidden Actions

- ❌ Never auto-fix with `rubocop -a` or `rubocop -A` — report only
- ❌ Never run `bundle update` — it mutates `Gemfile.lock` outside the user's intent
- ❌ Never modify `spec/spec_helper.rb` to lower `minimum_coverage`

## Quality Gate Configuration

### Configuration Examples

#### .rubocop.yml

```yaml
AllCops:
  NewCops: enable
  Exclude:
    - "vendor/**/*"
    - "db/schema.rb"
Metrics/MethodLength:
  Max: 20
```

#### spec/spec_helper.rb (SimpleCov)

```ruby
require "simplecov"

SimpleCov.start do
  add_filter "/spec/"
  minimum_coverage Integer(ENV.fetch("COV_THRESHOLD", "80"))
end
```

## Troubleshooting

### Common Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `bundler: command not found: rspec` | rspec not in bundle | See the shared Ruby profile |
| `Could not locate Gemfile` | Wrong working directory | `cd` to the directory containing `Gemfile` |
| `SimpleCov failed with exit 2` | Coverage below `minimum_coverage` | Add tests; do not lower the threshold |
| `Gemfile.lock out of date` | Dependencies drifted | Report to user — do NOT run `bundle update` |

### Exit Code Reference

| Code | Meaning | Action |
|------|---------|--------|
| 0 | Success | Continue to next gate |
| 1 | Test failure or RuboCop offense | Fix and re-run |
| 2 | SimpleCov below threshold | Add tests |

### FAQ

**Q: Why does coverage show 0%?**
A: `require "simplecov"` and `SimpleCov.start` must run **before** application code is loaded. Put them at the very top of `spec/spec_helper.rb`.

**Q: RuboCop and the formatter disagree?**
A: Gate 4 runs only `--only Layout`; Gate 5 runs the full rule set. Report both, fix neither.

### Performance Tips

- Use `bundle exec rspec --fail-fast` while iterating locally to stop at the first failure instead of running the full suite
- Use `bundle check` to verify installed dependencies without changing `Gemfile.lock`.
- Narrow SimpleCov's tracked files with `add_filter` (e.g. exclude `/spec/`, `/vendor/`) so coverage instrumentation only touches application code, reducing both runtime and noise
- Use `--only-failures` (RSpec's persistence feature, `config.example_status_persistence_file_path` in `spec_helper.rb`) to re-run just the specs that failed last time
