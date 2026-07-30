#!/usr/bin/env python3
"""
Simple privacy terms smoke check.
Scans repository files (text) for forbidden legacy phrases and fails if any are found.
Allows ERC-20 mentions when accompanied by known payment rails (USDT/USDC/BTC/ETH) in the same line.
"""
import os
import sys
import re

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..'))
FORBIDDEN = [
    r"ERC-20 Placeholder",
    r"ERC-20 Migration",
    r"Bridge Period",
    r"Token Migration",
    r"deploy on Ethereum",
    r"native after ERC-20",
]

PAYMENT_ALLOW = [
    r"USDT",
    r"USDC",
    r"BTC",
    r"ETH",
    r"payment",
    r"accepted",
    r"Payment",
]

EXCLUDE_DIRS = {'.git', 'target', 'node_modules', '.venv', '.github', 'docs/build'}

matches = []

for dirpath, dirnames, filenames in os.walk(ROOT):
    # prune excludes
    dirnames[:] = [d for d in dirnames if d not in EXCLUDE_DIRS]
    for fname in filenames:
        path = os.path.join(dirpath, fname)
        # skip binary files
        try:
            with open(path, 'rb') as fh:
                start = fh.read(8000)
                if b"\0" in start:
                    continue
        except Exception:
            continue
        # read as text
        try:
            with open(path, 'r', encoding='utf-8') as fh:
                for i, line in enumerate(fh, start=1):
                    for pat in FORBIDDEN:
                        if re.search(pat, line, re.IGNORECASE):
                            # allow some ERC-20 mentions that clearly are about payments
                            if 'ERC-20' in pat or 'ERC-20' in line or 'deploy on Ethereum' in pat:
                                allowed = False
                                for allow_pat in PAYMENT_ALLOW:
                                    if re.search(allow_pat, line, re.IGNORECASE):
                                        allowed = True
                                        break
                                if allowed:
                                    continue
                            matches.append((path, i, line.rstrip('\n')))
        except Exception:
            # skip unreadable
            continue

if matches:
    print('Forbidden privacy-related phrases found:')
    for p, ln, text in matches:
        print(f'{p}:{ln}: {text}')
    print('\nPlease remove or replace these phrases with Genesis Allocation terminology or document justification under ADR-0017.')
    sys.exit(2)

print('Privacy terms check passed.')
sys.exit(0)
