//! A lightweight, regex-free parser for standard G-code.

#[derive(Debug, Clone, PartialEq)]
pub enum GCodeCommand {
    /// Linear movement command (G0, G1).
    Move {
        x: Option<f32>,
        y: Option<f32>,
        z: Option<f32>,
        a: Option<f32>,
        c: Option<f32>,
        e: Option<f32>,
        f: Option<f32>,
    },
    /// Set absolute positioning (G90).
    SetAbsolutePositioning,
    /// Set relative positioning (G91).
    SetRelativePositioning,
    /// Any other command or empty line.
    Other,
}

/// Parses a single line of G-code into a `GCodeCommand`.
pub fn parse_line(line: &str) -> GCodeCommand {
    // 1. Strip comments (everything after ';')
    let line_no_comment = line.split(';').next().unwrap_or("").trim();
    if line_no_comment.is_empty() {
        return GCodeCommand::Other;
    }

    // 2. Split by whitespace
    let mut tokens = line_no_comment.split_whitespace();
    let command_token = match tokens.next() {
        Some(token) => token.to_uppercase(),
        None => return GCodeCommand::Other,
    };

    // 3. Match command type
    match command_token.as_str() {
        "G0" | "G1" => {
            let mut x = None;
            let mut y = None;
            let mut z = None;
            let mut a = None;
            let mut c = None;
            let mut e = None;
            let mut f = None;

            for token in tokens {
                let token = token.to_uppercase();
                if token.is_empty() {
                    continue;
                }
                
                let (axis, val_str) = token.split_at(1);
                if let Ok(value) = val_str.parse::<f32>() {
                    match axis {
                        "X" => x = Some(value),
                        "Y" => y = Some(value),
                        "Z" => z = Some(value),
                        "A" => a = Some(value),
                        "C" => c = Some(value),
                        "E" => e = Some(value),
                        "F" => f = Some(value),
                        _ => {}
                    }
                }
            }

            GCodeCommand::Move { x, y, z, a, c, e, f }
        }
        "G90" => GCodeCommand::SetAbsolutePositioning,
        "G91" => GCodeCommand::SetRelativePositioning,
        _ => GCodeCommand::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_move() {
        let cmd = parse_line("G1 X10.5 Y-5.0 Z2.2 E5.0 F3000.0 ; comment here");
        assert_eq!(
            cmd,
            GCodeCommand::Move {
                x: Some(10.5),
                y: Some(-5.0),
                z: Some(2.2),
                a: None,
                c: None,
                e: Some(5.0),
                f: Some(3000.0),
            }
        );
    }

    #[test]
    fn test_parse_g90_g91() {
        assert_eq!(parse_line("G90"), GCodeCommand::SetAbsolutePositioning);
        assert_eq!(parse_line("G91"), GCodeCommand::SetRelativePositioning);
    }

    #[test]
    fn test_parse_other_and_empty() {
        assert_eq!(parse_line("M104 S200"), GCodeCommand::Other);
        assert_eq!(parse_line("; just a comment"), GCodeCommand::Other);
        assert_eq!(parse_line("   "), GCodeCommand::Other);
    }
}
