# Java Quality Toolchain

**Detection:** `pom.xml` (Maven) or `build.gradle` / `build.gradle.kts` (Gradle) in project root.

## Gate Commands

### Maven

| # | Gate | Command | Pass Criteria |
|---|------|---------|---------------|
| 1 | build | `mvn compile -q` | exit 0 |
| 2 | test | `mvn test` | all pass |
| 3 | coverage | `mvn verify -Pcoverage` (requires JaCoCo) | incremental ≥ 80% |
| 4 | format | `mvn spotless:check` or `mvn formatter:validate` | exit 0 |
| 5 | static | `mvn pmd:check` or `mvn spotbugs:check` | exit 0 |
| 6 | pre-commit | `pre-commit run --all-files` | all hooks pass (or N/A) |

### Gradle

| # | Gate | Command | Pass Criteria |
|---|------|---------|---------------|
| 1 | build | `./gradlew compileJava` | exit 0 |
| 2 | test | `./gradlew test` | all pass |
| 3 | coverage | `./gradlew jacocoTestReport` | incremental ≥ 80% |
| 4 | format | `./gradlew spotlessCheck` | exit 0 |
| 5 | static | `./gradlew checkstyleMain` or `./gradlew pmdMain` | exit 0 |
| 6 | pre-commit | `pre-commit run --all-files` | all hooks pass (or N/A) |

## Tool Installation

Most tools are Maven/Gradle plugins — no separate install needed. If a plugin is missing, report N/A for that gate.

## Notes

- Gate 3: requires JaCoCo plugin configured; if not present, mark N/A
- Gate 4: Spotless is preferred; fall back to formatter-maven-plugin
- Gate 5: try PMD first, then SpotBugs, then Checkstyle — use whatever is configured
- Check for existing config files (`spotbugs-exclude.xml`, `pmd-ruleset.xml`, etc.)
- Respect `maven.test.skip` property — if set, warn user that tests are being skipped

## Forbidden Actions

- ❌ Never run `mvn clean` without user confirmation
- ❌ Never modify `pom.xml` or `build.gradle` during quality check
- ❌ Never skip tests silently — if tests are skipped, report it
