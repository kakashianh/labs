.PHONY: setup lint format-check type-check test test-unit test-integration smoke check clean infra-config infra-up infra-up-graph infra-up-vector infra-up-all infra-down infra-status infra-logs infra-reset

RUFF_VERSION ?= 0.14.5
MYPY_VERSION ?= 1.18.2
PYTEST_VERSION ?= 8.4.2

setup:
	uv sync --locked

lint:
	uvx --from ruff==$(RUFF_VERSION) ruff check .

format-check:
	uvx --from ruff==$(RUFF_VERSION) ruff format --check .

type-check:
	uvx --from mypy==$(MYPY_VERSION) --with pytest==$(PYTEST_VERSION) mypy src tests

test: test-unit test-integration

test-unit:
	uvx --from pytest==$(PYTEST_VERSION) pytest -m unit

test-integration:
	uvx --from pytest==$(PYTEST_VERSION) pytest -m integration

smoke:
	uv run --locked protein3d --help
	uv run --locked python -c "import protein3d; print('protein3d import: OK')"

check: lint format-check type-check test smoke

# Docker Compose infrastructure (PP-11)
infra-config:
	docker compose --env-file .env.example --profile graph --profile vector config --quiet

infra-up:
	docker compose up -d

infra-up-graph:
	docker compose --profile graph up -d

infra-up-vector:
	docker compose --profile vector up -d

infra-up-all:
	docker compose --profile graph --profile vector up -d

infra-down:
	docker compose --profile graph --profile vector down

infra-status:
	docker compose --profile graph --profile vector ps

infra-logs:
	docker compose --profile graph --profile vector logs -f

# Destructive: permanently removes every named volume for this Compose project.
infra-reset:
	docker compose --profile graph --profile vector down --volumes --remove-orphans

clean:
	rm -rf .venv .pytest_cache .mypy_cache .ruff_cache build dist *.egg-info
