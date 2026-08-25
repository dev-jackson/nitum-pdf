import pytest

from pwviewpdf import signing, trust
from tests.test_signing import OFFLINE, sign


def test_fingerprint_is_grouped_hex(identity):
    text = trust.fingerprint(identity["cert_der"])
    assert len(text.split()) == 32
    assert all(len(part) == 2 for part in text.split())


def test_nothing_is_trusted_by_default(tmp_path, identity):
    assert trust.roots(tmp_path) == []
    assert trust.is_trusted(identity["cert_der"], tmp_path) is False


def test_added_certificate_becomes_a_root(tmp_path, identity):
    trust.add(identity["cert_der"], "Ada Lovelace", tmp_path)
    assert trust.is_trusted(identity["cert_der"], tmp_path) is True
    assert len(trust.roots(tmp_path)) == 1


def test_adding_twice_does_not_duplicate(tmp_path, identity):
    trust.add(identity["cert_der"], "Ada Lovelace", tmp_path)
    trust.add(identity["cert_der"], "Ada Lovelace", tmp_path)
    assert len(trust.roots(tmp_path)) == 1


def test_removing_undoes_the_decision(tmp_path, identity):
    trust.add(identity["cert_der"], "Ada", tmp_path)
    trust.remove(identity["cert_der"], tmp_path)
    assert trust.is_trusted(identity["cert_der"], tmp_path) is False


def test_a_broken_file_does_not_break_the_list(tmp_path, identity):
    trust.add(identity["cert_der"], "Ada", tmp_path)
    (tmp_path / "garbage.crt").write_bytes(b"not a certificate")
    assert len(trust.roots(tmp_path)) == 1


def test_signature_carries_the_certificate_so_it_can_be_trusted(sample_pdf, identity):
    status = sign(sample_pdf, identity).statuses[0]
    assert status.certificate == identity["cert_der"]
    assert status.trusted is False


def test_trusting_the_issuer_turns_the_signature_green(sample_pdf, identity, tmp_path, monkeypatch):
    signed = sign(sample_pdf, identity).output
    store = tmp_path / "trusted"
    monkeypatch.setattr(trust, "TRUST_DIR", store)
    trust.add(identity["cert_der"], "Ada Lovelace", store)
    assert signing.verify(signed)[0].trusted is True
