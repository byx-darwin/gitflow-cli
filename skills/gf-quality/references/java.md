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

## Configuration

### Tool Setup

| Tool | Install | Config File | Required |
|------|---------|-------------|----------|
| JaCoCo | Maven/Gradle plugin | `pom.xml` or `build.gradle` | Gate 3 |
| Spotless | Maven/Gradle plugin | `pom.xml` or `build.gradle` | Gate 4 |
| PMD | Maven/Gradle plugin | `pmd-ruleset.xml` | Gate 5 |
| SpotBugs | Maven/Gradle plugin | `spotbugs-exclude.xml` | Gate 5 (fallback) |

### Config File Examples

#### pom.xml (Maven)

```xml
<project>
  <build>
    <plugins>
      <plugin>
        <groupId>org.jacoco</groupId>
        <artifactId>jacoco-maven-plugin</artifactId>
        <version>0.8.11</version>
        <executions>
          <execution>
            <goals>
              <goal>prepare-agent</goal>
            </goals>
          </execution>
          <execution>
            <id>report</id>
            <phase>test</phase>
            <goals>
              <goal>report</goal>
            </goals>
          </execution>
        </executions>
      </plugin>
    </plugins>
  </build>
</project>
```

#### build.gradle (Gradle)

```groovy
plugins {
    id 'jacoco'
    id 'com.diffplug.spotless' version '6.25.0'
}

jacoco {
    toolVersion = "0.8.11"
}

spotless {
    java {
        googleJavaFormat()
    }
}
```

#### spotbugs-exclude.xml

```xml
<FindBugsFilter>
  <Match>
    <Class name="~.*\.*Test"/>
  </Match>
</FindBugsFilter>
```

### Environment Variables

| Variable | Effect | Default |
|----------|--------|---------|
| `MAVEN_OPTS` | Maven JVM options | — |
| `GRADLE_OPTS` | Gradle JVM options | — |
| `JAVA_HOME` | JDK location | — |

### Language-Specific Notes

- Most tools are Maven/Gradle plugins — no separate install needed
- Gate 3: requires JaCoCo plugin configured; if not present, mark N/A
- Gate 4: Spotless is preferred; fall back to formatter-maven-plugin
- Gate 5: try PMD first, then SpotBugs, then Checkstyle — use whatever is configured
- Check for existing config files (`spotbugs-exclude.xml`, `pmd-ruleset.xml`, etc.)
- Respect `maven.test.skip` property — if set, warn user that tests are being skipped

## Troubleshooting

### Common Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `Plugin not found` | Plugin not configured | Add plugin to `pom.xml` or `build.gradle` |
| `./gradlew: Permission denied` | Wrapper not executable | `chmod +x gradlew` |
| `java.lang.OutOfMemoryError` | JVM memory issue | Increase heap: `MAVEN_OPTS="-Xmx2g"` |
| `BUILD FAILURE` | Build error | Read error message, fix code or config |

### Exit Code Reference

| Code | Meaning | Action |
|------|---------|--------|
| 0 | Success | Continue to next gate |
| 1 | Build failure | Fix errors and retry |
| 2 | Test failure | Fix failing tests |
| 137 | OOM killed | Increase JVM heap size |

### FAQ

**Q: Maven vs Gradle?**
A: Maven is XML-based, convention over configuration. Gradle is Groovy/Kotlin-based, more flexible.

**Q: How to skip tests temporarily?**
A: Maven: `mvn install -DskipTests`. Gradle: `./gradlew build -x test`. Warning: report will show tests SKIPPED.

**Q: JaCoCo coverage report location?**
A: Maven: `target/site/jacoco/index.html`. Gradle: `build/reports/jacoco/test/html/index.html`.

### Performance Tips

- Maven parallel builds: `mvn -T 1C` (1 thread per CPU core)
- Gradle daemon: enabled by default, speeds up builds
- Enable incremental compilation in Gradle: `org.gradle.caching=true`
- Use `mvn clean install` only when necessary; prefer `mvn install` for incremental builds
