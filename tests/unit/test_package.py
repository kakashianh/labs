"""Unit tests for the initial package bootstrap."""

import protein3d
from protein3d.cli import build_parser


def test_package_exposes_version() -> None:
    assert protein3d.__version__ == "0.1.0"


def test_cli_parser_has_expected_program_name() -> None:
    parser = build_parser()
    assert parser.prog == "protein3d"
