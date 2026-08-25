"""Timestamping and LTV against a real TSA. Skipped unless PWVIEW_NETWORK_TESTS=1."""

import os

import pytest

from pwviewpdf import signing

pytestmark = pytest.mark.skipif(
    os.environ.get("PWVIEW_NETWORK_TESTS") != "1",
    reason="needs internet; set PWVIEW_NETWORK_TESTS=1",
)


def test_timestamped_signature_reaches_pades_b_t(sample_pdf, identity):
    options = signing.SignOptions(want_timestamp=True, want_ltv=False)
    result = signing.sign_with_pkcs12(
        sample_pdf, signing.suggest_output(sample_pdf),
        identity["p12"], identity["password"], options,
    )
    assert result.level == signing.LEVEL_T
    assert result.downgrade_reason is None
    assert result.statuses[0].intact is True
