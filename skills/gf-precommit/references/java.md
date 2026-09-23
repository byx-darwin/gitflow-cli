# Java Pre-commit Checks

**Shared language profile:** `gf-quality/references/profiles/java.md`. Read it for tools, version sources, and scan exclusions.


## Maven Commands

| Check | Command | Fix Command |
|-------|---------|-------------|
| format | `mvn spotless:check` | `mvn spotless:apply` |
| lint | `mvn pmd:check` | — |
| test | `mvn test` | — |

## Gradle Commands

| Check | Command | Fix Command |
|-------|---------|-------------|
| format | `./gradlew spotlessCheck` | `./gradlew spotlessApply` |
| lint | `./gradlew checkstyleMain` | — |
| test | `./gradlew test` | — |

## Notes

- Fix commands require user confirmation before execution
- Never run `mvn clean` or `gradlew clean` without user confirmation
- Respect existing plugin configuration
