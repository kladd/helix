use std::collections::HashMap;

use super::common::space_keymap;
use super::macros::keymap;
use super::{KeyTrie, Mode};
use helix_core::hashmap;

pub fn vim_default() -> HashMap<Mode, KeyTrie> {
    let mut normal = keymap!({ "Normal mode"
        "h" | "left" => move_char_left,
        "j" | "down" => move_visual_line_down,
        "k" | "up" => move_visual_line_up,
        "l" | "right" => move_char_right,

        "w" => move_next_word_start,
        "W" => move_next_long_word_start,
        "b" => move_prev_word_start,
        "B" => move_prev_long_word_start,
        "e" => move_next_word_end,
        "E" => move_next_long_word_end,

        "0" => goto_line_start,
        "^" => goto_first_nonwhitespace,
        "$" => goto_line_end,

        "g" => { "Goto"
            "g" => goto_file_start,
            "e" => move_prev_word_end,
            "E" => move_prev_long_word_end,
            "h" => goto_line_start,
            "l" => goto_line_end,
            "s" => goto_first_nonwhitespace,
            "d" => goto_definition,
            "D" => goto_declaration,
            "y" => goto_type_definition,
            "r" => goto_reference,
            "i" => goto_implementation,
            "t" => goto_window_top,
            "c" => goto_window_center,
            "b" => goto_window_bottom,
            "x" => goto_file,
        },
        "G" => goto_last_line,

        "H" => goto_window_top,
        "M" => goto_window_center,
        "L" => goto_window_bottom,

        "i" => insert_mode,
        "I" => insert_at_line_start,
        "a" => append_mode,
        "A" => insert_at_line_end,
        "o" => open_below,
        "O" => open_above,

        "v" => vim_visual_mode,
        "V" => vim_visual_line_mode,
        "C-v" => vim_visual_block_mode,
        "R" => vim_replace_mode,

        ":" => command_mode,

        "u" => undo,
        "U" => redo,

        "/" => search,
        "?" => rsearch,
        "n" => search_next,
        "N" => search_prev,

        "f" => find_next_char,
        "F" => find_prev_char,
        "t" => find_till_char,
        "T" => till_prev_char,

        "%" => match_brackets,

        // Operators (enter operator-pending state)
        "d" => vim_op_delete,
        "c" => vim_op_change,
        "y" => vim_op_yank,
        ">" => vim_op_indent,
        "<" => vim_op_outdent,
        "=" => vim_op_autoindent,

        // Shortcuts
        "x" => vim_delete_char_forward,
        "X" => vim_delete_char_backward,
        "D" => vim_delete_to_line_end,
        "C" => vim_change_to_line_end,
        "Y" => vim_yank_line,
        "s" => vim_substitute_char,
        "S" => vim_substitute_line,
        "p" => paste_after,
        "P" => paste_before,
        "J" => join_selections,

        "~" => switch_case,

        "C-u" => half_page_up,
        "C-d" => half_page_down,
        "C-b" => page_up,
        "C-f" => page_down,

        "C-o" => jump_backward,

        "C-w" => { "Window"
            "C-w" | "w" => rotate_view,
            "C-s" | "s" => hsplit,
            "C-v" | "v" => vsplit,
            "C-t" | "t" => transpose_view,
            "f" => goto_file_hsplit,
            "F" => goto_file_vsplit,
            "C-q" | "q" => wclose,
            "C-o" | "o" => wonly,
            "C-h" | "h" | "left" => jump_view_left,
            "C-j" | "j" | "down" => jump_view_down,
            "C-k" | "k" | "up" => jump_view_up,
            "C-l" | "l" | "right" => jump_view_right,
            "H" => swap_view_left,
            "J" => swap_view_down,
            "K" => swap_view_up,
            "L" => swap_view_right,
            "n" => { "New split scratch buffer"
                "C-s" | "s" => hsplit_new,
                "C-v" | "v" => vsplit_new,
            },
            "x" => wclose,
            "=" => wclose, // TODO: equalize sizes when available
        },

        "z" => { "View"
            "z" => align_view_center,
            "t" => align_view_top,
            "b" => align_view_bottom,
        },
    });

    // Add the shared Space keymap to normal mode
    let space_key = "space".parse::<helix_view::input::KeyEvent>().unwrap();
    normal.node_mut().unwrap().insert(space_key, space_keymap());

    let insert = keymap!({ "Insert mode"
        "esc" => normal_mode,
        "C-[" => normal_mode,
        "backspace" => delete_char_backward,
        "del" => delete_char_forward,
        "ret" => insert_newline,
        "tab" => insert_tab,
        "C-w" => delete_word_backward,
        "C-u" => kill_to_line_start,
        "left" => move_char_left,
        "right" => move_char_right,
        "up" => move_visual_line_up,
        "down" => move_visual_line_down,
        "home" => goto_line_start,
        "end" => goto_line_end_newline,
        "pageup" => page_up,
        "pagedown" => page_down,
    });

    // Shared visual mode bindings (operators, mode switching, search, etc.)
    let visual_shared = keymap!({ "Visual shared"
        "esc" => normal_mode,
        "C-[" => normal_mode,

        // Operators on selection
        "d" | "x" => delete_selection,
        "c" | "s" => change_selection,
        "y" => yank,

        ">" => indent,
        "<" => unindent,
        "=" => format_selections,

        "~" => switch_case,
        "u" => switch_to_lowercase,
        "U" => switch_to_uppercase,

        "J" => join_selections,
        "r" => replace,
        "p" => paste_after,
        "P" => paste_before,

        // Mode switching
        "v" => vim_visual_mode,
        "V" => vim_visual_line_mode,
        "C-v" => vim_visual_block_mode,

        ":" => command_mode,

        // Find / search
        "f" => extend_next_char,
        "F" => extend_prev_char,
        "t" => extend_till_char,
        "T" => extend_till_prev_char,

        "%" => match_brackets,

        "/" => search,
        "?" => rsearch,
        "n" => search_next,
        "N" => search_prev,

        // Text objects
        "i" => select_textobject_inner,
        "a" => select_textobject_around,

        "o" => flip_selections,

        "G" => goto_last_line,
        "H" => goto_window_top,
        "M" => goto_window_center,
        "L" => goto_window_bottom,

        // Scrolling
        "C-u" => half_page_up,
        "C-d" => half_page_down,
        "C-b" => page_up,
        "C-f" => page_down,
    });

    // Visual mode: character-wise motions extend selection
    let mut visual = visual_shared.clone();
    visual.merge_nodes(keymap!({ "Visual mode"
        "h" | "left" => extend_char_left,
        "j" | "down" => extend_line_down,
        "k" | "up" => extend_line_up,
        "l" | "right" => extend_char_right,

        "w" => extend_next_word_start,
        "W" => extend_next_long_word_start,
        "b" => extend_prev_word_start,
        "B" => extend_prev_long_word_start,
        "e" => extend_next_word_end,
        "E" => extend_next_long_word_end,

        "0" => goto_line_start,
        "^" => goto_first_nonwhitespace,
        "$" => goto_line_end,

        "g" => { "Goto"
            "g" => goto_file_start,
            "e" => extend_prev_word_end,
            "E" => extend_prev_long_word_end,
        },
    }));
    visual.node_mut().unwrap().insert(space_key, space_keymap());

    // Visual-line: line-wise motions
    let mut visual_line = visual_shared.clone();
    visual_line.merge_nodes(keymap!({ "Visual-Line mode"
        "j" | "down" => vim_visual_line_down,
        "k" | "up" => vim_visual_line_up,

        "g" => { "Goto"
            "g" => goto_file_start,
        },
    }));
    visual_line
        .node_mut()
        .unwrap()
        .insert(space_key, space_keymap());

    // Visual-block: character-wise motions (same as visual)
    let mut visual_block = visual_shared;
    visual_block.merge_nodes(keymap!({ "Visual-Block mode"
        "h" | "left" => extend_char_left,
        "j" | "down" => extend_line_down,
        "k" | "up" => extend_line_up,
        "l" | "right" => extend_char_right,

        "w" => extend_next_word_start,
        "W" => extend_next_long_word_start,
        "b" => extend_prev_word_start,
        "B" => extend_prev_long_word_start,
        "e" => extend_next_word_end,
        "E" => extend_next_long_word_end,

        "0" => goto_line_start,
        "^" => goto_first_nonwhitespace,
        "$" => goto_line_end,

        "g" => { "Goto"
            "g" => goto_file_start,
            "e" => extend_prev_word_end,
            "E" => extend_prev_long_word_end,
        },
    }));
    visual_block
        .node_mut()
        .unwrap()
        .insert(space_key, space_keymap());

    // Replace mode: stub for now
    let replace = keymap!({ "Replace mode"
        "esc" => normal_mode,
        "C-[" => normal_mode,
    });

    hashmap!(
        Mode::Normal => normal.clone(),
        Mode::Insert => insert,
        Mode::Select => normal, // unused in vim mode
        Mode::Visual => visual,
        Mode::VisualLine => visual_line,
        Mode::VisualBlock => visual_block,
        Mode::Replace => replace,
    )
}

/// Motion trie used during operator-pending mode.
/// Maps motion keys to the same move commands used in normal mode.
pub fn vim_motion_trie() -> KeyTrie {
    keymap!({ "Motion"
        "h" | "left" => move_char_left,
        "j" | "down" => move_visual_line_down,
        "k" | "up" => move_visual_line_up,
        "l" | "right" => move_char_right,

        "w" => move_next_word_start,
        "W" => move_next_long_word_start,
        "b" => move_prev_word_start,
        "B" => move_prev_long_word_start,
        "e" => move_next_word_end,
        "E" => move_next_long_word_end,

        "0" => goto_line_start,
        "^" => goto_first_nonwhitespace,
        "$" => goto_line_end,

        "g" => { "Goto"
            "g" => goto_file_start,
            "e" => move_prev_word_end,
            "E" => move_prev_long_word_end,
        },
        "G" => goto_last_line,

        "H" => goto_window_top,
        "M" => goto_window_center,
        "L" => goto_window_bottom,

        "f" => find_next_char,
        "F" => find_prev_char,
        "t" => find_till_char,
        "T" => till_prev_char,

        "%" => match_brackets,

        "n" => search_next,
        "N" => search_prev,
    })
}
