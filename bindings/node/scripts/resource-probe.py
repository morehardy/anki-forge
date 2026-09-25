#!/usr/bin/env python3
"""Run the public Project/Media resource workload against the rebuilt host addon."""
import os
import pathlib
import platform
import subprocess
root = pathlib.Path(__file__).resolve().parents[1]
suffix = {('Darwin','arm64'):'darwin-arm64',('Darwin','x86_64'):'darwin-x64',('Linux','x86_64'):'linux-x64-gnu',('Windows','AMD64'):'win32-x64-msvc'}[(platform.system(), platform.machine())]
env = dict(os.environ, ANKI_FORGE_NATIVE_PATH=str(root/'npm'/suffix/'anki-forge.node'))
subprocess.run(['node','--expose-gc',str(root/'scripts/profile-resources.mjs')],env=env,check=True)
