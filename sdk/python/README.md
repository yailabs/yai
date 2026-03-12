# Python Wrapper (ctypes)

This directory contains a minimal `ctypes` wrapper proving that `sdk` can be consumed outside C.

## Usage

```bash
cd sdk
make -j4
python3 wrappers/python/yai_sdk.py
```

By default the script loads `dist/lib/libyai_sdk.so` and calls ABI/version/error helpers.
