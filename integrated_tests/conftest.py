from pathlib import Path

FIXTURES = Path(__file__).resolve().parent / "fixtures"

PY_FIXTURE = FIXTURES / "sample.py"
TS_FIXTURE = FIXTURES / "sample.ts"
PY_CLEAN_FIXTURE = FIXTURES / "clean.py"


def pytest_configure(config):
    config.addinivalue_line("markers", "llm: tests that require LLM API key")
    config.addinivalue_line("markers", "integration: integration tests")
