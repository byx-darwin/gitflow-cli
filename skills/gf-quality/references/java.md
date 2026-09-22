# Java Quality Toolchain

**Shared language profile:** `gf-quality/references/profiles/java.md`. Read it before running these gates.

## Gate Commands

### Maven

| # | Gate | Command | Pass Criteria |
|---|------|---------|---------------|
| 1 | build | `mvn compile -q` | exit 0 |
| 2 | test | `mvn test` | all pass |
| 3 | coverage | `mvn verify -Pcoverage` (requires a JaCoCo `check` rule whose LINE COVEREDRATIO limit equals `COV_THRESHOLD/100`, e.g. `0.80` for the default 80) | exit 0 (total line coverage ≥ threshold); N/A if no `.java` in change set; SKIPPED if the JaCoCo plugin or its `check` rule is absent |
| 4 | format | `mvn spotless:check` or `mvn formatter:validate` | exit 0 |
| 5 | static | `mvn pmd:check` or `mvn spotbugs:check` | exit 0 |
| 6 | pre-commit | `pre-commit run --all-files` | all hooks pass (or N/A) |

### Gradle

| # | Gate | Command | Pass Criteria |
|---|------|---------|---------------|
| 1 | build | `./gradlew compileJava` | exit 0 |
| 2 | test | `./gradlew test` | all pass |
| 3 | coverage | `./gradlew jacocoTestReport jacocoTestCoverageVerification` (violationRules: LINE COVEREDRATIO limit equals `COV_THRESHOLD/100`, e.g. `0.80` for the default 80) | exit 0 (total line coverage ≥ threshold); N/A if no `.java` in change set; SKIPPED if the JaCoCo plugin or its `violationRules` are absent |
| 4 | format | `./gradlew spotlessCheck` | exit 0 |
| 5 | static | `./gradlew checkstyleMain` or `./gradlew pmdMain` | exit 0 |
| 6 | pre-commit | `pre-commit run --all-files` | all hooks pass (or N/A) |


## Gate Notes

- Gate 3 requires the JaCoCo plugin with a coverage `check` rule — if either is absent, mark SKIPPED
- Gate 3 is N/A when the change set contains no `.java` file — report N/A, not SKIPPED
- Gate 4: Spotless is preferred; fall back to formatter-maven-plugin
- Gate 5: try PMD first, then SpotBugs, then Checkstyle — use whatever is configured
- Check for existing config files (`spotbugs-exclude.xml`, `pmd-ruleset.xml`, etc.)
- Respect `maven.test.skip` property — if set, warn user that tests are being skipped

## Forbidden Actions

- ❌ Never run `mvn clean` without user confirmation
- ❌ Never modify `pom.xml` or `build.gradle` during quality check
- ❌ Never skip tests silently — if tests are skipped, report it

## Quality Gate Configuration

### Configuration Examples

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

### Coverage Threshold Mapping

JaCoCo has no command-line threshold flag: the limit lives in the `check` goal
(Maven) or `jacocoTestCoverageVerification` task (Gradle) inside the build file,
and it is expressed as a **fraction**, not a percentage. `COV_THRESHOLD=90`
therefore corresponds to a `LINE` `COVEREDRATIO` minimum of `0.90`.

Because Gate 3 must not modify `pom.xml` or `build.gradle` (see Forbidden
Actions), the skill does **not** rewrite the rule. Instead it reads the
configured `COVEREDRATIO` minimum and compares it against `COV_THRESHOLD/100`:

- Rule absent → Gate 3 is **SKIPPED** (no threshold was enforced).
- Rule present and its minimum ≥ `COV_THRESHOLD/100` → the build's own exit
  status is the gate result.
- Rule present but its minimum < `COV_THRESHOLD/100` → report the gate result
  together with the weaker configured limit, so a `COV_THRESHOLD=90` request is
  never reported as satisfied by an `0.80` rule.

Maven (`pom.xml`):

```xml
<execution>
  <id>check</id>
  <goals><goal>check</goal></goals>
  <configuration>
    <rules>
      <rule>
        <element>BUNDLE</element>
        <limits>
          <limit>
            <counter>LINE</counter>
            <value>COVEREDRATIO</value>
            <minimum>0.80</minimum>
          </limit>
        </limits>
      </rule>
    </rules>
  </configuration>
</execution>
```

Gradle (`build.gradle`):

```groovy
jacocoTestCoverageVerification {
    violationRules {
        rule {
            limit {
                counter = 'LINE'
                value = 'COVEREDRATIO'
                minimum = 0.80
            }
        }
    }
}
```

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
