# Node.js and TypeScript Language Profile

Shared facts for the Node.js language layers in `gf-quality`, `gf-precommit`, and `gf-smell`.

## Version Source

- Read `engines` and `packageManager` from `package.json`, then check the active runtime version. Do not assume a fixed version.
- Select the package manager from the lockfile in this order: `bun.lockb` or `bun.lock` → Bun; `pnpm-lock.yaml` → pnpm; `yarn.lock` → Yarn; `package-lock.json` → npm. If no lockfile exists, use npm. Do not mix managers in one project.

## Tools and Installation

| Tool | Source or installation guidance | Used by |
|---|---|---|
| prettier | Recommend adding to the project's dev dependencies with its selected package manager | format |
| eslint | Recommend adding to the project's dev dependencies with its selected package manager | static checks |
| typescript | Recommend adding to the project's dev dependencies with its selected package manager | TypeScript build checks |

Never install packages automatically. Read configured scripts and tool versions from `package.json` and the matching lockfile; do not prescribe a package manager's install command as a quality gate.
