from __future__ import annotations

import os
from pathlib import Path

PACKAGE_NAME = "orbit-code-cli-bin"


def bundled_orbit_code_path() -> Path:
    exe = "orbit-code.exe" if os.name == "nt" else "orbit-code"
    path = Path(__file__).resolve().parent / "bin" / exe
    if not path.is_file():
        raise FileNotFoundError(
            f"{PACKAGE_NAME} is installed but missing its packaged Orbit Code binary at {path}"
        )
    return path


__all__ = ["PACKAGE_NAME", "bundled_orbit_code_path"]
