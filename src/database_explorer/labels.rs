use crate::database::{ConstraintType, TableColumn, TableConstraint, TableIndex};

pub(super) fn column_label(column: &TableColumn) -> String {
    let mut data_type = column.data_type.clone();
    if let Some(maximum_length) = column.character_maximum_length {
        data_type.push_str(&format!("({maximum_length})"));
    }
    let mut attributes = Vec::new();
    if !column.is_nullable {
        attributes.push("not null".to_string());
    }
    if column.is_auto_increment {
        attributes.push("auto increment".to_string());
    }
    if let Some(default) = &column.default {
        attributes.push(format!("default {default}"));
    }
    let suffix = if attributes.is_empty() {
        String::new()
    } else {
        format!(" ({})", attributes.join(", "))
    };
    format!("{}: {data_type}{suffix}", column.name)
}

pub(super) fn constraint_label(constraint: &TableConstraint) -> String {
    let constraint_type = match &constraint.constraint_type {
        ConstraintType::ForeignKey => "foreign key",
        ConstraintType::Unique => "unique",
        ConstraintType::PrimaryKey => "primary key",
        ConstraintType::Check => "check",
        ConstraintType::Other(value) => value,
    };
    let mut attributes = vec![constraint_type.to_string()];
    if constraint.is_deferrable {
        attributes.push("deferrable".to_string());
    }
    if constraint.nulls_distinct == Some(false) {
        attributes.push("nulls not distinct".to_string());
    }
    format!("{} ({})", constraint.name, attributes.join(", "))
}

pub(super) fn index_label(index: &TableIndex) -> String {
    let mut attributes = Vec::new();
    if index.is_primary {
        attributes.push("primary");
    } else if index.is_unique {
        attributes.push("unique");
    }
    if attributes.is_empty() {
        format!("{}: {}", index.name, index.definition)
    } else {
        format!(
            "{} ({}): {}",
            index.name,
            attributes.join(", "),
            index.definition
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_labels_preserve_type_and_attributes() {
        let column = TableColumn {
            name: "name".to_owned(),
            default: Some("'anonymous'".to_owned()),
            is_nullable: false,
            data_type: "varchar".to_owned(),
            character_maximum_length: Some(80),
            is_auto_increment: false,
        };
        assert_eq!(
            column_label(&column),
            "name: varchar(80) (not null, default 'anonymous')"
        );
    }
}
