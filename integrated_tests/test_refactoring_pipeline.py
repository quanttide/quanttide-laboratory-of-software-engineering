"""已弃用 — 由 test_python_pipeline.py + test_fixtures.py 替代。

保留此文件仅用于 fixture 自检，确保旧测试继续运行直到完全迁移。
"""

import pytest
from conftest import PY_FIXTURE, TS_FIXTURE


@pytest.mark.integration
def test_python_fixture_has_content():
    assert len(PY_FIXTURE.read_text()) > 0


@pytest.mark.integration
def test_typescript_fixture_has_content():
    assert len(TS_FIXTURE.read_text()) > 0
