"""Command-line bootstrap for the Protein3D-PPI workspace."""

from __future__ import annotations

import argparse


def build_parser() -> argparse.ArgumentParser:
    """Build the command-line argument parser."""
    parser = argparse.ArgumentParser(
        prog="protein3d",
        description="Protein 3D structure and protein-protein interaction workspace",
    )
    parser.add_argument(
        "--version",
        action="version",
        version="%(prog)s 0.1.0",
    )
    return parser


def main() -> int:
    """Run the CLI bootstrap. Domain commands are added by downstream tasks."""
    parser = build_parser()
    parser.parse_args()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
