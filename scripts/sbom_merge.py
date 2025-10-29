#!/usr/bin/env python3
"""
Merge all **/*.cdx.json CycloneDX files into a single SBOM.

Deduplication key order: bom-ref || purl || (name, version).
Prints merged JSON to stdout by default, or writes to --out <file>.
Emits a minimal CycloneDX document with metadata.component
{type: application, name: "zaevrynth-workspace"}.

No external dependencies.
"""
from __future__ import annotations

import argparse
import glob
import json
import os
from typing import Dict, Tuple


def component_key(c: Dict) -> Tuple:
    return (
        c.get("bom-ref")
        or c.get("purl")
        or (c.get("name"), c.get("version"))
    )


def main() -> int:
    ap = argparse.ArgumentParser(description="Merge CycloneDX .cdx.json files")
    ap.add_argument(
        "--out", dest="out", default=None, help="Write merged SBOM to file (default: stdout)"
    )
    ap.add_argument(
        "--root", dest="root", default=".", help="Root directory to search (default: .)"
    )
    args = ap.parse_args()

    pattern = os.path.join(args.root, "**", "*.cdx.json")
    paths = sorted(glob.glob(pattern, recursive=True))

    seen: Dict[Tuple, Dict] = {}
    for p in paths:
        try:
            with open(p, "r", encoding="utf-8") as fh:
                data = json.load(fh)
        except Exception:
            continue
        for comp in data.get("components", []) or []:
            k = component_key(comp)
            if k and k not in seen:
                seen[k] = comp

    sbom = {
        "bomFormat": "CycloneDX",
        "specVersion": "1.3",
        "serialNumber": "urn:uuid:00000000-0000-0000-0000-000000000000",
        "version": 1,
        "metadata": {
            "component": {
                "type": "application",
                "name": "zaevrynth-workspace",
            }
        },
        "components": list(seen.values()),
    }

    output = json.dumps(sbom, indent=2, sort_keys=False)
    if args.out:
        os.makedirs(os.path.dirname(args.out), exist_ok=True)
        with open(args.out, "w", encoding="utf-8") as fh:
            fh.write(output)
    else:
        print(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

