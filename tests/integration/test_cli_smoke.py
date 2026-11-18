"""Smoke test for the installed command-line entry point."""

import subprocess
import sys

import pytest


@pytest.mark.integration
def test_cli_help_starts_successfully() -> None:
    result = subprocess.run(
        [sys.executable, "-m", "protein3d.cli", "--help"],
        check=False,
        capture_output=True,
        text=True,
    )
    assert result.returncode == 0
    assert "Protein 3D structure" in result.stdout
