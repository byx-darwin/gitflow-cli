//! 共享输出格式化模块。
//!
//! 提供统一的 [`print_output`] 函数，支持 JSON（默认）、Text、TOON 和 Auto 格式。
//! - JSON: 美化 JSON（默认，机器消费）
//! - Text: 人类友好的表格/键值对格式
//! - TOON: Token-Oriented Object Notation，紧凑格式（LLM 消费）
//! - Auto: 根据数据结构自动选择 JSON 或 TOON

use crate::OutputFormat;

/// 将可序列化的值按指定格式打印到 stdout。
///
/// # Errors
///
/// 返回错误当：
/// - JSON 序列化失败。
/// - Text/TOON 格式化时内部转换失败。
pub fn print_output<T: serde::Serialize>(value: &T, format: &OutputFormat) -> miette::Result<()> {
    match format {
        OutputFormat::Json => print_json(value),
        OutputFormat::Text => print_text(value),
        OutputFormat::Toon => print_toon(value),
        OutputFormat::Auto => print_auto(value),
    }
}

/// JSON 格式输出。
fn print_json<T: serde::Serialize>(value: &T) -> miette::Result<()> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| miette::miette!("Failed to serialize output to JSON: {e}"))?;
    println!("{json}");
    Ok(())
}

/// TOON 格式输出。
fn print_toon<T: serde::Serialize>(value: &T) -> miette::Result<()> {
    let json_value = serde_json::to_value(value)
        .map_err(|e| miette::miette!("Failed to serialize value for TOON output: {e}"))?;
    let toon = gitflow_cli_core::toon::encode(&json_value)
        .map_err(|e| miette::miette!("Failed to encode TOON output: {e}"))?;
    println!("{toon}");
    Ok(())
}

/// 自动选择格式输出。
///
/// 根据数据结构分析选择 JSON 或 TOON：
/// - 小数据 (< 200 字符) → JSON
/// - 均匀数组 (≥5 元素) → TOON
/// - 深度嵌套 (> 3 层) → TOON
/// - 其他 → JSON
fn print_auto<T: serde::Serialize>(value: &T) -> miette::Result<()> {
    let json_value = serde_json::to_value(value)
        .map_err(|e| miette::miette!("Failed to serialize value for auto output: {e}"))?;
    let shape = gitflow_cli_core::toon::analyze(&json_value);
    let strategy = gitflow_cli_core::toon::select_strategy(&shape);

    match strategy {
        gitflow_cli_core::toon::ToonStrategy::Toon => {
            let toon = gitflow_cli_core::toon::encode(&json_value)
                .map_err(|e| miette::miette!("Failed to encode TOON output: {e}"))?;
            println!("{toon}");
        }
        gitflow_cli_core::toon::ToonStrategy::Json => {
            let json = serde_json::to_string_pretty(&json_value)
                .map_err(|e| miette::miette!("Failed to serialize output to JSON: {e}"))?;
            println!("{json}");
        }
    }
    Ok(())
}

/// Text 格式输出。先将值序列化为 [`serde_json::Value`]，再根据结构选择格式化策略。
fn print_text<T: serde::Serialize>(value: &T) -> miette::Result<()> {
    let v = serde_json::to_value(value)
        .map_err(|e| miette::miette!("Failed to serialize value for text output: {e}"))?;
    match &v {
        serde_json::Value::Array(arr) => {
            print_text_array(arr);
        }
        serde_json::Value::Object(obj) => {
            print_text_object(obj);
        }
        serde_json::Value::Null => {
            println!("(空)");
        }
        other => {
            // 标量值直接打印
            println!("{other}");
        }
    }
    Ok(())
}

/// 数组 → 表格格式。
///
/// 自动检测数组中每个元素的公共字段作为列名。
/// 空数组打印 "(空)"。
fn print_text_array(arr: &[serde_json::Value]) {
    if arr.is_empty() {
        println!("(空)");
        return;
    }

    // 收集所有可能的列名（按首次出现顺序）
    let mut columns: Vec<String> = Vec::new();
    for item in arr {
        if let serde_json::Value::Object(obj) = item {
            for key in obj.keys() {
                let col = key.clone();
                if !columns.contains(&col) {
                    columns.push(col);
                }
            }
        }
    }

    if columns.is_empty() {
        // 数组元素不是对象（如字符串数组），按列表打印
        for (i, item) in arr.iter().enumerate() {
            println!("  {}. {}", i + 1, format_value(item));
        }
        return;
    }

    // 计算列宽
    let mut col_widths: Vec<usize> = columns.iter().map(String::len).collect();
    for item in arr {
        if let serde_json::Value::Object(obj) = item {
            for (ci, col) in columns.iter().enumerate() {
                if let Some(val) = obj.get(col) {
                    let display = format_value(val);
                    if let Some(w) = col_widths.get_mut(ci) {
                        *w = (*w).max(display.chars().count());
                    }
                }
            }
        }
    }

    // 打印表头
    let header: Vec<String> = columns
        .iter()
        .enumerate()
        .map(|(i, c)| pad_right(c, *col_widths.get(i).unwrap_or(&0)))
        .collect();
    println!("{}", header.join(" │ "));

    // 打印分隔线
    let sep: Vec<String> = col_widths.iter().map(|w| "─".repeat(*w)).collect();
    println!("{}", sep.join("─┼─"));

    // 打印数据行
    for item in arr {
        let row: Vec<String> = columns
            .iter()
            .enumerate()
            .map(|(ci, col)| {
                if let serde_json::Value::Object(obj) = item {
                    let val = obj.get(col).map_or(String::new(), format_value);
                    pad_right(&val, *col_widths.get(ci).unwrap_or(&0))
                } else {
                    String::new()
                }
            })
            .collect();
        println!("{}", row.join(" │ "));
    }
}

