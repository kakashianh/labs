# PP-9 — Initialize repository and Python development environment

## 1. Objective

Establish a reproducible, maintainable Python workspace that every later data pipeline, protein-structure processing, graph, ML, and API task can build on.

## 2. Scope and design context

Create the repository layout, Python compatibility policy, dependency lock strategy, environment template, developer quality gates, CI baseline, and test conventions. Do not implement domain pipelines or prematurely create domain-specific modules owned by PP-10 and later tasks.

**Upstream dependencies:** None; this is the starting task.

**Related plans:** [PP-10](../PP-10%20Design%20project%20architecture%20and%20module%20structure/plan.md) · [PP-11](../PP-11%20Configure%20Docker%20Compose%20infrastructure/plan.md) · [PP-12](../PP-12%20Design%20metadata%20database%20schema%20and%20application%20configuration/plan.md). Keep the contracts and dependency direction consistent with these linked tasks.

### 2.1 Decisions and conventions

- **Package layout:** use the Python `src/` layout. The import package name is `protein3d`; keep repository and distribution names distinct if packaging conventions require it.
- **Python version:** use CPython 3.13.x (`>=3.13,<3.14`) as the initial project baseline. The version was checked against the planned PyTorch, PySpark, and Biopython support ranges and is configured consistently in `.python-version`, `pyproject.toml`, README, and CI. Re-check exact compatibility when downstream tasks pin library versions.
- **Dependency management:** use `uv` for project environment and runtime dependency management; declare direct dependencies in `pyproject.toml` and commit `uv.lock`. PP-9 starts with no scientific/runtime dependencies. Ruff, Mypy, and pytest are invoked through `uvx` at explicit versions in the Makefile and CI; add them to a project development dependency group when a future task needs a unified in-project tool environment. Avoid parallel lockfiles or mixing dependency managers.
- **Configuration and secrets:** commit `.env.example` with variable names and safe placeholders only. Keep real credentials, local `.env`, tokens, and private connection strings out of Git. Follow PP-12 for the canonical typed application settings and configuration contract.
- **Data and artifacts:** keep only directory placeholders, small synthetic fixtures, and documented examples in Git. Do not commit downloaded protein structures, large datasets, model weights, generated embeddings, or runtime outputs; define their storage ownership in the relevant downstream plans.
- **Architecture boundary:** this task creates only the package skeleton, configuration bootstrap placeholder, and CLI bootstrap. PP-10 owns domain module boundaries and interfaces.

## 3. Step-by-step implementation and to-do

### Step 1 — Inspect repository and define conventions

Review existing files, repository history, and branch state before editing. Choose the initial directory layout and confirm the Python version against the intended dependency stack.

**To-do**
- [x] Record the current repository baseline and avoid overwriting existing work
- [x] Define directory/package naming and import conventions using the `protein3d` import package
- [x] Check supported Python versions for the planned scientific/ML dependencies and select one compatible minor version (CPython 3.13.x)
- [x] Record the selected Python version and the compatibility rationale in project documentation
- [x] Confirm that the initial layout does not duplicate or conflict with module ownership in PP-10

### Step 2 — Create the Python project skeleton

Add package metadata, dependency management, environment template, and minimal application entry points.

**To-do**
- [x] Add `pyproject.toml` with project metadata, the selected Python constraint, an initially empty runtime dependency list, and a reserved development dependency group; pin quality tools in Makefile/CI
- [x] Initialize `uv` dependency management and commit the generated `uv.lock`
- [x] Add `.env.example` with documented variable names and safe placeholders only
- [x] Add the minimal `src/protein3d/` package and CLI bootstrap; do not implement domain logic
- [x] Add the agreed top-level directories for tests, configs, scripts, docs, and data placeholders
- [x] Add `.gitignore` rules for virtual environments, caches, local environment files, generated data, model artifacts, and temporary outputs

**Initial directory convention**
```text
.
├── src/
│   └── protein3d/
│       ├── __init__.py
│       └── cli.py
├── tests/
│   ├── unit/
│   ├── integration/
│   └── conftest.py
├── configs/
├── scripts/
├── docs/
├── data/
│   └── .gitkeep
├── .env.example
├── .gitignore
├── pyproject.toml
├── uv.lock
├── Makefile
└── README.md
```

This is an initial workspace convention, not a commitment to implement all future domain modules in PP-9. Add further directories only when their owning plans define them.

### Step 3 — Configure developer quality gates

Set up consistent local commands for formatting, linting, type checking, and tests. Keep the checks deterministic and runnable in CI.

