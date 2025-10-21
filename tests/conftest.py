import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parent.parent
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))


def pytest_addoption(parser: pytest.Parser) -> None:
    parser.addoption(
        "--update-transcripts",
        action="store_true",
        help="Regenerate golden client transcripts while running integration tests.",
    )


@pytest.fixture
def update_transcripts(request: pytest.FixtureRequest) -> bool:
    return bool(request.config.getoption("--update-transcripts"))
