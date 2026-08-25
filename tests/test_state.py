from pwviewpdf import state


def test_missing_file_reads_as_empty(tmp_path):
    assert state.load(tmp_path / "nope.json") == {}


def test_corrupt_file_reads_as_empty(tmp_path):
    path = tmp_path / "state.json"
    path.write_text("{not json")
    assert state.load(path) == {}


def test_remember_round_trips(tmp_path):
    path = tmp_path / "sub" / "state.json"
    state.remember("last_identity", "ada-lovelace", path)
    assert state.load(path)["last_identity"] == "ada-lovelace"


def test_remember_keeps_other_keys(tmp_path):
    path = tmp_path / "state.json"
    state.remember("a", 1, path)
    state.remember("b", 2, path)
    assert state.load(path) == {"a": 1, "b": 2}


def test_unwritable_location_is_survivable(tmp_path):
    unwritable = tmp_path / "ro"
    unwritable.mkdir(mode=0o500)
    state.save({"x": 1}, unwritable / "state.json")     # must not raise
