import hashlib
import io
import json

import pytest

from pwviewpdf import updater


class Response(io.BytesIO):
    def __enter__(self):
        return self

    def __exit__(self, *_args):
        self.close()


def opener_for(payload):
    def open_url(request, timeout=0):
        value = payload[request.full_url]
        return Response(value if isinstance(value, bytes) else json.dumps(value).encode())
    return open_url


def test_versions_are_numeric_not_lexicographic():
    assert updater._version("v1.10.0") > updater._version("1.9.9")
    with pytest.raises(ValueError):
        updater._version("latest")


def test_latest_release_selects_arch_package(monkeypatch, tmp_path):
    monkeypatch.setattr(updater, "__version__", "0.2.0")
    monkeypatch.setattr(updater, "_architecture", lambda: "amd64")
    monkeypatch.setattr(updater.state, "STATE_FILE", tmp_path / "state.json")
    payload = {"tag_name": "v0.3.0", "assets": [
        {"name": "nitum-pdf_0.3.0_amd64.deb", "browser_download_url": "https://x/pkg"},
        {"name": "nitum-pdf_0.3.0_amd64.deb.sha256", "browser_download_url": "https://x/sum"},
    ]}
    result = updater.latest_release(opener_for({updater.API_URL: payload}))
    assert result.version == "0.3.0"
    assert result.package_url == "https://x/pkg"


def test_current_release_returns_none(monkeypatch, tmp_path):
    monkeypatch.setattr(updater, "__version__", "0.2.0")
    monkeypatch.setattr(updater.state, "STATE_FILE", tmp_path / "state.json")
    payload = {"tag_name": "v0.2.0", "assets": []}
    assert updater.latest_release(opener_for({updater.API_URL: payload})) is None


def test_download_verifies_checksum(tmp_path, monkeypatch):
    monkeypatch.setattr(updater.tempfile, "mkdtemp", lambda prefix: str(tmp_path))
    body = b"a deb package"
    digest = hashlib.sha256(body).hexdigest().encode()
    release = updater.Release("0.3.0", "https://x/nitum.deb", "https://x/nitum.deb.sha256")
    package = updater.download_and_verify(release, opener_for({
        release.package_url: body, release.checksum_url: digest,
    }))
    assert package.read_bytes() == body


def test_download_rejects_bad_checksum(tmp_path, monkeypatch):
    monkeypatch.setattr(updater.tempfile, "mkdtemp", lambda prefix: str(tmp_path))
    release = updater.Release("0.3.0", "https://x/nitum.deb", "https://x/nitum.deb.sha256")
    with pytest.raises(RuntimeError, match="SHA-256"):
        updater.download_and_verify(release, opener_for({
            release.package_url: b"bad", release.checksum_url: b"0" * 64,
        }))


def test_installer_returns_process_so_ui_can_wait_and_restart(tmp_path, monkeypatch):
    package = tmp_path / "nitum.deb"
    package.write_bytes(b"deb")
    process = object()
    calls = []
    monkeypatch.setattr(updater.subprocess, "Popen",
                        lambda command: calls.append(command) or process)
    assert updater.install_deb(package) is process
    assert calls[0][:4] == ["pkexec", "apt-get", "install", "-y"]
