pub use helix_view::editor::{VimOperator, VimState};

use helix_core::{Range, RopeSlice};

use crate::commands;

/// Whether a motion is linewise, exclusive, or inclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionKind {
    /// Operate on full lines.
    Linewise,
    /// The end position is NOT included in the range.
    Exclusive,
    /// The end position IS included in the range.
    Inclusive,
}

/// Look up the MotionKind for a command by name.
pub fn motion_kind(command_name: &str) -> MotionKind {
    match command_name {
        // Linewise motions
        "move_line_up"
        | "move_line_down"
        | "move_visual_line_up"
        | "move_visual_line_down"
        | "goto_file_start"
        | "goto_last_line"
        | "goto_window_top"
        | "goto_window_center"
        | "goto_window_bottom"
        | "half_page_up"
        | "half_page_down"
        | "page_up"
        | "page_down" => MotionKind::Linewise,

        // Inclusive motions
        "move_next_word_end"
        | "move_next_long_word_end"
        | "move_prev_word_end"
        | "move_prev_long_word_end"
        | "find_next_char"
        | "find_till_char"
        | "goto_line_end"
        | "match_brackets" => MotionKind::Inclusive,

        // Exclusive motions (default)
        _ => MotionKind::Exclusive,
    }
}

/// Given the cursor position before and after a motion, plus the motion kind,
/// compute the (from, to) character range the operator should act on.
/// Returns (from, to) where from <= to.
pub fn compute_operator_range(
    text: RopeSlice,
    before: usize,
    after: usize,
    kind: MotionKind,
) -> (usize, usize) {
    match kind {
        MotionKind::Linewise => {
            let (min_pos, max_pos) = if before <= after {
                (before, after)
            } else {
                (after, before)
            };
            let first_line = text.char_to_line(min_pos);
            let last_line = text.char_to_line(max_pos);
            let start = text.line_to_char(first_line);
            let end = if last_line + 1 < text.len_lines() {
                text.line_to_char(last_line + 1)
            } else {
                text.len_chars()
            };
            (start, end)
        }
        MotionKind::Exclusive => {
            if before <= after {
                (before, after)
            } else {
                (after, before)
            }
        }
        MotionKind::Inclusive => {
            let (min_pos, max_pos) = if before <= after {
                (before, after)
            } else {
                (after, before)
            };
            let end = helix_core::graphemes::next_grapheme_boundary(text, max_pos);
            (min_pos, end)
        }
    }
}

/// Apply a vim operator to the current selection.
pub fn apply_operator(cxt: &mut commands::Context, operator: VimOperator) {
    match operator {
        VimOperator::Delete => {
            commands::MappableCommand::delete_selection.execute(cxt);
        }
        VimOperator::Change => {
            commands::MappableCommand::change_selection.execute(cxt);
        }
        VimOperator::Yank => {
            commands::MappableCommand::yank.execute(cxt);
            let (view, doc) = current!(cxt.editor);
            let selection = doc
                .selection(view.id)
                .clone()
                .transform(|range| Range::point(range.from()));
            doc.set_selection(view.id, selection);
        }
        VimOperator::Indent => {
            commands::MappableCommand::indent.execute(cxt);
            let (view, doc) = current!(cxt.editor);
            let selection = doc
                .selection(view.id)
                .clone()
                .transform(|range| Range::point(range.from()));
            doc.set_selection(view.id, selection);
        }
        VimOperator::Outdent => {
            commands::MappableCommand::unindent.execute(cxt);
            let (view, doc) = current!(cxt.editor);
            let selection = doc
                .selection(view.id)
                .clone()
                .transform(|range| Range::point(range.from()));
            doc.set_selection(view.id, selection);
        }
        VimOperator::AutoIndent => {
            commands::MappableCommand::format_selections.execute(cxt);
            let (view, doc) = current!(cxt.editor);
            let selection = doc
                .selection(view.id)
                .clone()
                .transform(|range| Range::point(range.from()));
            doc.set_selection(view.id, selection);
        }
        VimOperator::Uppercase => {
            commands::MappableCommand::switch_to_uppercase.execute(cxt);
            let (view, doc) = current!(cxt.editor);
            let selection = doc
                .selection(view.id)
                .clone()
                .transform(|range| Range::point(range.from()));
            doc.set_selection(view.id, selection);
        }
        VimOperator::Lowercase => {
            commands::MappableCommand::switch_to_lowercase.execute(cxt);
            let (view, doc) = current!(cxt.editor);
            let selection = doc
                .selection(view.id)
                .clone()
                .transform(|range| Range::point(range.from()));
            doc.set_selection(view.id, selection);
        }
        VimOperator::ToggleCase => {
            commands::MappableCommand::switch_case.execute(cxt);
            let (view, doc) = current!(cxt.editor);
            let selection = doc
                .selection(view.id)
                .clone()
                .transform(|range| Range::point(range.from()));
            doc.set_selection(view.id, selection);
        }
    }

    cxt.editor.exit_select_mode();
}

/// Compute a linewise range for `count` lines starting from the cursor position.
/// Used for doubled operators (dd, yy, cc, etc.).
pub fn linewise_range(text: RopeSlice, cursor_pos: usize, count: usize) -> (usize, usize) {
    let cursor_line = text.char_to_line(cursor_pos);
    let last_line = (cursor_line + count)
        .min(text.len_lines())
        .saturating_sub(1);
    let start = text.line_to_char(cursor_line);
    let end = if last_line + 1 < text.len_lines() {
        text.line_to_char(last_line + 1)
    } else {
        text.len_chars()
    };
    (start, end)
}
