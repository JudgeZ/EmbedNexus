"""Red phase integration tests for the Python client CLI."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import pytest

HERE = Path(__file__).resolve().parent
FIXTURE_ROOT = HERE.parent / "fixtures" / "python"
TRANSPORTS = ("stdio", "http", "tls")


@pytest.mark.parametrize("transport", TRANSPORTS)
def test_python_client_transcripts(update_transcripts: bool, transport: str) -> None:
    request_path = FIXTURE_ROOT / transport / "request.json"
    response_path = FIXTURE_ROOT / transport / "response.json"

    request_payload = json.loads(request_path.read_text(encoding="utf-8"))
    response_payload = json.loads(response_path.read_text(encoding="utf-8"))

    assert request_payload["params"]["transport"] == transport
    assert response_payload["id"] == request_payload["id"]

    cli_entry = Path("clients/python/client.py")
    command = [sys.executable, str(cli_entry), "--transport", transport]

    if update_transcripts:
        pytest.skip("transcript regeneration not yet implemented for Python client tests")

    try:
        subprocess.run(
            command,
            check=True,
            capture_output=True,
            text=True,
        )
    except FileNotFoundError as exc:
        pytest.fail(
            "not yet implemented: Python client CLI entry point missing"
            f" (attempted to run {' '.join(command)}): {exc}"
        )
    except subprocess.CalledProcessError as exc:  # pragma: no cover - pending CLI implementation
        pytest.fail(
            "not yet implemented: Python client CLI execution placeholder"
            f" (exit code {exc.returncode})\nSTDOUT:\n{exc.stdout}\nSTDERR:\n{exc.stderr}"
        )

    pytest.fail("not yet implemented: Python client subprocess execution should assert transcripts")
