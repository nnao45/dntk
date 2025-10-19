use super::error::BcError;

impl super::BcExecuter {
    pub(super) fn parse_branch<'a>(
        &self,
        input: &'a str,
    ) -> Result<(Vec<&'a str>, &'a str), BcError> {
        let trimmed = input.trim_start();
        if trimmed.starts_with('{') {
            let end = Self::find_matching(trimmed, 0, '{', '}')?;
            let body = &trimmed[1..end];
            let remainder = &trimmed[end + 1..];
            let statements = self.split_statements(body);
            Ok((statements, remainder))
        } else {
            let chars: Vec<char> = trimmed.chars().collect();
            let mut idx = 0usize;
            let mut depth_round = 0;
            let mut depth_square = 0;
            let mut depth_curly = 0;

            while idx < chars.len() {
                match chars[idx] {
                    '(' => depth_round += 1,
                    ')' => {
                        if depth_round > 0 {
                            depth_round -= 1;
                        }
                    }
                    '[' => depth_square += 1,
                    ']' => {
                        if depth_square > 0 {
                            depth_square -= 1;
                        }
                    }
                    '{' => depth_curly += 1,
                    '}' => {
                        if depth_curly > 0 {
                            depth_curly -= 1;
                        }
                    }
                    'e' | 'E' if depth_round == 0 && depth_square == 0 && depth_curly == 0 => {
                        if trimmed[idx..].starts_with("else")
                            && Self::is_keyword_boundary(trimmed, idx, idx + 4)
                        {
                            break;
                        }
                    }
                    ';' if depth_round == 0 && depth_square == 0 && depth_curly == 0 => {
                        idx += 1;
                        break;
                    }
                    _ => {}
                }
                idx += 1;
            }

            let statement = trimmed[..idx].trim();
            let remainder = trimmed[idx..].trim_start();
            let statements = if statement.is_empty() {
                Vec::new()
            } else {
                vec![statement]
            };
            Ok((statements, remainder))
        }
    }

    pub(super) fn split_statements<'a>(&self, input: &'a str) -> Vec<&'a str> {
        let mut statements = Vec::new();
        let mut depth_round = 0;
        let mut depth_square = 0;
        let mut depth_curly = 0;

        let mut start = 0;
        for (idx, ch) in input.char_indices() {
            match ch {
                '(' => depth_round += 1,
                ')' => {
                    if depth_round > 0 {
                        depth_round -= 1;
                    }
                }
                '[' => depth_square += 1,
                ']' => {
                    if depth_square > 0 {
                        depth_square -= 1;
                    }
                }
                '{' => depth_curly += 1,
                '}' => {
                    if depth_curly > 0 {
                        depth_curly -= 1;
                    }
                }
                ';' | '\n' if depth_round == 0 && depth_square == 0 && depth_curly == 0 => {
                    let trimmed = input[start..idx].trim();
                    if !trimmed.is_empty() {
                        statements.push(trimmed);
                    }
                    start = idx + ch.len_utf8();
                    continue;
                }
                _ => {}
            }
        }

        let trimmed = input[start..].trim();
        if !trimmed.is_empty() {
            statements.push(trimmed);
        }

        statements
    }

    pub(super) fn detect_assignment(stmt: &str) -> Option<(&str, &str)> {
        let mut depth_round = 0;
        let mut depth_square = 0;
        let mut depth_curly = 0;
        let mut prev_char: Option<char> = None;

        for (index, ch) in stmt.char_indices() {
            match ch {
                '(' => depth_round += 1,
                ')' => {
                    if depth_round > 0 {
                        depth_round -= 1;
                    }
                }
                '[' => depth_square += 1,
                ']' => {
                    if depth_square > 0 {
                        depth_square -= 1;
                    }
                }
                '{' => depth_curly += 1,
                '}' => {
                    if depth_curly > 0 {
                        depth_curly -= 1;
                    }
                }
                '=' if depth_round == 0 && depth_square == 0 && depth_curly == 0 => {
                    if matches!(prev_char, Some('<') | Some('>') | Some('!')) {
                        prev_char = Some('=');
                        continue;
                    }
                    let rest = &stmt[index + ch.len_utf8()..];
                    if rest.starts_with('=') {
                        prev_char = Some('=');
                        continue;
                    }

                    let left = stmt[..index].trim();
                    let right = rest.trim();
                    if left.is_empty() || right.is_empty() {
                        return None;
                    }
                    return Some((left, right));
                }
                _ => {}
            }
            prev_char = Some(ch);
        }
        None
    }

    pub(super) fn split_top_level(input: &str, delimiter: char) -> Vec<&str> {
        let mut parts = Vec::new();
        let mut depth_round = 0;
        let mut depth_square = 0;
        let mut depth_curly = 0;

        let mut start = 0;
        for (idx, ch) in input.char_indices() {
            match ch {
                '(' => depth_round += 1,
                ')' => {
                    if depth_round > 0 {
                        depth_round -= 1;
                    }
                }
                '[' => depth_square += 1,
                ']' => {
                    if depth_square > 0 {
                        depth_square -= 1;
                    }
                }
                '{' => depth_curly += 1,
                '}' => {
                    if depth_curly > 0 {
                        depth_curly -= 1;
                    }
                }
                _ => {}
            }

            if ch == delimiter && depth_round == 0 && depth_square == 0 && depth_curly == 0 {
                let segment = &input[start..idx];
                parts.push(segment.trim());
                start = idx + ch.len_utf8();
            }
        }

        let tail = input[start..].trim();
        if !tail.is_empty() {
            parts.push(tail);
        }

        parts
    }

    pub(super) fn lookup_keyword(bytes: &str, expected: &str) -> bool {
        bytes.len() >= expected.len()
            && bytes[..expected.len()].eq_ignore_ascii_case(expected)
            && Self::is_keyword_boundary(bytes, 0, expected.len())
    }

    pub(super) fn starts_with_keyword(input: &str, keyword: &str) -> bool {
        let trimmed = input.trim_start();
        Self::lookup_keyword(trimmed, keyword)
    }

    pub(super) fn is_keyword_boundary(input: &str, start: usize, end: usize) -> bool {
        let bytes = input.as_bytes();
        let before = start.checked_sub(1).and_then(|idx| bytes.get(idx));
        let after = bytes.get(end);

        let prev_ok = before.is_none_or(|c| !Self::is_ident_char(*c));
        let next_ok = after.is_none_or(|c| !Self::is_ident_char(*c));
        prev_ok && next_ok
    }

    pub(super) fn find_matching(
        input: &str,
        start: usize,
        open: char,
        close: char,
    ) -> Result<usize, BcError> {
        let mut depth = 0;
        for (index, ch) in input.char_indices().skip(start) {
            if ch == open {
                depth += 1;
            } else if ch == close {
                depth -= 1;
                if depth == 0 {
                    return Ok(index);
                }
            }
        }
        Err(BcError::Error("Unmatched delimiter".to_string()))
    }

    pub(super) fn preprocess_bc_syntax(&self, statement: &str) -> String {
        let bytes = statement.as_bytes();
        let mut result = String::with_capacity(statement.len());
        let mut i = 0;
        while i < bytes.len() {
            if i + 1 < bytes.len() && bytes[i + 1] == b'(' && !Self::has_ident_before(bytes, i) {
                let replacement = match bytes[i] {
                    b's' => Some("sin("),
                    b'c' => Some("cos("),
                    b'a' => Some("atan("),
                    b'l' => Some("ln("),
                    b'e' => Some("exp("),
                    _ => None,
                };
                if let Some(rep) = replacement {
                    result.push_str(rep);
                    i += 2;
                    continue;
                }
            }
            result.push(bytes[i] as char);
            i += 1;
        }
        result
    }

    pub(super) fn is_valid_identifier(name: &str) -> bool {
        let mut chars = name.chars();
        match chars.next() {
            Some(c) if c.is_ascii_alphabetic() || c == '_' => (),
            _ => return false,
        }
        chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
    }

    pub(super) fn is_ident_char(byte: u8) -> bool {
        byte.is_ascii_alphanumeric() || byte == b'_'
    }

    pub(super) fn has_ident_before(bytes: &[u8], idx: usize) -> bool {
        if idx == 0 {
            return false;
        }
        Self::is_ident_char(bytes[idx - 1])
    }
}
