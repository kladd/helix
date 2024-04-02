# Helix with things I like from Vim

## Visual-line mode

Shift-V enters line-wise selection mode.

## Action-selection commands

Delete with `daw`, `diw`, `dt<char>`, etc. Yank with `yaw`, `yiw`, `yt<char>`,
etc.

## Misc.

* Find to/through character does not select unless in visual mode.
* Exiting visual modes clears selection.

## Installation/Use

From `helix` repo:
```sh
export HELIX_RUNTIME=$PWD/runtime
cargo install --profile opt --locked --path helix-term
```

Check local language support:
```sh
hx --health
```

Edit something:
```
hx
```

---

[Helix README](https://github.com/helix-editor/helix/blob/master/README.md)
