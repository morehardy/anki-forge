"""Same package versions must not let an older embedded contract slip through."""
import json
import pytest
from ankiforge import _loader, _native


def test_same_version_old_native_contract_is_rejected(monkeypatch):
    metadata = json.loads(_native.binding_metadata())
    assert metadata['contract_version'] == '2.0.0'
    metadata['contract_version'] = '1.0.0'
    monkeypatch.setattr(_native, 'binding_metadata', lambda: json.dumps(metadata))
    with pytest.raises(ImportError, match='BINDING.VERSION_MISMATCH.*expected contract 2.0.0'):
        _loader._load()
