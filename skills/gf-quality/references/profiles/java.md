# Java Language Profile

Shared facts for the Java language layers in `gf-quality`, `gf-precommit`, and `gf-smell`.

## Version Source

- Read the required Java version from the build configuration and the active JDK from `java -version`; do not pin an example version as the project requirement.

## Tools and Installation

| Tool | Source or installation guidance | Used by |
|---|---|---|
| Maven or Gradle wrapper | Use the project's configured build tool and wrapper when present | build and tests |
| JaCoCo, Spotless, PMD, SpotBugs | Use only plugins already configured in `pom.xml` or Gradle build files | coverage, format, static checks |

Missing plugins cause the corresponding gate to be reported as skipped. Do not install or modify build plugins automatically. Read `JAVA_HOME`, `MAVEN_OPTS`, and `GRADLE_OPTS` from the environment when relevant.
