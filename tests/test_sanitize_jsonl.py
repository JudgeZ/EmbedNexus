import io
import json

import importlib.util
import sys
from pathlib import Path

import pytest


def _load_module():
    module_path = Path(__file__).resolve().parent.parent / "scripts" / "sanitize_jsonl.py"
    spec = importlib.util.spec_from_file_location("sanitize_jsonl", module_path)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


sanitize_jsonl = _load_module()


def test_sanitize_stream_happy_path():
    payload_lines = [
        json.dumps(
            {
                "scenario": "fuzz",
                "tenant": "alpha-corp",
                "operation": "ingest",
                "request_count": 256,
                "budget_bytes": 4096,
                "overflow_expected": False,
            }
        ),
        json.dumps(
            {
                "scenario": "quota-throughput",
                "tenant": "beta-labs",
                "window_seconds": 300,
                "requests": 3450,
                "average_latency_ms": 135,
                "saturation_ratio": 0.85,
            }
        ),
    ]
    output = io.StringIO()
    sanitize_jsonl.sanitize_stream(io.StringIO("\n".join(payload_lines)), output)

    lines = output.getvalue().splitlines()
    assert len(lines) == 2

    fuzz = json.loads(lines[0])
    assert fuzz == {
        "scenario": "fuzz",
        "tenant_alias": "tenant-001",
        "operation": "ingest",
        "request_count": 256,
        "budget_bytes": 4096,
        "overflow_expected": False,
    }

    throughput = json.loads(lines[1])
    assert throughput == {
        "scenario": "quota-throughput",
        "tenant_alias": "tenant-002",
        "window_seconds": 300,
        "requests": 3450,
        "average_latency_ms": 135,
        "saturation_ratio": 0.85,
    }


def test_rejects_invalid_records():
    bad_payload = json.dumps({"scenario": "fuzz", "tenant": ""})
    with pytest.raises(sanitize_jsonl.SchemaError):
        sanitize_jsonl.sanitize_stream(io.StringIO(bad_payload), io.StringIO())


def test_invalid_operation_fails():
    payload = json.dumps(
        {
            "scenario": "fuzz",
            "tenant": "alpha",
            "operation": "invalid",
            "request_count": 1,
            "budget_bytes": 1,
            "overflow_expected": False,
        }
    )
    with pytest.raises(sanitize_jsonl.SchemaError):
        sanitize_jsonl.sanitize_stream(io.StringIO(payload), io.StringIO())