/// 对象 → 键值对格式。
fn print_text_object(obj: &serde_json::Map<String, serde_json::Value>) {
    if obj.is_empty() {
        println!("(空)");
        return;
    }

    // 计算最大键宽
    let max_key_len = obj.keys().map(|k| k.chars().count()).max().unwrap_or(0);

    for (key, val) in obj {
        let display = format_value(val);
        if display.contains('\n') {
            // 多行值：键单独一行，值缩进
            println!("{key}:");
            for line in display.lines() {
                println!("  {line}");
            }
        } else {
            println!("  {key:>max_key_len$}: {display}");
        }
    }
}

/// 将 [`serde_json::Value`] 转为人类友好的字符串。
fn format_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => String::from("-"),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(format_value).collect();
            items.join(", ")
        }
        serde_json::Value::Object(_) => String::from("{…}"),
    }
}

/// 右填充字符串到指定宽度（按字符数，非字节数）。
fn pad_right(s: &str, width: usize) -> String {
    let char_count = s.chars().count();
    if char_count >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - char_count))
    }
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "允许在测试中使用 expect/unwrap/panic"
)]
mod tests {
    use serde::Serialize;

    use super::*;

    #[derive(Serialize)]
    struct TestIssue {
        number: u64,
        title: String,
        state: String,
    }

    #[derive(Serialize)]
    struct TestPr {
        number: u64,
        title: String,
        state: String,
        branch: String,
        draft: bool,
    }

    #[test]
    fn test_should_output_json_format() {
        let issues = vec![TestIssue {
            number: 1,
            title: String::from("测试标题"),
            state: String::from("open"),
        }];
        let result = print_output(&issues, &OutputFormat::Json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_output_text_format_for_array() {
        let issues = vec![
            TestIssue {
                number: 1,
                title: String::from("修复登录 bug"),
                state: String::from("open"),
            },
            TestIssue {
                number: 2,
                title: String::from("添加新功能"),
                state: String::from("closed"),
            },
        ];
        let result = print_output(&issues, &OutputFormat::Text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_output_text_format_for_object() {
        let pr = TestPr {
            number: 42,
            title: String::from("重构模块"),
            state: String::from("open"),
            branch: String::from("feat/refactor"),
            draft: false,
        };
        let result = print_output(&pr, &OutputFormat::Text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_handle_empty_array() {
        let empty: Vec<TestIssue> = vec![];
        let result = print_output(&empty, &OutputFormat::Text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_handle_null_value() {
        let null_val: Option<String> = None;
        let result = print_output(&null_val, &OutputFormat::Text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_value_null() {
        assert_eq!(format_value(&serde_json::Value::Null), "-");
    }

    #[test]
    fn test_format_value_bool() {
        assert_eq!(format_value(&serde_json::Value::Bool(true)), "true");
    }

    #[test]
    fn test_format_value_number() {
        assert_eq!(format_value(&serde_json::json!(42)), "42");
    }

    #[test]
    fn test_format_value_string() {
        assert_eq!(
            format_value(&serde_json::Value::String(String::from("hello"))),
            "hello"
        );
    }

    #[test]
    fn test_format_value_array() {
        let arr = serde_json::json!(["bug", "enhancement"]);
        assert_eq!(format_value(&arr), "bug, enhancement");
    }

    #[test]
    fn test_pad_right_pads() {
        assert_eq!(pad_right("hi", 5), "hi   ");
    }

    #[test]
    fn test_pad_right_no_truncation() {
        assert_eq!(pad_right("hello", 3), "hello");
    }

    #[test]
    fn test_should_output_toon_format() {
        let issues = vec![TestIssue {
            number: 1,
            title: String::from("test"),
            state: String::from("open"),
        }];
        let result = print_output(&issues, &OutputFormat::Toon);
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_output_auto_format() {
        let issues = vec![TestIssue {
            number: 1,
            title: String::from("test"),
            state: String::from("open"),
        }];
        let result = print_output(&issues, &OutputFormat::Auto);
        assert!(result.is_ok());
    }
}
