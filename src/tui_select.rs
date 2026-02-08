use std::io::{self, BufRead, Write};

pub fn select_bin<R: BufRead, W: Write>(
    bin_names: &[String],
    reader: &mut R,
    writer: &mut W,
) -> io::Result<String> {
    if bin_names.len() == 1 {
        return Ok(bin_names[0].clone());
    }

    if bin_names.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "no binaries available",
        ));
    }

    writeln!(writer, "Select a binary:")?;
    for (idx, name) in bin_names.iter().enumerate() {
        writeln!(writer, "  {}) {}", idx + 1, name)?;
    }

    loop {
        write!(writer, "Enter choice (1-{}): ", bin_names.len())?;
        writer.flush()?;

        let mut input = String::new();
        if reader.read_line(&mut input)? == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "no selection provided",
            ));
        }

        let trimmed = input.trim();
        if let Ok(choice) = trimmed.parse::<usize>() {
            if (1..=bin_names.len()).contains(&choice) {
                return Ok(bin_names[choice - 1].clone());
            }
        }

        writeln!(writer, "Invalid selection. Try again.")?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn select_bin_returns_single_entry() {
        let mut input = Cursor::new("");
        let mut output = Vec::new();
        let bins = vec!["demo".to_string()];
        let selected = select_bin(&bins, &mut input, &mut output).expect("select bin");
        assert_eq!(selected, "demo");
        assert!(output.is_empty());
    }

    #[test]
    fn select_bin_prompts_and_selects_choice() {
        let mut input = Cursor::new("2\n");
        let mut output = Vec::new();
        let bins = vec!["alpha".to_string(), "beta".to_string()];
        let selected = select_bin(&bins, &mut input, &mut output).expect("select bin");
        assert_eq!(selected, "beta");
        let output_str = String::from_utf8(output).expect("utf8");
        assert!(output_str.contains("Select a binary:"));
        assert!(output_str.contains("1) alpha"));
        assert!(output_str.contains("2) beta"));
    }

    #[test]
    fn select_bin_reprompts_on_invalid_input() {
        let mut input = Cursor::new("foo\n1\n");
        let mut output = Vec::new();
        let bins = vec!["alpha".to_string(), "beta".to_string()];
        let selected = select_bin(&bins, &mut input, &mut output).expect("select bin");
        assert_eq!(selected, "alpha");
        let output_str = String::from_utf8(output).expect("utf8");
        assert!(output_str.contains("Invalid selection"));
    }

    #[test]
    fn select_bin_errors_on_empty_list() {
        let mut input = Cursor::new("");
        let mut output = Vec::new();
        let bins = Vec::<String>::new();
        let err = select_bin(&bins, &mut input, &mut output).expect_err("expected error");
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }
}
