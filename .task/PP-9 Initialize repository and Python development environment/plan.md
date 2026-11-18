# PP-9 — Initialize repository and Python development environment

## 1. Objective

Establish a reproducible, maintainable Python workspace that every later data pipeline, protein-structure processing, graph, ML, and API task can build on.

## 2. Scope and design context

Create the repository layout, Python compatibility policy, dependency lock strategy, environment template, developer quality gates, CI baseline, and test conventions. Do not implement domain pipelines or prematurely create domain-specific modules owned by PP-10 and later tasks.

**Upstream dependencies:** None; this is the starting task.

**Related plans:** [PP-10](../PP-10%20Design%20project%20architecture%20and%20module%20structure/plan.md) · [PP-11](../PP-11%20Configure%20Docker%20Compose%20infrastructure/plan.md) · [PP-12](../PP-12%20Design%20metadata%20database%20schema%20and%20application%20configuration/plan.md). Keep the contracts and dependency direction consistent with these linked tasks.

### 2.1 Decisions and conventions

- **Package layout:** use the Python `src/` layout. The import package name is `protein3d`; keep repository and distribution names distinct if packaging conventions require it.
- **Python version:** select one project-wide minor version only after checking compatibility across the planned scientific/ML stack, especially PyTorch, PySpark, Biopython, and the selected structure-processing libraries. Record the chosen version and rationale in the README and `pyproject.toml`; do not assume the newest Python release is compatible. CI and local setup must use the same supported version initially.
- **Dependency management:** use `uv` for environment and dependency management, declare direct dependencies in `pyproject.toml`, and commit `uv.lock`. Avoid maintaining parallel lockfiles or mixing multiple dependency managers.
- **Configuration and secrets:** commit `.env.example` with variable names and safe placeholders only. Keep real credentials, local `.env`, tokens, and private connection strings out of Git. Follow PP-12 for the canonical typed application settings and configuration contract.
- **Data and artifacts:** keep only directory placeholders, small synthetic fixtures, and documented examples in Git. Do not commit downloaded protein structures, large datasets, model weights, generated embeddings, or runtime outputs; define their storage ownership in the relevant downstream plans.
- **Architecture boundary:** this task creates only the package skeleton, configuration bootstrap placeholder, and CLI bootstrap. PP-10 owns domain module boundaries and interfaces.

## 3. Step-by-step implementation and to-do

### Step 1 — Inspect repository and define conventions

Review existing files, repository history, and branch state before editing. Choose the initial directory layout and confirm the Python version against the intended dependency stack.

**To-do**
- [ ] Record the current repository baseline and avoid overwriting existing work
- [ ] Define directory/package naming and import conventions using the `protein3d` import package
- [ ] Check supported Python versions for the planned scientific/ML dependencies and select one compatible minor version
- [ ] Record the selected Python version and the compatibility rationale in project documentation
- [ ] Confirm that the initial layout does not duplicate or conflict with module ownership in PP-10

### Step 2 — Create the Python project skeleton

Add package metadata, dependency management, environment template, and minimal application entry points.

**To-do**
- [ ] Add `pyproject.toml` with project metadata, the selected Python constraint, runtime dependencies, and development dependency groups
- [ ] Initialize `uv` dependency management and commit the generated `uv.lock`
- [ ] Add `.env.example` with documented variable names and safe placeholders only
- [ ] Add the minimal `src/protein3d/` package and CLI bootstrap; do not implement domain logic
- [ ] Add the agreed top-level directories for tests, configs, scripts, docs, and data placeholders
- [ ] Add `.gitignore` rules for virtual environments, caches, local environment files, generated data, model artifacts, and temporary outputs

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
- [ ] Configure Ruff for linting and formatting, with project-wide rules in `pyproject.toml`
- [ ] Configure Mypy for static type checking; document any narrowly scoped exclusions rather than disabling checks globally
- [ ] Configure pytest discovery and separate unit from integration tests
- [ ] Add Makefile targets for environment setup, lint, format-check, type-check, unit tests, integration tests, and the aggregate check
- [ ] Ensure commands use the locked environment and fail with a non-zero exit code when a check fails
- [ ] Add a minimal unit test for package import and CLI bootstrap behavior

### Step 4 — Add CI and verify a clean setup

Prove that a new developer and GitHub Actions runner can recreate the environment from the committed project files.

**To-do**
- [ ] Add a GitHub Actions workflow triggered on pull requests and pushes to the project branch/default branch as appropriate
- [ ] Pin or explicitly select the same Python minor version chosen in Step 1
- [ ] Install `uv` and synchronize dependencies from `uv.lock` in frozen/locked mode
- [ ] Run Ruff lint, Ruff format-check, Mypy, and pytest in CI
- [ ] Run a CLI/package-import smoke check in CI
- [ ] Write clean-checkout setup instructions, including prerequisites and exact commands
- [ ] Run the documented setup and all quality gates in a clean environment
- [ ] Record the selected Python version, dependency-lock workflow, and verification results

## 4. Deliverables

- A minimal `src/`-layout Python project with a `protein3d` import package and CLI bootstrap.
- `pyproject.toml` and committed `uv.lock` defining reproducible dependencies.
- Safe `.env.example`, `.gitignore`, and documented data/artifact handling rules.
- Ruff, Mypy, pytest, and Makefile quality gates.
- GitHub Actions CI covering dependency installation, static checks, tests, and smoke verification.
- README instructions for creating the environment and running the checks from a clean checkout.

## 5. Acceptance criteria

- [ ] All Step 1–4 to-dos are completed and reviewed.
- [ ] The selected Python minor version is compatible with the initial planned dependency stack and is consistently configured in project metadata, documentation, and CI.
- [ ] A clean checkout can install the exact locked dependency set using `uv` without manually editing dependency versions.
- [ ] The `protein3d` package imports and the CLI bootstrap starts successfully.
- [ ] Ruff lint, Ruff format-check, Mypy, unit tests, and the smoke check pass locally and in GitHub Actions.
- [ ] Integration-test conventions and execution commands are documented; infrastructure-dependent integration tests may be implemented by their owning downstream tasks.
- [ ] No credentials, local environment files, large datasets, model weights, or generated artifacts are committed.
- [ ] The initial package skeleton does not pre-empt PP-10's domain module boundaries or PP-12's configuration/schema ownership.
- [ ] Setup instructions include exact commands and the supported Python version.

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

Record the selected Python version and compatibility rationale, dependency-management decisions, commands executed, CI run/result, artifacts produced, and any deviations from this plan. Do not mark verification items complete until the corresponding checks have actually passed.
