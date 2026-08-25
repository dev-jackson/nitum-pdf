"""Tiny persisted state: remembering the last identity is worth a lot of clicks."""

from __future__ import annotations

import json
from pathlib import Path

from .identities import APP_DIR

STATE_FILE = APP_DIR / "state.json"


def load(path: Path | None = None) -> dict:
    path = path or STATE_FILE
    try:
        return json.loads(path.read_text())
    except (OSError, ValueError):
        return {}


def save(values: dict, path: Path | None = None) -> None:
    path = path or STATE_FILE
    try:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(values, indent=2))
    except OSError:
        pass          # never let a read-only home stop the user from signing


def remember(key: str, value, path: Path | None = None) -> None:
    values = load(path)
    values[key] = value
    save(values, path)
