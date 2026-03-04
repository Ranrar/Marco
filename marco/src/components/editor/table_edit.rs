use gtk4::{gdk, prelude::*, TextBuffer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnAlignment {
    Left,
    Center,
    Right,
    None,
}

#[derive(Debug, Clone)]
struct ParsedRow {
    cells: Vec<String>,
    is_delimiter: bool,
}

#[derive(Debug, Clone)]
struct ParsedTable {
    start_line: i32,
    end_line: i32,
    rows: Vec<ParsedRow>,
    delimiter_row_idx: usize,
    col_count: usize,
}

#[derive(Debug, Clone)]
struct CursorContext {
    table: ParsedTable,
    row_idx: usize,
    col_idx: usize,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TableActionAvailability {
    pub in_table: bool,
    pub can_delete_row: bool,
    pub can_move_row_up: bool,
    pub can_move_row_down: bool,
    pub can_delete_column: bool,
    pub can_move_column_left: bool,
    pub can_move_column_right: bool,
    pub can_align_column: bool,
}

fn is_table_candidate_line(line: &str) -> bool {
    let trimmed = line.trim();
    !trimmed.is_empty() && trimmed.matches('|').count() >= 2
}

fn parse_pipe_row(line: &str) -> Option<Vec<String>> {
    let trimmed = line.trim();
    if !is_table_candidate_line(trimmed) {
        return None;
    }

    let without_left = trimmed.strip_prefix('|').unwrap_or(trimmed);
    let without_edges = without_left.strip_suffix('|').unwrap_or(without_left);
    let cells: Vec<String> = without_edges
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect();

    if cells.len() < 2 {
        return None;
    }

    Some(cells)
}

fn parse_alignment_marker(cell: &str) -> Option<ColumnAlignment> {
    let trimmed = cell.trim();
    if trimmed.is_empty() {
        return None;
    }

    let left = trimmed.starts_with(':');
    let right = trimmed.ends_with(':');
    let core = trimmed.trim_matches(':');

    if core.len() < 3 || !core.chars().all(|ch| ch == '-') {
        return None;
    }

    Some(match (left, right) {
        (true, true) => ColumnAlignment::Center,
        (true, false) => ColumnAlignment::Left,
        (false, true) => ColumnAlignment::Right,
        (false, false) => ColumnAlignment::None,
    })
}

fn alignment_marker(alignment: ColumnAlignment, width: usize) -> String {
    let width = width.max(3);
    match alignment {
        ColumnAlignment::Left => format!(":{}", "-".repeat(width)),
        ColumnAlignment::Center => {
            format!(":{}:", "-".repeat(width))
        }
        ColumnAlignment::Right => format!("{}:", "-".repeat(width)),
        ColumnAlignment::None => "-".repeat(width),
    }
}

fn is_delimiter_row(cells: &[String]) -> bool {
    !cells.is_empty()
        && cells
            .iter()
            .all(|cell| parse_alignment_marker(cell).is_some())
}

fn line_text(buffer: &TextBuffer, line: i32) -> Option<String> {
    let start = buffer.iter_at_line(line)?;
    let mut end = start;
    if !end.ends_line() {
        end.forward_to_line_end();
    }
    Some(buffer.text(&start, &end, false).to_string())
}

fn current_line(buffer: &TextBuffer) -> i32 {
    let iter = buffer.iter_at_offset(buffer.cursor_position());
    iter.line().max(0)
}

fn current_line_offset(buffer: &TextBuffer) -> i32 {
    let iter = buffer.iter_at_offset(buffer.cursor_position());
    iter.line_offset().max(0)
}

fn parse_table_around_cursor(buffer: &TextBuffer) -> Option<ParsedTable> {
    let line_count = buffer.line_count().max(0);
    if line_count == 0 {
        return None;
    }

    let mut start_line = current_line(buffer).clamp(0, line_count - 1);
    let center_line = line_text(buffer, start_line)?;
    if !is_table_candidate_line(&center_line) {
        return None;
    }

    while start_line > 0 {
        let probe = line_text(buffer, start_line - 1)?;
        if !is_table_candidate_line(&probe) {
            break;
        }
        start_line -= 1;
    }

    let mut end_line = current_line(buffer).clamp(0, line_count - 1);
    while end_line + 1 < line_count {
        let probe = line_text(buffer, end_line + 1)?;
        if !is_table_candidate_line(&probe) {
            break;
        }
        end_line += 1;
    }

    let mut rows = Vec::new();
    for line in start_line..=end_line {
        let text = line_text(buffer, line)?;
        let cells = parse_pipe_row(&text)?;
        rows.push(ParsedRow {
            is_delimiter: is_delimiter_row(&cells),
            cells,
        });
    }

    let delimiter_row_idx = rows.iter().position(|row| row.is_delimiter)?;
    if delimiter_row_idx + 1 >= rows.len() {
        return None;
    }

    let col_count = rows[delimiter_row_idx].cells.len().max(1);
    for row in &mut rows {
        if row.cells.len() < col_count {
            row.cells.resize(col_count, String::new());
        } else if row.cells.len() > col_count {
            row.cells.truncate(col_count);
        }
    }

    Some(ParsedTable {
        start_line,
        end_line,
        rows,
        delimiter_row_idx,
        col_count,
    })
}

fn infer_column_index_from_cursor(line: &str, col_count: usize, line_offset: i32) -> usize {
    let mut bars = Vec::new();
    for (idx, ch) in line.chars().enumerate() {
        if ch == '|' {
            bars.push(idx as i32);
        }
    }

    if bars.len() >= 2 {
        for i in 0..(bars.len() - 1) {
            if line_offset <= bars[i + 1] {
                return i.min(col_count.saturating_sub(1));
            }
        }
        return col_count.saturating_sub(1);
    }

    0
}

fn cursor_context(buffer: &TextBuffer) -> Option<CursorContext> {
    let table = parse_table_around_cursor(buffer)?;
    let line = current_line(buffer);
    let row_idx = (line - table.start_line).clamp(0, (table.rows.len() - 1) as i32) as usize;
    let raw_line = line_text(buffer, line)?;
    let col_idx =
        infer_column_index_from_cursor(&raw_line, table.col_count, current_line_offset(buffer));

    Some(CursorContext {
        table,
        row_idx,
        col_idx,
    })
}

fn non_delimiter_row_indices(table: &ParsedTable) -> Vec<usize> {
    table
        .rows
        .iter()
        .enumerate()
        .filter_map(|(idx, row)| (!row.is_delimiter).then_some(idx))
        .collect()
}

fn nearest_editable_row_idx(table: &ParsedTable, row_idx: usize) -> Option<usize> {
    if row_idx < table.rows.len() && !table.rows[row_idx].is_delimiter {
        return Some(row_idx);
    }

    ((row_idx + 1)..table.rows.len())
        .find(|idx| !table.rows[*idx].is_delimiter)
        .or_else(|| {
            (0..row_idx)
                .rev()
                .find(|idx| !table.rows[*idx].is_delimiter)
        })
}

fn replace_table_in_buffer(buffer: &TextBuffer, table: &ParsedTable, lines: &[String]) {
    let mut replacement = lines.join("\n");

    let line_count = buffer.line_count().max(0);
    let has_line_after = table.end_line + 1 < line_count;
    if has_line_after {
        replacement.push('\n');
    }

    let Some(mut start_iter) = buffer.iter_at_line(table.start_line) else {
        return;
    };
    let mut end_iter = if has_line_after {
        match buffer.iter_at_line(table.end_line + 1) {
            Some(iter) => iter,
            None => buffer.end_iter(),
        }
    } else {
        buffer.end_iter()
    };

    buffer.begin_user_action();
    buffer.delete(&mut start_iter, &mut end_iter);
    buffer.insert(&mut start_iter, &replacement);
    buffer.end_user_action();
}

fn cell_start_char_index(line: &str, col_idx: usize) -> usize {
    let mut bars = Vec::new();
    for (idx, ch) in line.chars().enumerate() {
        if ch == '|' {
            bars.push(idx);
        }
    }

    if bars.len() >= 2 {
        let left = bars
            .get(col_idx)
            .copied()
            .unwrap_or(*bars.last().unwrap_or(&0));
        return left.saturating_add(2);
    }

    0
}

fn place_cursor_at_table_cell(buffer: &TextBuffer, line: i32, col_idx: usize) {
    let Some(line_iter) = buffer.iter_at_line(line) else {
        return;
    };
    let Some(text) = line_text(buffer, line) else {
        return;
    };
    let char_index = cell_start_char_index(&text, col_idx);
    let offset = line_iter.offset().saturating_add(char_index as i32);
    let iter = buffer.iter_at_offset(offset);
    buffer.place_cursor(&iter);
}

fn format_rows(table: &ParsedTable) -> Vec<String> {
    let mut widths = vec![3usize; table.col_count];

    for row in &table.rows {
        if row.is_delimiter {
            continue;
        }
        for (idx, cell) in row.cells.iter().enumerate() {
            widths[idx] = widths[idx].max(cell.trim().chars().count().max(1));
        }
    }

    let alignments: Vec<ColumnAlignment> = table.rows[table.delimiter_row_idx]
        .cells
        .iter()
        .map(|cell| parse_alignment_marker(cell).unwrap_or(ColumnAlignment::None))
        .collect();

    table
        .rows
        .iter()
        .map(|row| {
            let cells: Vec<String> = if row.is_delimiter {
                row.cells
                    .iter()
                    .enumerate()
                    .map(|(idx, _)| alignment_marker(alignments[idx], widths[idx]))
                    .collect()
            } else {
                row.cells
                    .iter()
                    .enumerate()
                    .map(|(idx, cell)| {
                        let trimmed = cell.trim();
                        format!("{trimmed:<width$}", width = widths[idx])
                    })
                    .collect()
            };
            format!("| {} |", cells.join(" | "))
        })
        .collect()
}

fn apply_navigation_result(view: &sourceview5::View, new_line: i32, new_col: usize) {
    let buffer = view.buffer();
    place_cursor_at_table_cell(&buffer.clone().upcast::<TextBuffer>(), new_line, new_col);

    let mut iter = buffer.iter_at_offset(buffer.cursor_position());
    view.scroll_to_iter(&mut iter, 0.12, true, 0.0, 0.25);
    view.grab_focus();
}

pub fn table_action_availability(buffer: &TextBuffer) -> TableActionAvailability {
    let Some(ctx) = cursor_context(buffer) else {
        return TableActionAvailability::default();
    };

    let editable_rows = non_delimiter_row_indices(&ctx.table);
    let Some(current_editable) = nearest_editable_row_idx(&ctx.table, ctx.row_idx) else {
        return TableActionAvailability {
            in_table: true,
            can_align_column: true,
            ..Default::default()
        };
    };

    let editable_pos = editable_rows
        .iter()
        .position(|idx| *idx == current_editable)
        .unwrap_or(0);

    TableActionAvailability {
        in_table: true,
        can_delete_row: editable_rows.len() > 1,
        can_move_row_up: editable_pos > 0,
        can_move_row_down: editable_pos + 1 < editable_rows.len(),
        can_delete_column: ctx.table.col_count > 1,
        can_move_column_left: ctx.col_idx > 0,
        can_move_column_right: ctx.col_idx + 1 < ctx.table.col_count,
        can_align_column: true,
    }
}

pub fn format_table_at_cursor(buffer: &TextBuffer) -> bool {
    let Some(ctx) = cursor_context(buffer) else {
        return false;
    };

    let formatted = format_rows(&ctx.table);
    replace_table_in_buffer(buffer, &ctx.table, &formatted);
    place_cursor_at_table_cell(
        buffer,
        ctx.table.start_line + ctx.row_idx as i32,
        ctx.col_idx,
    );
    true
}

pub fn insert_row_above(buffer: &TextBuffer) -> bool {
    let Some(ctx) = cursor_context(buffer) else {
        return false;
    };
    let Some(target) = nearest_editable_row_idx(&ctx.table, ctx.row_idx) else {
        return false;
    };

    let mut table = ctx.table.clone();
    table.rows.insert(
        target,
        ParsedRow {
            cells: vec![String::new(); table.col_count],
            is_delimiter: false,
        },
    );

    let lines = format_rows(&table);
    replace_table_in_buffer(buffer, &ctx.table, &lines);
    place_cursor_at_table_cell(buffer, table.start_line + target as i32, ctx.col_idx);
    true
}

pub fn insert_row_below(buffer: &TextBuffer) -> bool {
    let Some(ctx) = cursor_context(buffer) else {
        return false;
    };
    let Some(target) = nearest_editable_row_idx(&ctx.table, ctx.row_idx) else {
        return false;
    };

    let mut table = ctx.table.clone();
    let insert_idx = (target + 1).min(table.rows.len());
    table.rows.insert(
        insert_idx,
        ParsedRow {
            cells: vec![String::new(); table.col_count],
            is_delimiter: false,
        },
    );

    let lines = format_rows(&table);
    replace_table_in_buffer(buffer, &ctx.table, &lines);
    place_cursor_at_table_cell(buffer, table.start_line + insert_idx as i32, ctx.col_idx);
    true
}

pub fn delete_current_row(buffer: &TextBuffer) -> bool {
    let Some(ctx) = cursor_context(buffer) else {
        return false;
    };

    let editable_rows = non_delimiter_row_indices(&ctx.table);
    if editable_rows.len() <= 1 {
        return false;
    }

    let Some(target) = nearest_editable_row_idx(&ctx.table, ctx.row_idx) else {
        return false;
    };

    let mut table = ctx.table.clone();
    table.rows.remove(target);

    let lines = format_rows(&table);
    replace_table_in_buffer(buffer, &ctx.table, &lines);

    let new_editable = non_delimiter_row_indices(&table);
    if let Some(&line_row_idx) = new_editable
        .iter()
        .find(|&&idx| idx >= target)
        .or_else(|| new_editable.last())
    {
        place_cursor_at_table_cell(
            buffer,
            table.start_line + line_row_idx as i32,
            ctx.col_idx.min(table.col_count.saturating_sub(1)),
        );
    }

    true
}

pub fn move_current_row_up(buffer: &TextBuffer) -> bool {
    let Some(ctx) = cursor_context(buffer) else {
        return false;
    };
    let Some(target) = nearest_editable_row_idx(&ctx.table, ctx.row_idx) else {
        return false;
    };
    let Some(upper) = (0..target)
        .rev()
        .find(|idx| !ctx.table.rows[*idx].is_delimiter)
    else {
        return false;
    };

    let mut table = ctx.table.clone();
    table.rows.swap(target, upper);

    let lines = format_rows(&table);
    replace_table_in_buffer(buffer, &ctx.table, &lines);
    place_cursor_at_table_cell(buffer, table.start_line + upper as i32, ctx.col_idx);
    true
}

pub fn move_current_row_down(buffer: &TextBuffer) -> bool {
    let Some(ctx) = cursor_context(buffer) else {
        return false;
    };
    let Some(target) = nearest_editable_row_idx(&ctx.table, ctx.row_idx) else {
        return false;
    };
    let Some(lower) =
        ((target + 1)..ctx.table.rows.len()).find(|idx| !ctx.table.rows[*idx].is_delimiter)
    else {
        return false;
    };

    let mut table = ctx.table.clone();
    table.rows.swap(target, lower);

    let lines = format_rows(&table);
    replace_table_in_buffer(buffer, &ctx.table, &lines);
    place_cursor_at_table_cell(buffer, table.start_line + lower as i32, ctx.col_idx);
    true
}

pub fn insert_column_left(buffer: &TextBuffer) -> bool {
    insert_column(buffer, false)
}

pub fn insert_column_right(buffer: &TextBuffer) -> bool {
    insert_column(buffer, true)
}

fn insert_column(buffer: &TextBuffer, to_right: bool) -> bool {
    let Some(ctx) = cursor_context(buffer) else {
        return false;
    };

    let mut table = ctx.table.clone();
    let insert_idx = if to_right {
        (ctx.col_idx + 1).min(table.col_count)
    } else {
        ctx.col_idx.min(table.col_count)
    };

    for row in &mut table.rows {
        let value = if row.is_delimiter {
            "---".to_string()
        } else {
            String::new()
        };
        row.cells.insert(insert_idx, value);
    }
    table.col_count += 1;

    let lines = format_rows(&table);
    replace_table_in_buffer(buffer, &ctx.table, &lines);
    place_cursor_at_table_cell(buffer, table.start_line + ctx.row_idx as i32, insert_idx);
    true
}

pub fn delete_current_column(buffer: &TextBuffer) -> bool {
    let Some(ctx) = cursor_context(buffer) else {
        return false;
    };
    if ctx.table.col_count <= 1 {
        return false;
    }

    let mut table = ctx.table.clone();
    for row in &mut table.rows {
        row.cells.remove(ctx.col_idx.min(table.col_count - 1));
    }
    table.col_count -= 1;

    let lines = format_rows(&table);
    replace_table_in_buffer(buffer, &ctx.table, &lines);
    place_cursor_at_table_cell(
        buffer,
        table.start_line + ctx.row_idx as i32,
        ctx.col_idx.min(table.col_count - 1),
    );
    true
}

pub fn move_current_column_left(buffer: &TextBuffer) -> bool {
    move_column(buffer, false)
}

pub fn move_current_column_right(buffer: &TextBuffer) -> bool {
    move_column(buffer, true)
}

fn move_column(buffer: &TextBuffer, to_right: bool) -> bool {
    let Some(ctx) = cursor_context(buffer) else {
        return false;
    };

    let source = ctx.col_idx;
    let target = if to_right {
        if source + 1 >= ctx.table.col_count {
            return false;
        }
        source + 1
    } else {
        if source == 0 {
            return false;
        }
        source - 1
    };

    let mut table = ctx.table.clone();
    for row in &mut table.rows {
        row.cells.swap(source, target);
    }

    let lines = format_rows(&table);
    replace_table_in_buffer(buffer, &ctx.table, &lines);
    place_cursor_at_table_cell(buffer, table.start_line + ctx.row_idx as i32, target);
    true
}

pub fn align_current_column(buffer: &TextBuffer, alignment: ColumnAlignment) -> bool {
    let Some(ctx) = cursor_context(buffer) else {
        return false;
    };

    let mut table = ctx.table.clone();
    table.rows[table.delimiter_row_idx].cells[ctx.col_idx] = alignment_marker(alignment, 3);

    let lines = format_rows(&table);
    replace_table_in_buffer(buffer, &ctx.table, &lines);
    place_cursor_at_table_cell(buffer, table.start_line + ctx.row_idx as i32, ctx.col_idx);
    true
}

pub fn handle_table_navigation_key(
    view: &sourceview5::View,
    key: gdk::Key,
    state: gdk::ModifierType,
) -> bool {
    let buffer = view.buffer();
    let text_buffer: TextBuffer = buffer.clone().upcast();

    if text_buffer.selection_bounds().is_some() {
        return false;
    }

    match key {
        gdk::Key::Tab => {
            if state.contains(gdk::ModifierType::SHIFT_MASK) {
                navigate_prev_cell(view)
            } else {
                navigate_next_cell_or_append_row(view)
            }
        }
        gdk::Key::Return => {
            if state.contains(gdk::ModifierType::SHIFT_MASK) {
                false
            } else {
                navigate_next_row_same_column_or_append(view)
            }
        }
        _ => false,
    }
}

fn navigate_prev_cell(view: &sourceview5::View) -> bool {
    let buffer: TextBuffer = view.buffer().upcast();
    let Some(ctx) = cursor_context(&buffer) else {
        return false;
    };

    if ctx.col_idx > 0 {
        apply_navigation_result(
            view,
            ctx.table.start_line + ctx.row_idx as i32,
            ctx.col_idx - 1,
        );
        return true;
    }

    let prev_row = (0..ctx.row_idx)
        .rev()
        .find(|idx| !ctx.table.rows[*idx].is_delimiter);
    if let Some(prev_row) = prev_row {
        apply_navigation_result(
            view,
            ctx.table.start_line + prev_row as i32,
            ctx.table.col_count.saturating_sub(1),
        );
        return true;
    }

    false
}

fn navigate_next_cell_or_append_row(view: &sourceview5::View) -> bool {
    let buffer: TextBuffer = view.buffer().upcast();
    let Some(ctx) = cursor_context(&buffer) else {
        return false;
    };

    if ctx.col_idx + 1 < ctx.table.col_count {
        apply_navigation_result(
            view,
            ctx.table.start_line + ctx.row_idx as i32,
            ctx.col_idx + 1,
        );
        return true;
    }

    let next_row =
        ((ctx.row_idx + 1)..ctx.table.rows.len()).find(|idx| !ctx.table.rows[*idx].is_delimiter);
    if let Some(next_row) = next_row {
        apply_navigation_result(view, ctx.table.start_line + next_row as i32, 0);
        return true;
    }

    let mut table = ctx.table.clone();
    let insert_idx = table.rows.len();
    table.rows.push(ParsedRow {
        cells: vec![String::new(); table.col_count],
        is_delimiter: false,
    });
    let lines = format_rows(&table);
    replace_table_in_buffer(&buffer, &ctx.table, &lines);
    apply_navigation_result(view, table.start_line + insert_idx as i32, 0);
    true
}

fn navigate_next_row_same_column_or_append(view: &sourceview5::View) -> bool {
    let buffer: TextBuffer = view.buffer().upcast();
    let Some(ctx) = cursor_context(&buffer) else {
        return false;
    };

    let next_row =
        ((ctx.row_idx + 1)..ctx.table.rows.len()).find(|idx| !ctx.table.rows[*idx].is_delimiter);
    if let Some(next_row) = next_row {
        apply_navigation_result(view, ctx.table.start_line + next_row as i32, ctx.col_idx);
        return true;
    }

    let mut table = ctx.table.clone();
    let insert_idx = table.rows.len();
    table.rows.push(ParsedRow {
        cells: vec![String::new(); table.col_count],
        is_delimiter: false,
    });
    let lines = format_rows(&table);
    replace_table_in_buffer(&buffer, &ctx.table, &lines);
    apply_navigation_result(view, table.start_line + insert_idx as i32, ctx.col_idx);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_alignment_marker() {
        assert_eq!(parse_alignment_marker(":---"), Some(ColumnAlignment::Left));
        assert_eq!(
            parse_alignment_marker(":---:"),
            Some(ColumnAlignment::Center)
        );
        assert_eq!(parse_alignment_marker("---:"), Some(ColumnAlignment::Right));
        assert_eq!(parse_alignment_marker("---"), Some(ColumnAlignment::None));
    }

    #[test]
    fn smoke_test_format_rows_normalizes_spacing() {
        let table = ParsedTable {
            start_line: 0,
            end_line: 2,
            delimiter_row_idx: 1,
            col_count: 2,
            rows: vec![
                ParsedRow {
                    cells: vec!["A".to_string(), "Long".to_string()],
                    is_delimiter: false,
                },
                ParsedRow {
                    cells: vec![":---".to_string(), "---:".to_string()],
                    is_delimiter: true,
                },
                ParsedRow {
                    cells: vec!["1".to_string(), "2".to_string()],
                    is_delimiter: false,
                },
            ],
        };

        let lines = format_rows(&table);
        assert_eq!(lines[0], "| A   | Long |");
        assert_eq!(lines[1], "| :--- | ----: |");
        assert_eq!(lines[2], "| 1   | 2    |");
    }
}
