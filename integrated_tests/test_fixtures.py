import pytest
from conftest import PY_FIXTURE, TS_FIXTURE, PY_CLEAN_FIXTURE


@pytest.mark.integration
def test_python_fixture_is_readable():
    content = PY_FIXTURE.read_text()
    assert len(content) > 0
    assert "class OrderProcessor" in content


@pytest.mark.integration
def test_ts_fixture_is_readable():
    content = TS_FIXTURE.read_text()
    assert len(content) > 0


@pytest.mark.integration
def test_clean_fixture_is_readable():
    content = PY_CLEAN_FIXTURE.read_text()
    assert len(content) > 0
