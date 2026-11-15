"""Dump a FORMAT_VERSION 8 English pack (scowl-en-us) as one word per line.

The core reads only the current pack format, so the experiment cannot load the
old pack directly. Layout (crates/qim-core/src/pack.rs at 594aa9b): 4-byte magic,
postcard varint u32 version, then EnglishPack { id: String, words: Vec<String> }
in postcard (varint length prefixes). Run: python3 -I dump_pack_v8.py <pack> > words.txt
"""

import sys


def varint(data, i):
    value = shift = 0
    while True:
        b = data[i]
        i += 1
        value |= (b & 0x7F) << shift
        if b < 0x80:
            return value, i
        shift += 7


def string(data, i):
    n, i = varint(data, i)
    return data[i : i + n].decode("utf-8"), i + n


def main():
    data = open(sys.argv[1], "rb").read()
    version, i = varint(data, 4)
    if version != 8:
        sys.exit(f"pack format {version}, expected 8")
    _, i = string(data, i)
    count, i = varint(data, i)
    for _ in range(count):
        word, i = string(data, i)
        print(word)
    if i != len(data):
        sys.exit(f"{len(data) - i} trailing bytes")


if __name__ == "__main__":
    main()
