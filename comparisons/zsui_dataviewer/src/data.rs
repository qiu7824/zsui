use std::{
    fmt,
    path::{Path, PathBuf},
    time::Instant,
};

use duckdb::{Connection, Row, types::ValueRef};

pub const PREVIEW_LIMIT: usize = 200;
pub const DATASET_ALIAS: &str = "dataset";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataSetFormat {
    Csv,
    Tsv,
    Parquet,
    Json,
}

impl DataSetFormat {
    pub fn from_path(path: &Path) -> Result<Self, DataError> {
        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        match extension.to_ascii_lowercase().as_str() {
            "csv" => Ok(Self::Csv),
            "tsv" => Ok(Self::Tsv),
            "parquet" => Ok(Self::Parquet),
            "json" | "jsonl" | "ndjson" => Ok(Self::Json),
            _ => Err(DataError::UnsupportedFormat(extension.to_string())),
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Csv => "CSV",
            Self::Tsv => "TSV",
            Self::Parquet => "PARQUET",
            Self::Json => "JSON",
        }
    }

    fn relation_source(self, quoted_path: &str) -> String {
        match self {
            Self::Csv => format!("read_csv_auto({quoted_path}, header = true)"),
            Self::Tsv => {
                format!("read_csv_auto({quoted_path}, delim = '\\t', header = true)")
            }
            Self::Parquet => format!("read_parquet({quoted_path})"),
            Self::Json => format!("read_json_auto({quoted_path})"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedDataSet {
    pub path: PathBuf,
    pub file_name: String,
    pub alias: String,
    pub format: DataSetFormat,
    pub columns: Vec<String>,
    pub preview_limit: usize,
}

impl LoadedDataSet {
    pub fn default_sql(&self) -> String {
        format!(
            "SELECT * FROM {} LIMIT {};",
            quote_identifier(&self.alias),
            self.preview_limit
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryRow {
    pub id: u64,
    pub cells: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<QueryRow>,
    pub elapsed_micros: u128,
    pub truncated: bool,
}

impl QueryResult {
    pub fn filtered_rows<'a>(&'a self, filter: &str) -> Vec<&'a QueryRow> {
        let terms = filter
            .split_whitespace()
            .map(str::to_lowercase)
            .collect::<Vec<_>>();
        if terms.is_empty() {
            return self.rows.iter().collect();
        }
        self.rows
            .iter()
            .filter(|row| {
                let searchable = row.cells.join("\u{1f}").to_lowercase();
                terms.iter().all(|term| searchable.contains(term))
            })
            .collect()
    }

    pub fn sort_by_column(&mut self, column: usize, descending: bool) {
        self.rows.sort_by(|left, right| {
            let ordering = left
                .cells
                .get(column)
                .map(String::as_str)
                .unwrap_or_default()
                .cmp(
                    right
                        .cells
                        .get(column)
                        .map(String::as_str)
                        .unwrap_or_default(),
                );
            if descending {
                ordering.reverse()
            } else {
                ordering
            }
        });
    }

    pub fn row(&self, id: u64) -> Option<&QueryRow> {
        self.rows.iter().find(|row| row.id == id)
    }
}

#[derive(Debug)]
pub enum DataError {
    UnsupportedFormat(String),
    DuckDb(duckdb::Error),
}

impl fmt::Display for DataError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedFormat(extension) if extension.is_empty() => {
                formatter.write_str("文件没有可识别的扩展名")
            }
            Self::UnsupportedFormat(extension) => {
                write!(formatter, "不支持 .{extension} 文件")
            }
            Self::DuckDb(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for DataError {}

impl From<duckdb::Error> for DataError {
    fn from(value: duckdb::Error) -> Self {
        Self::DuckDb(value)
    }
}

pub fn load_and_preview(
    path: impl AsRef<Path>,
) -> Result<(LoadedDataSet, String, QueryResult), DataError> {
    let path = path.as_ref();
    let format = DataSetFormat::from_path(path)?;
    let connection = open_relation(path, format)?;
    let columns = relation_columns(&connection)?;
    let data_set = LoadedDataSet {
        path: path.to_path_buf(),
        file_name: path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("dataset")
            .to_string(),
        alias: DATASET_ALIAS.to_string(),
        format,
        columns,
        preview_limit: PREVIEW_LIMIT,
    };
    let sql = data_set.default_sql();
    let result = query_connection(&connection, &sql)?;
    Ok((data_set, sql, result))
}

pub fn run_query(path: impl AsRef<Path>, sql: &str) -> Result<QueryResult, DataError> {
    let path = path.as_ref();
    let format = DataSetFormat::from_path(path)?;
    let connection = open_relation(path, format)?;
    query_connection(&connection, sql)
}

fn open_relation(path: &Path, format: DataSetFormat) -> Result<Connection, DataError> {
    let connection = Connection::open_in_memory()?;
    let source = format.relation_source(&quote_literal(&path.to_string_lossy()));
    connection.execute_batch(&format!(
        "CREATE OR REPLACE VIEW {} AS SELECT * FROM {source};",
        quote_identifier(DATASET_ALIAS)
    ))?;
    Ok(connection)
}

fn relation_columns(connection: &Connection) -> Result<Vec<String>, DataError> {
    let mut statement = connection.prepare(&format!(
        "SELECT * FROM {} LIMIT 0",
        quote_identifier(DATASET_ALIAS)
    ))?;
    let rows = statement.query([])?;
    Ok(rows
        .as_ref()
        .expect("DuckDB query rows retain their statement")
        .column_names())
}

fn query_connection(connection: &Connection, sql: &str) -> Result<QueryResult, DataError> {
    let started = Instant::now();
    let mut statement = connection.prepare(sql)?;
    let mut cursor = statement.query([])?;
    let executed_statement = cursor
        .as_ref()
        .expect("DuckDB query rows retain their statement");
    let columns = executed_statement.column_names();
    let column_count = executed_statement.column_count();
    let mut rows = Vec::new();
    let mut truncated = false;
    while let Some(row) = cursor.next()? {
        if rows.len() == PREVIEW_LIMIT {
            truncated = true;
            break;
        }
        let cells = (0..column_count)
            .map(|index| display_cell(row, index))
            .collect::<Result<Vec<_>, _>>()?;
        rows.push(QueryRow {
            id: rows.len() as u64 + 1,
            cells,
        });
    }
    Ok(QueryResult {
        columns,
        rows,
        elapsed_micros: started.elapsed().as_micros(),
        truncated,
    })
}

fn display_cell(row: &Row<'_>, index: usize) -> Result<String, duckdb::Error> {
    let value = row.get_ref(index)?;
    Ok(match value {
        ValueRef::Null => "NULL".to_string(),
        ValueRef::Boolean(value) => value.to_string(),
        ValueRef::TinyInt(value) => value.to_string(),
        ValueRef::SmallInt(value) => value.to_string(),
        ValueRef::Int(value) => value.to_string(),
        ValueRef::BigInt(value) => value.to_string(),
        ValueRef::HugeInt(value) => value.to_string(),
        ValueRef::UHugeInt(value) => value.to_string(),
        ValueRef::UTinyInt(value) => value.to_string(),
        ValueRef::USmallInt(value) => value.to_string(),
        ValueRef::UInt(value) => value.to_string(),
        ValueRef::UBigInt(value) => value.to_string(),
        ValueRef::Float(value) => value.to_string(),
        ValueRef::Double(value) => value.to_string(),
        ValueRef::Decimal(value) => value.to_string(),
        ValueRef::Text(value) => String::from_utf8_lossy(value).into_owned(),
        ValueRef::Blob(value) => format!("<{} bytes>", value.len()),
        ValueRef::Geometry(value) => format!("<geometry: {} bytes>", value.len()),
        other => format!("{:?}", other.to_owned()),
    })
}

pub fn quote_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

pub fn quote_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_extensions_are_case_insensitive() {
        assert_eq!(
            DataSetFormat::from_path(Path::new("sample.CSV")).unwrap(),
            DataSetFormat::Csv
        );
        assert_eq!(
            DataSetFormat::from_path(Path::new("sample.jsonl")).unwrap(),
            DataSetFormat::Json
        );
        assert!(DataSetFormat::from_path(Path::new("sample.xlsx")).is_err());
    }

    #[test]
    fn sql_quoting_preserves_paths_and_identifiers() {
        assert_eq!(
            quote_literal("C:/O'Brien/file.csv"),
            "'C:/O''Brien/file.csv'"
        );
        assert_eq!(quote_identifier("order\"value"), "\"order\"\"value\"");
    }

    #[test]
    fn csv_fixture_loads_and_queries_with_duckdb() {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/sales.csv");
        let (data_set, sql, result) = load_and_preview(fixture).unwrap();

        assert_eq!(data_set.format, DataSetFormat::Csv);
        assert_eq!(
            data_set.columns,
            vec!["order_id", "customer", "region", "amount", "paid"]
        );
        assert!(sql.contains("LIMIT 200"));
        assert_eq!(result.rows.len(), 12);
    }

    #[test]
    fn json_fixture_uses_the_same_dataset_contract() {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/sales.json");
        let (data_set, _, result) = load_and_preview(fixture).unwrap();

        assert_eq!(data_set.format, DataSetFormat::Json);
        assert_eq!(data_set.alias, DATASET_ALIAS);
        assert_eq!(result.rows.len(), 3);
        assert_eq!(result.rows[0].cells[1], "示例甲公司");
    }

    #[test]
    fn filtering_and_sorting_keep_strong_row_ids() {
        let mut result = QueryResult {
            columns: vec!["name".to_string()],
            rows: vec![
                QueryRow {
                    id: 7,
                    cells: vec!["Beta".to_string()],
                },
                QueryRow {
                    id: 9,
                    cells: vec!["Alpha".to_string()],
                },
            ],
            elapsed_micros: 1,
            truncated: false,
        };

        result.sort_by_column(0, false);
        assert_eq!(result.rows[0].id, 9);
        assert_eq!(result.filtered_rows("bet")[0].id, 7);
    }
}
