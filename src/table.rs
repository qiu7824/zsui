use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{Dp, HorizontalAlign};

/// Stable application-owned identity for one DataGrid column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ZsTableColumnId(u64);

impl ZsTableColumnId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for ZsTableColumnId {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

/// Stable application-owned identity for one DataGrid row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ZsTableRowId(u64);

impl ZsTableRowId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for ZsTableRowId {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ZsTableColumnWidth {
    Fixed(Dp),
    Fill(u16),
}

impl Default for ZsTableColumnWidth {
    fn default() -> Self {
        Self::Fill(1)
    }
}

/// Application-owned metadata for one read-only DataGrid column.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZsTableColumn {
    id: ZsTableColumnId,
    header: String,
    width: ZsTableColumnWidth,
    alignment: HorizontalAlign,
    sortable: bool,
}

impl ZsTableColumn {
    pub fn new(id: impl Into<ZsTableColumnId>, header: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            header: header.into(),
            width: ZsTableColumnWidth::default(),
            alignment: HorizontalAlign::Start,
            sortable: false,
        }
    }

    pub fn fixed_width(mut self, width: Dp) -> Self {
        self.width = ZsTableColumnWidth::Fixed(width);
        self
    }

    pub fn fill_width(mut self, weight: u16) -> Self {
        self.width = ZsTableColumnWidth::Fill(weight.max(1));
        self
    }

    pub fn alignment(mut self, alignment: HorizontalAlign) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }

    pub const fn id(&self) -> ZsTableColumnId {
        self.id
    }

    pub fn header(&self) -> &str {
        &self.header
    }

    pub const fn width(&self) -> ZsTableColumnWidth {
        self.width
    }

    pub const fn column_alignment(&self) -> HorizontalAlign {
        self.alignment
    }

    pub const fn is_sortable(&self) -> bool {
        self.sortable
    }
}

/// Application-owned display values for one read-only DataGrid row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsTableRow {
    id: ZsTableRowId,
    cells: Vec<String>,
}

impl ZsTableRow {
    pub fn new(
        id: impl Into<ZsTableRowId>,
        cells: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            id: id.into(),
            cells: cells.into_iter().map(Into::into).collect(),
        }
    }

    pub const fn id(&self) -> ZsTableRowId {
        self.id
    }

    pub fn cells(&self) -> &[String] {
        &self.cells
    }

    pub fn cell(&self, index: usize) -> &str {
        self.cells
            .get(index)
            .map(String::as_str)
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsTableSortDirection {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsTableSort {
    pub column: ZsTableColumnId,
    pub direction: ZsTableSortDirection,
}

impl ZsTableSort {
    pub const fn new(column: ZsTableColumnId, direction: ZsTableSortDirection) -> Self {
        Self { column, direction }
    }
}

/// Read-only explicit state used by native input routing and diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsTableViewState {
    pub selected: Option<ZsTableRowId>,
    pub sort: Option<ZsTableSort>,
    pub rows: Vec<ZsTableRowId>,
}

impl ZsTableViewState {
    pub fn contains_row(&self, row: ZsTableRowId) -> bool {
        self.rows.contains(&row)
    }

    pub fn first_row(&self) -> Option<ZsTableRowId> {
        self.rows.first().copied()
    }

    pub fn last_row(&self) -> Option<ZsTableRowId> {
        self.rows.last().copied()
    }

    pub fn relative_row(&self, offset: isize) -> Option<ZsTableRowId> {
        let current = self
            .selected
            .and_then(|selected| self.rows.iter().position(|row| *row == selected));
        let index = match current {
            Some(index) => index
                .saturating_add_signed(offset)
                .min(self.rows.len().saturating_sub(1)),
            None if offset < 0 => self.rows.len().checked_sub(1)?,
            None => 0,
        };
        self.rows.get(index).copied()
    }
}

pub(crate) fn unique_table_columns(columns: &[ZsTableColumn]) -> Vec<&ZsTableColumn> {
    let mut seen = BTreeSet::new();
    columns
        .iter()
        .filter(|column| seen.insert(column.id()))
        .collect()
}

pub(crate) fn unique_table_rows(rows: &[ZsTableRow]) -> Vec<&ZsTableRow> {
    let mut seen = BTreeSet::new();
    rows.iter().filter(|row| seen.insert(row.id())).collect()
}

pub(crate) fn next_table_sort(
    columns: &[ZsTableColumn],
    current: Option<ZsTableSort>,
    column: ZsTableColumnId,
) -> Option<ZsTableSort> {
    let sortable = unique_table_columns(columns)
        .into_iter()
        .find(|candidate| candidate.id() == column)
        .is_some_and(ZsTableColumn::is_sortable);
    if !sortable {
        return None;
    }
    let direction = match current {
        Some(sort)
            if sort.column == column && sort.direction == ZsTableSortDirection::Ascending =>
        {
            ZsTableSortDirection::Descending
        }
        _ => ZsTableSortDirection::Ascending,
    };
    Some(ZsTableSort::new(column, direction))
}

pub(crate) fn table_view_state(
    rows: &[ZsTableRow],
    selected: Option<ZsTableRowId>,
    sort: Option<ZsTableSort>,
) -> ZsTableViewState {
    ZsTableViewState {
        selected,
        sort,
        rows: unique_table_rows(rows)
            .into_iter()
            .map(ZsTableRow::id)
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_state_preserves_strong_ids_and_deduplicates_rows() {
        let state = table_view_state(
            &[
                ZsTableRow::new(1, ["Alpha"]),
                ZsTableRow::new(2, ["Beta"]),
                ZsTableRow::new(1, ["Duplicate"]),
            ],
            Some(ZsTableRowId::new(2)),
            None,
        );

        assert_eq!(state.rows, vec![1_u64.into(), 2_u64.into()]);
        assert_eq!(state.relative_row(-1), Some(ZsTableRowId::new(1)));
        assert_eq!(state.relative_row(1), Some(ZsTableRowId::new(2)));
    }

    #[test]
    fn sortable_columns_cycle_explicit_direction() {
        let columns = [
            ZsTableColumn::new(1, "Name").sortable(true),
            ZsTableColumn::new(2, "Status"),
        ];
        let ascending = next_table_sort(&columns, None, ZsTableColumnId::new(1));
        assert_eq!(
            ascending,
            Some(ZsTableSort::new(
                ZsTableColumnId::new(1),
                ZsTableSortDirection::Ascending
            ))
        );
        assert_eq!(
            next_table_sort(&columns, ascending, ZsTableColumnId::new(1)),
            Some(ZsTableSort::new(
                ZsTableColumnId::new(1),
                ZsTableSortDirection::Descending
            ))
        );
        assert_eq!(
            next_table_sort(&columns, None, ZsTableColumnId::new(2)),
            None
        );
    }
}
