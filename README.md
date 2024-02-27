# Helix with things I like from Vim
* Visual line mode
* `action -> selection` commands
* Misc. Vim-alike behavior

## Visual line mode

`Shift-v` to enter a limited visual line mode.

> Just like kakoune we have no plan to add a linewise selection mode.
>
> [helix-editor/helix#356](https://github.com/helix-editor/helix/issues/356#issuecomment-1785792949), [helix-editor/helix#2317](https://github.com/helix-editor/helix/issues/2317), [helix-editor/helix#5548](https://github.com/helix-editor/helix/discussions/5548#discussioncomment-4694127)

## `action -> selection` commands

* Delete `daw`, `diw`, `dt<char>`, etc.
* Yank `yaw`, `yiw`, `yt<char>`, etc.

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
