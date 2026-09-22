"""Fixture checks for read-only skill permission grants."""

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).resolve().parents[1] / "validate-skill-permissions.py"


class SkillPermissionTests(unittest.TestCase):
    def run_validator(
        self, body: str, grants: str, skill_name: str = "gf-example"
    ) -> subprocess.CompletedProcess[str]:
        with tempfile.TemporaryDirectory() as temporary:
            skill = Path(temporary) / skill_name
            skill.mkdir()
            (skill / "SKILL.md").write_text(
                f"---\nname: {skill_name}\ndescription: Example\n{grants}---\n\n{body}\n",
                encoding="utf-8",
            )
            return subprocess.run(
                [sys.executable, str(SCRIPT), "--root", temporary],
                capture_output=True,
                text=True,
                check=False,
            )

    def test_should_reject_write_grant_for_read_only_skill(self) -> None:
        result = self.run_validator("Read-only analysis.", "allowed-tools: Read, Write\n")
        self.assertEqual(result.returncode, 1)
        self.assertIn("gf-example", result.stdout)

    def test_should_reject_edit_grant_for_never_modify_skill(self) -> None:
        result = self.run_validator("Never modifies files.", "allowed-tools: [Read, Edit]\n")
        self.assertEqual(result.returncode, 1)

    def test_should_reject_write_grant_in_yaml_list(self) -> None:
        result = self.run_validator(
            "Read-only analysis.", "allowed-tools:\n  - Read\n  - Write\n"
        )
        self.assertEqual(result.returncode, 1)

    def test_should_allow_read_only_grants(self) -> None:
        result = self.run_validator("Read-only analysis.", "allowed-tools: Read, Grep, Glob\n")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_should_allow_documented_write_scope(self) -> None:
        result = self.run_validator(
            "Writes an audit report to disk.", "allowed-tools: Read, Write\n"
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_should_require_direct_denials_on_restricted_skill(self) -> None:
        result = self.run_validator(
            "Read-only security audit.",
            "allowed-tools: Read, Grep, Glob\n",
            "gf-security-check",
        )
        self.assertEqual(result.returncode, 1)
        self.assertIn("disallowed-tools", result.stdout)

    def test_should_reject_bash_preapproval_on_restricted_skill(self) -> None:
        result = self.run_validator(
            "Read-only security audit.",
            "allowed-tools: Read, Bash\ndisallowed-tools: Write, Edit\n",
            "gf-security-check",
        )
        self.assertEqual(result.returncode, 1)
        self.assertIn("Bash", result.stdout)

    def test_should_reject_missing_read_only_scope_on_restricted_skill(self) -> None:
        result = self.run_validator(
            "Audit security findings.",
            "allowed-tools: Read, Grep, Glob\ndisallowed-tools: Write, Edit\n",
            "gf-security-check",
        )
        self.assertEqual(result.returncode, 1)
        self.assertIn("read-only", result.stdout)


if __name__ == "__main__":
    unittest.main()
