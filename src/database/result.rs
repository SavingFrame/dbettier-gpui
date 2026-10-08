use std::{fmt, time::Duration};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum CellValue {
    Null,
    Boolean(bool),
    Integer(i64),
    UnsignedInteger(u64),
    Float(f64),
    // Preserve exact decimal precision rather than converting to floating point.
    Decimal(String),
    Uuid(uuid::Uuid),
    Text(String),
    Bytes(Vec<u8>),
    // Preserve database precision and timezone information in temporal values.
    Date(String),
    Time(String),
    Timestamp(String),
    Json(serde_json::Value),
}

impl fmt::Display for CellValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => formatter.write_str("NULL"),
            Self::Boolean(value) => write!(formatter, "{value}"),
            Self::Integer(value) => write!(formatter, "{value}"),
            Self::UnsignedInteger(value) => write!(formatter, "{value}"),
            Self::Float(value) => write!(formatter, "{value}"),
            Self::Uuid(value) => write!(formatter, "{value}"),
            Self::Decimal(value)
            | Self::Text(value)
            | Self::Date(value)
            | Self::Time(value)
            | Self::Timestamp(value) => formatter.write_str(value),
            Self::Bytes(bytes) => {
                formatter.write_str("\\x")?;
                for byte in bytes {
                    write!(formatter, "{byte:02x}")?;
                }
                Ok(())
            }
            Self::Json(value) => write!(formatter, "{value}"),
        }
    }
}

#[derive(Clone)]
pub(crate) struct ResultColumn {
    pub(crate) name: String,
    pub(crate) column_type: String,
}

pub(crate) struct QueryOutput {
    pub(crate) columns: Vec<ResultColumn>,
    pub(crate) rows: Vec<Vec<CellValue>>,
    pub(crate) rows_affected: u64,
    pub(crate) elapsed: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cells_display_without_losing_text_precision() {
        let cases = [
            (CellValue::Null, "NULL"),
            (CellValue::Boolean(true), "true"),
            (CellValue::Integer(-42), "-42"),
            (CellValue::UnsignedInteger(u64::MAX), "18446744073709551615"),
            (CellValue::Float(1.5), "1.5"),
            (CellValue::Decimal("123.4500".to_owned()), "123.4500"),
            (CellValue::Text(String::new()), ""),
            (CellValue::Bytes(vec![0, 15, 255]), "\\x000fff"),
            (CellValue::Bytes(Vec::new()), "\\x"),
            (CellValue::Date("2026-01-01".to_owned()), "2026-01-01"),
            (CellValue::Time("12:30:00".to_owned()), "12:30:00"),
            (
                CellValue::Timestamp("2026-01-01T12:30:00+02:00".to_owned()),
                "2026-01-01T12:30:00+02:00",
            ),
            (CellValue::Json(serde_json::json!({"id": 1})), "{\"id\":1}"),
        ];

        for (value, expected) in cases {
            assert_eq!(value.to_string(), expected);
        }
    }
}
