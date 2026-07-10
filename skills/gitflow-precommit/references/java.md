# Java Pre-commit Checks

**Detection:** `pom.xml` (Maven) or `build.gradle` / `build.gradle.kts` (Gradle) in project root.

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
