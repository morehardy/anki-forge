"""Validity rules shared by publication timing and offline evidence checks."""


def validate(measurement):
    assert measurement["reaped"], "exporter was not reaped"
    assert not measurement["spawn_error"], "exporter did not start"
    assert not measurement["exit_code"], "exporter failed"
    assert not measurement["signal"], "exporter was terminated"
    assert not measurement["interrupted_signal"], "collector was interrupted"
    assert not measurement["leftover_descendants"], "exporter left descendant work"
