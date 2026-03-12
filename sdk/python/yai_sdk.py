#!/usr/bin/env python3
"""Minimal ctypes wrapper for yai-sdk."""

from __future__ import annotations

import ctypes
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
LIB = ROOT / "dist" / "lib" / "libyai_sdk.so"


def main() -> int:
    if not LIB.exists():
        print(f"library not found: {LIB}")
        print("run: make -j4")
        return 1

    sdk = ctypes.CDLL(str(LIB))

    sdk.yai_sdk_abi_version.restype = ctypes.c_int
    sdk.yai_sdk_version.restype = ctypes.c_char_p
    sdk.yai_sdk_errstr.argtypes = [ctypes.c_int]
    sdk.yai_sdk_errstr.restype = ctypes.c_char_p

    abi = sdk.yai_sdk_abi_version()
    version = sdk.yai_sdk_version().decode("utf-8")
    server_off = sdk.yai_sdk_errstr(107).decode("utf-8")

    print(f"abi={abi}")
    print(f"version={version}")
    print(f"err(107)={server_off}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
