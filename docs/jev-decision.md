# Optional Jev decisions (#382)

`gf decide` is an optional, read-only typed decision interface. The default
binary builds without the Jev adapter. Installed users can run `cargo install
gitflow-cli --features gitflow-jev --force`; repository developers can build
with `cargo build -p gitflow-cli --features gitflow-jev`. Set
`GF_DECISION_PROVIDER=jev` to enable the adapter at runtime.
Place a newly generated TypeSafe key in the local `TYPESAFE_API_KEY` environment
variable. On macOS, the adapter also reads the `ai.typesafe.api-key` generic
password item for the current `$USER` account from Keychain when that
environment variable is absent or empty. The environment variable takes
precedence. Run `security add-generic-password -a "$USER" -s
ai.typesafe.api-key -U -w` to store a prompted value.

During the compatibility period, migrate the legacy item to the shared service
name with:

```sh
security add-generic-password -a "$USER" -s ai.typesafe.api-key -U -w "$(security find-generic-password -a "$USER" -s gitflow-cli-typesafe -w)"
```

Do not put the key in command arguments, request files, chat, or logs. `gf auth`
credentials are separate from TypeSafe credentials.

The CLI accepts a JSON `DecisionRequest` from standard input or `--input PATH`.
`gf decide noul`, `choice`, and `score` require every question to have the named
type; `gf decide batch` accepts a mix. Each invocation sends one request to
TypeSafe and emits a validated JSON response. Example:

```json
{
  "state": {"title": "CI crashes on startup"},
  "questions": {
    "blocked": {"type": "noul", "instructions": "Is work on this issue blocked?"}
  }
}
```

The request limit is 128 KiB at the CLI boundary, 32 KiB for serialized state,
and 16 questions. Choice allows 2–32 options; Score allows 2–10 ordered levels.
Empty state and several obvious credential shapes are rejected before network
transmission. This check is a last guard; review and remove sensitive content
from the state yourself because no pattern matcher can recognize every secret.
Responses are limited to 128 KiB, validated against the exact question set,
and must arrive within 10 seconds by default. `GF_JEV_TIMEOUT_MS` may be set
from 100 to 30000. `GF_JEV_MODEL` defaults to `jev-latest`.

Provider failures yield content-free errors. Callers should continue with their
existing deterministic or human review workflow; Jev output does not authorize
state changes. The `gf-issue-triage` Skill uses one batch of four questions as
optional guidance. Review and redact Issue excerpts before transmission. It
does not automatically map Score or Noul values to labels. Each mapping needs
its own threshold calibrated on labeled repository Issues.

For triage, construct the request below from the Issue fetched by `gf issue
view`. Replace the example title and body with a short, reviewed excerpt.
Only `state.title` and `state.body_excerpt` should be sent; omit the author,
assignees, URLs, comments, and raw platform payload.

```json
{
  "state": {
    "title": "Example issue title",
    "body_excerpt": "Reviewed excerpt with secrets and personal data removed"
  },
  "questions": {
    "type": {
      "type": "choice",
      "instructions": "Which issue type best describes the work in state? Choose other when none fit.",
      "criteria": {
        "bug": "Existing behavior is broken or regressed",
        "feature": "A new capability is requested",
        "enhancement": "An existing capability should improve",
        "docs": "Documentation needs to be written or corrected",
        "question": "The issue asks for an answer or discussion",
        "other": "None of the listed types fits"
      }
    },
    "priority": {
      "type": "score",
      "instructions": "Rate the impact and urgency of this issue using only evidence in state.",
      "criteria": [
        "Low: minor polish or documentation, no blocked workflow",
        "Medium: useful improvement, workaround or limited impact",
        "High: important workflow affected or near-term delivery risk",
        "Urgent: production outage, confirmed security exposure, or critical work blocked"
      ]
    },
    "security_related": {
      "type": "noul",
      "instructions": "Does state provide concrete evidence that this issue concerns a security vulnerability or exposure?"
    },
    "blocked": {
      "type": "noul",
      "instructions": "Does state say implementation or users are currently blocked by this issue?"
    }
  }
}
```

The adapter uses TypeSafe's documented `POST /v1/systemone` contract. See the
[official API reference](https://docs.typesafe.ai/api) and
[TypeSafe Skill](https://github.com/typesafe-ai/skills/blob/main/skills/typesafe-ai/SKILL.md).

## Evaluation status

Offline tests cover schema validation, valid and malformed provider responses,
missing key, timeout, and oversized state. The [pilot evaluation](./jev-triage-evaluation-2026-09-22.md)
reports a live run on 22 labeled public Issues and separates six synthetic
boundary cases. No automatic labeling threshold has been deployed.
