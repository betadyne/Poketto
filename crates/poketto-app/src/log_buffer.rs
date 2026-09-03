use std::collections::VecDeque;
use std::fmt;
use std::sync::Arc;

use parking_lot::Mutex;
use tracing::field::{Field, Visit};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;

pub const LOG_CAPACITY: usize = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn tag(self) -> &'static str {
        match self {
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }

    pub fn as_int(self) -> i32 {
        match self {
            LogLevel::Info => 0,
            LogLevel::Warn => 1,
            LogLevel::Error => 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub target: String,
    pub message: String,
}

impl LogEntry {
    pub fn line(&self) -> String {
        format!(
            "{} [{}] {}: {}",
            self.timestamp,
            self.level.tag(),
            self.target,
            self.message
        )
    }
}

#[derive(Debug, Default)]
struct Ring {
    entries: VecDeque<LogEntry>,
    generation: u64,
}

#[derive(Debug, Clone, Default)]
pub struct LogBuffer {
    inner: Arc<Mutex<Ring>>,
}

impl LogBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&self, entry: LogEntry) {
        let mut ring = self.inner.lock();
        while ring.entries.len() >= LOG_CAPACITY {
            ring.entries.pop_front();
        }
        ring.entries.push_back(entry);
        ring.generation += 1;
    }

    pub fn snapshot(&self) -> Vec<LogEntry> {
        self.inner.lock().entries.iter().cloned().collect()
    }

    pub fn clear(&self) {
        let mut ring = self.inner.lock();
        ring.entries.clear();
        ring.generation += 1;
    }

    pub fn generation(&self) -> u64 {
        self.inner.lock().generation
    }

    pub fn export_text(&self) -> String {
        self.snapshot()
            .iter()
            .map(LogEntry::line)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

pub struct LogBufferLayer {
    buffer: LogBuffer,
}

impl LogBufferLayer {
    pub fn new(buffer: LogBuffer) -> Self {
        Self { buffer }
    }
}

struct MessageVisitor(Option<String>);

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            self.0 = Some(format!("{value:?}"));
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.0 = Some(value.to_string());
        }
    }
}

impl<S> Layer<S> for LogBufferLayer
where
    S: tracing::Subscriber,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = MessageVisitor(None);
        event.record(&mut visitor);
        let level = match *event.metadata().level() {
            tracing::Level::ERROR => LogLevel::Error,
            tracing::Level::WARN => LogLevel::Warn,
            _ => LogLevel::Info,
        };
        self.buffer.push(LogEntry {
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            level,
            target: event.metadata().target().to_string(),
            message: visitor.0.unwrap_or_default(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber::prelude::*;

    fn entry(n: u64) -> LogEntry {
        LogEntry {
            timestamp: format!("t{n}"),
            level: LogLevel::Info,
            target: "test".to_string(),
            message: format!("msg{n}"),
        }
    }

    #[test]
    fn evicts_oldest_beyond_capacity() {
        let buffer = LogBuffer::new();
        for n in 0..(LOG_CAPACITY as u64 + 2) {
            buffer.push(entry(n));
        }
        let snapshot = buffer.snapshot();
        assert_eq!(snapshot.len(), LOG_CAPACITY);
        assert_eq!(snapshot[0].message, "msg2");
        assert_eq!(buffer.generation(), LOG_CAPACITY as u64 + 2);
    }

    #[test]
    fn clear_empties_and_bumps_generation() {
        let buffer = LogBuffer::new();
        buffer.push(entry(0));
        let before = buffer.generation();
        buffer.clear();
        assert!(buffer.snapshot().is_empty());
        assert_eq!(buffer.generation(), before + 1);
    }

    #[test]
    fn layer_maps_levels_and_messages() {
        let buffer = LogBuffer::new();
        let layer = LogBufferLayer::new(buffer.clone());
        let _guard = tracing_subscriber::registry().with(layer).set_default();
        tracing::info!(target: "logs_test", "hello info");
        tracing::warn!(target: "logs_test", "hello warn");
        tracing::error!(target: "logs_test", "hello error");
        tracing::debug!(target: "logs_test", "hello debug");
        let snapshot = buffer.snapshot();
        assert_eq!(snapshot.len(), 4);
        assert_eq!(
            snapshot
                .iter()
                .map(|e| e.level)
                .collect::<Vec<_>>(),
            vec![LogLevel::Info, LogLevel::Warn, LogLevel::Error, LogLevel::Info]
        );
        assert_eq!(snapshot[0].message, "hello info");
        assert_eq!(snapshot[1].message, "hello warn");
        assert_eq!(snapshot[2].message, "hello error");
        for e in &snapshot {
            assert!(!e.target.is_empty());
            assert!(!e.timestamp.is_empty());
        }
    }
}
