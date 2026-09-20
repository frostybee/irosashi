use crate::types::{LineMarker, LineRange, MarkerType};

pub fn process_diff_block(code: &str) -> (String, Vec<LineMarker>) {
    let lines: Vec<&str> = code.split('\n').collect();
    let mut stripped = Vec::with_capacity(lines.len());
    let mut ins_lines = Vec::new();
    let mut del_lines = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;
        if line.is_empty() {
            stripped.push(String::new());
            continue;
        }
        match line.as_bytes()[0] {
            b'+' => {
                stripped.push(strip_diff_prefix(line));
                ins_lines.push(LineRange::single(line_num));
            }
            b'-' => {
                stripped.push(strip_diff_prefix(line));
                del_lines.push(LineRange::single(line_num));
            }
            b' ' => {
                stripped.push(line[1..].to_owned());
            }
            _ => {
                stripped.push((*line).to_owned());
            }
        }
    }

    let mut markers = Vec::new();
    if !ins_lines.is_empty() {
        markers.push(LineMarker {
            marker_type: MarkerType::Ins,
            lines: ins_lines,
            label: String::new(),
        });
    }
    if !del_lines.is_empty() {
        markers.push(LineMarker {
            marker_type: MarkerType::Del,
            lines: del_lines,
            label: String::new(),
        });
    }

    (stripped.join("\n"), markers)
}

/// The code without the lines carrying a `Del` marker: what a reader wants on the
/// clipboard is the result of the change.
pub fn drop_deleted_lines(code: &str, markers: &[LineMarker]) -> String {
    let deleted: Vec<&LineRange> = markers
        .iter()
        .filter(|m| m.marker_type == MarkerType::Del)
        .flat_map(|m| &m.lines)
        .collect();
    if deleted.is_empty() {
        return code.to_owned();
    }
    code.split('\n')
        .enumerate()
        .filter(|(i, _)| !deleted.iter().any(|r| r.contains(i + 1)))
        .map(|(_, line)| line)
        .collect::<Vec<_>>()
        .join("\n")
}

fn strip_diff_prefix(line: &str) -> String {
    if line.len() <= 1 {
        return String::new();
    }
    if line.as_bytes()[1] == b' ' {
        line[2..].to_owned()
    } else {
        line[1..].to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_prefixes_and_generates_markers() {
        let code = "+ added\n- removed\n context\n more context";
        let (stripped, markers) = process_diff_block(code);
        assert_eq!(stripped, "added\nremoved\ncontext\nmore context");
        assert_eq!(markers.len(), 2);
        assert_eq!(markers[0].marker_type, MarkerType::Ins);
        assert_eq!(markers[0].lines, [LineRange::single(1)]);
        assert_eq!(markers[1].marker_type, MarkerType::Del);
        assert_eq!(markers[1].lines, [LineRange::single(2)]);
    }

    #[test]
    fn empty_lines_preserved() {
        let code = "+ a\n\n- b";
        let (stripped, markers) = process_diff_block(code);
        assert_eq!(stripped, "a\n\nb");
        assert_eq!(markers.len(), 2);
    }

    #[test]
    fn no_prefix_lines_kept_as_is() {
        let code = "header\n+ added";
        let (stripped, markers) = process_diff_block(code);
        assert_eq!(stripped, "header\nadded");
        assert_eq!(markers.len(), 1);
        assert_eq!(markers[0].lines, [LineRange::single(2)]);
    }

    #[test]
    fn prefix_without_space_strips_only_prefix_char() {
        let code = "+compact\n-also";
        let (stripped, _) = process_diff_block(code);
        assert_eq!(stripped, "compact\nalso");
    }

    #[test]
    fn single_char_prefix_produces_empty_string() {
        let code = "+";
        let (stripped, markers) = process_diff_block(code);
        assert_eq!(stripped, "");
        assert_eq!(markers[0].marker_type, MarkerType::Ins);
    }

    #[test]
    fn drop_deleted_lines_keeps_everything_else() {
        let (stripped, markers) = process_diff_block("+ a\n- b\n c\n- d");
        assert_eq!(drop_deleted_lines(&stripped, &markers), "a\nc");
        assert_eq!(drop_deleted_lines("x\ny", &[]), "x\ny");
    }

    #[test]
    fn no_diff_markers_returns_empty_markers() {
        let code = " plain\n text";
        let (stripped, markers) = process_diff_block(code);
        assert_eq!(stripped, "plain\ntext");
        assert!(markers.is_empty());
    }
}
