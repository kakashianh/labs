"""Command-line entry points for the Protein3D-PPI workspace."""

from __future__ import annotations

import argparse
from datetime import UTC, datetime
from pathlib import Path

from protein3d.config import get_settings


def build_parser() -> argparse.ArgumentParser:
    """Build the command-line argument parser."""
    parser = argparse.ArgumentParser(
        prog="protein3d",
        description="Protein 3D structure and protein-protein interaction workspace",
    )
    parser.add_argument("--version", action="version", version="%(prog)s 0.1.0")
    commands = parser.add_subparsers(dest="command")

    crawl = commands.add_parser(
        "alphafold-crawl",
        help="crawl a bounded cohort of AlphaFold DB structure predictions",
    )
    crawl.add_argument(
        "--sample-limit",
        type=int,
        default=None,
        help="maximum resolved UniProt mappings to process (default: configured safe limit)",
    )
    crawl.add_argument(
        "--manifest",
        type=Path,
        default=None,
        help="JSONL manifest output path (default: data/manifests with UTC timestamp)",
    )
    return parser


def main() -> int:
    """Run the selected domain command."""
    parser = build_parser()
    args = parser.parse_args()
    if args.command == "alphafold-crawl":
        return _run_alphafold_crawl(args.sample_limit, args.manifest)
    return 0


def _run_alphafold_crawl(sample_limit: int | None, manifest: Path | None) -> int:
    from protein3d.ingestion.alphafold_crawler import run_configured_crawl
    from protein3d.metadata.database import build_engine, build_session_factory, session_scope

    settings = get_settings()
    manifest_path = manifest or Path("data/manifests") / (
        f"alphafold-{datetime.now(UTC).strftime('%Y%m%dT%H%M%SZ')}.jsonl"
    )
    engine = build_engine(settings)
    factory = build_session_factory(engine)
    try:
        with session_scope(factory) as session:
            summary = run_configured_crawl(
                session,
                settings,
                manifest_path,
                sample_limit=sample_limit,
            )
        print(
            "AlphaFold crawl finished: "
            f"discovered={summary.discovered}, downloaded={summary.downloaded}, "
            f"no_model={summary.no_model}, unsupported={summary.unsupported}, "
            f"failed={summary.failed}, "
            f"dataset_run_id={summary.dataset_run_id}, job_id={summary.job_id}, "
            f"manifest={summary.manifest_path}"
        )
        return 1 if summary.failed else 0
    finally:
        engine.dispose()


if __name__ == "__main__":
    raise SystemExit(main())
