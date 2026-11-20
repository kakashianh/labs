"""Smoke test for the installed command-line entry point."""

import subprocess

import pytest


@pytest.mark.integration
def test_cli_help_starts_successfully() -> None:
    result = subprocess.run(
        ["uv", "run", "--locked", "protein3d", "--help"],
        check=False,
        capture_output=True,
        text=True,
    )
    assert result.returncode == 0
    assert "Protein 3D structure" in result.stdout


@pytest.mark.integration
def test_alphafold_crawl_help_exposes_safe_sample_options() -> None:
    result = subprocess.run(
        ["uv", "run", "--locked", "protein3d", "alphafold-crawl", "--help"],
        check=False,
        capture_output=True,
        text=True,
    )
    assert result.returncode == 0
    assert "--sample-limit" in result.stdout
    assert "--manifest" in result.stdout
