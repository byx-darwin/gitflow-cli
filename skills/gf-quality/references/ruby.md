# Ruby Quality Toolchain

**Detection:** `Gemfile` in project root.

## Gate Commands

| # | Gate | Command | Pass Criteria |
|---|------|---------|---------------|
| 1 | build | `bundle install --quiet` | exit 0 |
| 2 | test | `bundle exec rspec` | all pass |
| 3 | coverage | `bundle exec rspec` with SimpleCov `minimum_coverage ${COV_THRESHOLD:-80}` in `spec/spec_helper.rb` | exit 0 (total line coverage ≥ threshold); N/A if no `.rb` in change set |
| 4 | format | `bundle exec rubocop --only Layout` | exit 0, no offenses |
| 5 | static | `bundle exec rubocop` | exit 0, no offenses |
| 6 | pre-commit | `pre-commit run --all-files` | all hooks pass (or N/A if no `.pre-commit-config.yaml`) |

## Tool Installation

| Tool | Install Command | Required By |
|------|----------------|-------------|
| bundler | `gem install bundler` | Gates 1–5 |
| rspec | `bundle add rspec --group development,test` | Gates 2, 3 |
| simplecov | `bundle add simplecov --group test` | Gate 3 (coverage) |
| rubocop | `bundle add rubocop --group development` | Gates 4, 5 |

If a tool is missing, **warn the user and recommend install** — do NOT auto-install.

## Environment Variables

| Variable | Effect | Default |
|----------|--------|---------|
| `COV_THRESHOLD` / `COVERAGE_THRESHOLD` | Override coverage threshold | 80% |
| `BUNDLE_GEMFILE` | Alternate Gemfile location | `./Gemfile` |
| `RAILS_ENV` / `RACK_ENV` | Environment for test runs | `test` |

## Forbidden Actions

- ❌ Never auto-fix with `rubocop -a` or `rubocop -A` — report only
- ❌ Never run `bundle update` — it mutates `Gemfile.lock` outside the user's intent
- ❌ Never modify `spec/spec_helper.rb` to lower `minimum_coverage`

## Makefile-First Rule

If project root contains a `Makefile` with matching targets, prefer `make` commands over direct tool invocations:

| Gate | Preferred Command | Fallback |
|------|-------------------|----------|
| build | `make build` | `bundle install --quiet` |
| test | `make test` | `bundle exec rspec` |
| format | `make fmt` | `bundle exec rubocop --only Layout` |
| static | `make lint` | `bundle exec rubocop` |

Detection: `make -n <target> >/dev/null 2>&1` returns 0 → target exists.

## Configuration

### Config File Examples

#### `.rubocop.yml`

```yaml
AllCops:
  NewCops: enable
  Exclude:
    - "vendor/**/*"
    - "db/schema.rb"
Metrics/MethodLength:
  Max: 20
```

#### `spec/spec_helper.rb` (SimpleCov)

```ruby
require "simplecov"

SimpleCov.start do
  add_filter "/spec/"
  minimum_coverage Integer(ENV.fetch("COV_THRESHOLD", "80"))
end
```

### Language-Specific Notes

- Gate 3 requires `simplecov` wired into `spec_helper.rb` — if absent, mark SKIPPED
- Gate 3 is N/A when the change set contains no `.rb` file — report N/A, not SKIPPED
- SimpleCov reports **line** coverage, matching the other language layers
- Run gates from the directory containing `Gemfile`

## Troubleshooting

### Common Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `bundler: command not found: rspec` | rspec not in bundle | `bundle add rspec --group development,test` |
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
