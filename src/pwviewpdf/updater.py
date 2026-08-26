"""Small, dependency-free updater backed by GitHub Releases.

The package is never installed before its published SHA-256 checksum matches.
Installation is delegated to apt through polkit, so the app never handles or
stores an administrator password.
"""

from __future__ import annotations

import hashlib
import json
import os
import platform
import subprocess
import tempfile
import time
import urllib.request
from dataclasses import dataclass
from pathlib import Path

from . import __version__
from . import state

REPOSITORY = os.environ.get("NITUM_PDF_REPOSITORY", "dev-jackson/nitum-pdf")
API_URL = f"https://api.github.com/repos/{REPOSITORY}/releases/latest"
CHECK_INTERVAL = 24 * 60 * 60


@dataclass(frozen=True)
class Release:
    version: str
    package_url: str
    checksum_url: str


def _version(value: str) -> tuple[int, ...]:
    clean = value.strip().lower().lstrip("v")
    numbers = clean.split("-", 1)[0].split(".")
    if not numbers or any(not item.isdigit() for item in numbers):
        raise ValueError(f"versión no válida: {value}")
    return tuple(int(item) for item in numbers)


def _architecture() -> str:
    machine = platform.machine().lower()
    return {"x86_64": "amd64", "amd64": "amd64", "aarch64": "arm64",
            "arm64": "arm64"}.get(machine, machine)


def should_check(now: float | None = None) -> bool:
    last = float(state.load().get("last_update_check", 0))
    return (now if now is not None else time.time()) - last >= CHECK_INTERVAL


def latest_release(opener=urllib.request.urlopen) -> Release | None:
    request = urllib.request.Request(
        API_URL,
        headers={"Accept": "application/vnd.github+json", "User-Agent": "Nitum-PDF"},
    )
    with opener(request, timeout=10) as response:
        payload = json.load(response)
    state.remember("last_update_check", time.time())
    remote = str(payload["tag_name"]).lstrip("v")
    if _version(remote) <= _version(__version__):
        return None
    suffix = f"_{_architecture()}.deb"
    assets = {asset["name"]: asset["browser_download_url"] for asset in payload["assets"]}
    package = next((name for name in assets if name.endswith(suffix)), None)
    if package is None or f"{package}.sha256" not in assets:
        raise RuntimeError(f"la versión {remote} no contiene un paquete para {_architecture()}")
    return Release(remote, assets[package], assets[f"{package}.sha256"])


def _download(url: str, target: Path, opener=urllib.request.urlopen) -> None:
    request = urllib.request.Request(url, headers={"User-Agent": "Nitum-PDF"})
    with opener(request, timeout=60) as response, target.open("wb") as output:
        while chunk := response.read(1024 * 1024):
            output.write(chunk)


def download_and_verify(release: Release, opener=urllib.request.urlopen) -> Path:
    directory = Path(tempfile.mkdtemp(prefix="nitum-pdf-update-"))
    package = directory / Path(release.package_url).name
    checksum = directory / f"{package.name}.sha256"
    _download(release.package_url, package, opener)
    _download(release.checksum_url, checksum, opener)
    expected = checksum.read_text(encoding="utf-8").strip().split()[0].lower()
    actual = hashlib.sha256(package.read_bytes()).hexdigest()
    if len(expected) != 64 or actual != expected:
        package.unlink(missing_ok=True)
        raise RuntimeError("la comprobación SHA-256 del paquete no coincide")
    return package


def install_deb(package: Path) -> subprocess.Popen:
    if package.suffix != ".deb" or not package.is_file():
        raise ValueError("el archivo de actualización no es un paquete .deb")
    return subprocess.Popen(["pkexec", "apt-get", "install", "-y", str(package)])
