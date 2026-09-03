use crate::log_buffer::LogBuffer;

pub fn log_lines(buffer: &LogBuffer) -> Vec<(String, i32)> {
    buffer
        .snapshot()
        .iter()
        .map(|entry| (entry.line(), entry.level.as_int()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log_buffer::{LogEntry, LogLevel};

    #[test]
    fn formats_line_and_level() {
        let buffer = LogBuffer::new();
        buffer.push(LogEntry {
            timestamp: "12:01:33".to_string(),
            level: LogLevel::Warn,
            target: "poketto_core::vndb".to_string(),
            message: "slow".to_string(),
        });
        assert_eq!(
            log_lines(&buffer),
            vec![(
                "12:01:33 [WARN] poketto_core::vndb: slow".to_string(),
                1
            )]
        );
    }
}
