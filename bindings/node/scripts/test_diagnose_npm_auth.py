"""Authentication probes must remain dry-runs and keep credentials out of output."""

import contextlib
import io
import subprocess
import unittest
from unittest.mock import patch

import diagnose_npm_auth as diagnostic
from npm_release import NAMES


class DiagnosticTests(unittest.TestCase):
    def run_probe(self, stderr, **changes):
        record = {"version": "0.3.1", "ref": "npm-v0.3.1", "commit": "a" * 40,
                  "run_id": "123", "packages": [{"name": name, "filename": f"{name}-0.3.1.tgz"} for name in NAMES]}
        environment = {"GITHUB_ACTIONS": "true", "ACTIONS_ID_TOKEN_REQUEST_URL": "https://example.invalid/",
                       "ACTIONS_ID_TOKEN_REQUEST_TOKEN": "secret-request-token"}
        environment.update(changes)
        self.output = io.StringIO()
        self.command = self.enterContext(patch.object(diagnostic.subprocess, "run",
            return_value=subprocess.CompletedProcess([], 0, stdout="secret-output", stderr=stderr)))
        self.enterContext(patch.object(diagnostic, "verify_local", return_value=record))
        self.enterContext(patch.object(diagnostic.os, "environ", environment))
        self.enterContext(patch("sys.argv", ["diagnose_npm_auth.py", "--directory", ".", "--run-id", "123"]))
        with contextlib.redirect_stdout(self.output):
            diagnostic.main()

    def test_success_always_uses_dry_run_and_redacts_unrelated_output(self):
        self.run_probe("npm verbose oidc Successfully retrieved and set token\nBearer secret-response-token")
        commands = [call.args[0] for call in self.command.call_args_list if call.args[0][:2] == ["npm", "publish"]]
        self.assertEqual(len(commands), 5)
        for command in commands:
            self.assertIn("--dry-run", command)
            self.assertIn("--ignore-scripts", command)
            self.assertIn("https://registry.npmjs.org/", command)
        self.assertNotIn("secret", self.output.getvalue())

    def test_failed_exchange_is_reported_even_if_npm_dry_run_exits_zero(self):
        with self.assertRaisesRegex(ValueError, "OIDC authentication failed"):
            self.run_probe("npm verbose oidc Failed token exchange request with body message: Invalid publisher")
        self.assertIn("Invalid publisher", self.output.getvalue())
        self.assertEqual(self.command.call_count, 7)

    def test_missing_oidc_permissions_stops_before_running_npm(self):
        with self.assertRaisesRegex(ValueError, "Missing GitHub OIDC environment"):
            self.run_probe("", ACTIONS_ID_TOKEN_REQUEST_TOKEN="")
        self.command.assert_not_called()


if __name__ == "__main__":
    unittest.main()