**To-do**
- [x] Configure Ruff for linting and formatting, with project-wide rules in `pyproject.toml`
- [x] Configure Mypy for static type checking; install pytest into its isolated tool environment so test markers are typed
- [x] Configure pytest discovery and separate unit from integration tests
- [ ] Add Makefile targets for environment setup, lint, format-check, type-check, unit tests, integration tests, and the aggregate check
- [x] Ensure project commands use the locked `uv` environment and quality tools use explicitly pinned `uvx` versions; commands fail on non-zero check results
- [x] Add unit tests for package import/version and CLI parser bootstrap

### Step 4 — Add CI and verify a clean setup

Prove that a new developer and GitHub Actions runner can recreate the environment from the committed project files.

**To-do**
- [x] Add a GitHub Actions workflow triggered on pushes to the project/feature branches and pull requests targeting the project branch
- [x] Pin or explicitly select Python 3.13 in CI, matching Step 1
- [x] Install `uv` and synchronize dependencies from `uv.lock` in locked mode
- [x] Run Ruff lint, Ruff format-check, Mypy, and pytest in CI
- [x] Run CLI/package-import smoke checks in CI
- [x] Write clean-checkout setup instructions, including prerequisites and exact commands
- [x] Run the documented setup and all quality gates on a fresh GitHub Actions runner
- [x] Record the selected Python version, dependency-lock workflow, and verification results

## 4. Deliverables

- A minimal `src/`-layout Python project with a `protein3d` import package and CLI bootstrap.
- `pyproject.toml` and committed `uv.lock` defining reproducible dependencies.
- Safe `.env.example`, `.gitignore`, and documented data/artifact handling rules.
- Ruff, Mypy, pytest, and Makefile quality gates.
- GitHub Actions CI covering dependency installation, static checks, tests, and smoke verification.
- README instructions for creating the environment and running the checks from a clean checkout.

## 5. Acceptance criteria

- [x] All Step 1–4 to-dos are completed and reviewed.
- [x] Python 3.13 is compatible with the initial planned dependency stack and is consistently configured in project metadata, documentation, and CI.
- [x] A clean checkout installs the exact locked project dependency set using `uv sync --locked` without manually editing dependency versions.
- [x] The `protein3d` package imports and the installed CLI bootstrap starts successfully.
- [x] Ruff lint, Ruff format-check, Mypy, pytest, and CLI smoke checks pass in GitHub Actions.
- [x] Integration-test conventions and execution commands are documented; infrastructure-dependent integration tests remain owned by downstream tasks.
- [x] No credentials, local environment files, large datasets, model weights, or generated artifacts are committed.
- [x] The initial package skeleton does not pre-empt PP-10's domain module boundaries or PP-12's configuration/schema ownership.
- [x] Setup instructions include exact commands and the supported Python version.

## 6. Testing strategy

- **Unit:** verify package import, CLI bootstrap, and basic configuration bootstrap behavior without external services.
- **Integration:** establish pytest markers/conventions and document how infrastructure-dependent tests are invoked; implement service-specific integration tests in the relevant downstream tasks.
- **Regression:** verify stable package entry points, dependency-lock reproducibility, and configuration conventions with fixed checks.
- **CI:** run locked dependency installation, Ruff lint/format checks, Mypy, pytest, and CLI smoke verification on every applicable pull request/push.
- **Manual:** follow the documented clean-checkout workflow in a fresh environment and confirm that no untracked secrets or generated data are required.

## 7. Cross-task links

- **Upstream:** None; this is the starting task.
- **Downstream / coordination:** PP-10, PP-11, PP-12
- **PP-10:** owns domain module boundaries, data contracts, and dependency direction. PP-9 supplies only the package and test scaffolding.
- **PP-11:** may consume the documented environment and command conventions when defining local services and Compose workflows.
- **PP-12:** owns typed application settings, metadata schema, and configuration validation. PP-9 provides only safe environment-file and package bootstrap conventions.
- **Rule:** consume upstream outputs as defined; do not silently redefine shared identifiers, schemas, or configuration. If a contract must change, update the owning plan and all affected consumers in the same change set.

## 8. Completion notes

**Completed implementation:** CPython 3.13.x; `uv` project management with committed `uv.lock`; `src/protein3d` package and `protein3d` CLI; `.env.example` and artifact/secret ignore rules; Ruff 0.14.5, Mypy 1.18.2, and pytest 8.4.2 invoked at pinned versions through `uvx`; Makefile commands; GitHub Actions CI.

**Verification:** local smoke checks and 3 pytest tests passed. GitHub Actions run [#607](https://github.com/kakashianh/labs/actions/runs/44310250138) passed all steps: locked environment installation, Ruff lint, Ruff format, Mypy, pytest (3 passed), and package/CLI smoke checks.

**Implementation note:** PP-9 has no scientific/runtime dependencies yet, so `uv.lock` captures the initial project package graph. Quality tools are independently pinned in Makefile/CI rather than adding scientific or tooling packages to the runtime lock. Future tasks should add domain dependencies to `pyproject.toml` and refresh `uv.lock` when their owning modules are introduced.
