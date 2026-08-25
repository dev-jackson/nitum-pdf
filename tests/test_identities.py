import subprocess
from types import SimpleNamespace

from pwviewpdf import identities

P11KIT_OUTPUT = """
module: opensc-pkcs11
    path: /usr/lib64/pkcs11/opensc-pkcs11.so
    uri: pkcs11:library-description=OpenSC
    library-description: OpenSC smartcard framework
    token: DNIe 001234567
        uri: pkcs11:model=PKCS%2315
        manufacturer: FNMT
module: p11-kit-trust
    path: /usr/lib64/pkcs11/p11-kit-trust.so
    token: System Trust
"""


def fake_runner(output=P11KIT_OUTPUT, fail=False):
    def run(*_args, **_kwargs):
        if fail:
            raise FileNotFoundError("p11-kit")
        return SimpleNamespace(stdout=output)
    return run


def test_tokens_are_read_from_p11kit(tmp_path):
    found = identities.discover(tmp_path, runner=fake_runner())
    labels = [i.label for i in found]
    assert "DNIe 001234567" in labels
    assert all(i.kind == "pkcs11" for i in found)


def test_token_keeps_its_module_path(tmp_path):
    dnie = next(i for i in identities.discover(tmp_path, runner=fake_runner())
                if i.label.startswith("DNIe"))
    assert dnie.path.endswith("opensc-pkcs11.so")
    assert dnie.token_label == "DNIe 001234567"


def test_missing_p11kit_is_not_fatal(tmp_path, monkeypatch):
    monkeypatch.setattr(identities, "WELL_KNOWN_MODULES", ())
    assert identities.discover(tmp_path, runner=fake_runner(fail=True)) == []


def test_pkcs12_files_are_listed(tmp_path, monkeypatch):
    monkeypatch.setattr(identities, "WELL_KNOWN_MODULES", ())
    (tmp_path / "ada.p12").write_bytes(b"x")
    (tmp_path / "bruno.pfx").write_bytes(b"x")
    (tmp_path / "notes.txt").write_bytes(b"x")
    found = identities.discover(tmp_path, runner=fake_runner(fail=True))
    assert [i.label for i in found] == ["ada", "bruno"]
    assert all(i.kind == "pkcs12" for i in found)


def test_tokens_are_offered_before_files(tmp_path):
    (tmp_path / "ada.p12").write_bytes(b"x")
    kinds = [i.kind for i in identities.discover(tmp_path, runner=fake_runner())]
    assert kinds.index("pkcs11") < kinds.index("pkcs12")


def test_secret_prompt_matches_the_credential(tmp_path):
    (tmp_path / "ada.p12").write_bytes(b"x")
    found = identities.discover(tmp_path, runner=fake_runner())
    assert "PIN" in found[0].secret_prompt
    assert "Contraseña" in found[-1].secret_prompt


def test_import_copies_and_locks_down_the_file(tmp_path):
    source = tmp_path / "from-windows.pfx"
    source.write_bytes(b"pretend-pkcs12")
    target_dir = tmp_path / "store"
    stored = identities.import_pkcs12(source, target_dir)
    assert stored.read_bytes() == b"pretend-pkcs12"
    assert oct(stored.stat().st_mode)[-3:] == "600"
