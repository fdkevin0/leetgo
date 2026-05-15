use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub fn deserialize<'de, T: Deserialize<'de>>(s: &'de str) -> Result<T> {
    let res: T = serde_json::from_str(s)?;
    Ok(res)
}

pub fn serialize<T: Serialize>(v: T) -> Result<String> {
    let res = serde_json::to_string(&v)?;
    Ok(res)
}

pub fn split_array(raw: &str) -> Result<Vec<String>> {
    let trimmed = raw.trim();

    if trimmed.len() <= 1 || !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        bail!("invalid array: {}", trimmed);
    }

    let splits: Vec<Value> = serde_json::from_str(trimmed)?;
    let res: Vec<String> = splits.iter().map(|v| v.to_string()).collect();
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_array_valid_inputs() {
        let test_cases = [
            ("[]", vec![]),
            ("[1]", vec!["1"]),
            (r#"["a", "b"]"#, vec![r#""a""#, r#""b""#]),
            ("[1, 2, 3]", vec!["1", "2", "3"]),
            (
                r#"[1, "a", null, true, false]"#,
                vec!["1", r#""a""#, "null", "true", "false"],
            ),
            ("[1, [2, 3], 4]", vec!["1", "[2,3]", "4"]),
            (
                r#"[{"nums":[1, null]}, "a,b", true]"#,
                vec![r#"{"nums":[1,null]}"#, r#""a,b""#, "true"],
            ),
            ("   [1, 2]  ", vec!["1", "2"]),
        ];

        for (input, expected) in test_cases {
            let result = split_array(input).unwrap_or_else(|err| {
                panic!("split_array failed for input {input:?}: {err}");
            });
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_split_array_invalid_inputs() {
        for input in ["", "1", "1,2", "[1", "1]", "[1,]"] {
            assert!(split_array(input).is_err(), "input should fail: {input:?}");
        }
    }

    #[test]
    fn test_deserialize_array_with_nulls() {
        let values: Vec<Option<i32>> = deserialize("[1,null,3]").unwrap();
        assert_eq!(values, vec![Some(1), None, Some(3)]);
    }

    #[test]
    fn test_deserialize_invalid_json_returns_error() {
        let values = deserialize::<Vec<i32>>("[1,]");
        assert!(values.is_err());
    }

    #[test]
    fn test_serialize_array_with_nulls() {
        let values = vec![Some(1), None, Some(3)];
        assert_eq!(serialize(values).unwrap(), "[1,null,3]");
    }

    #[test]
    fn test_serialize_escapes_strings() {
        let values = vec!["a,b", r#"quote""#];
        assert_eq!(serialize(values).unwrap(), r#"["a,b","quote\""]"#);
    }
}
