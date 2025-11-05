//! Metrics collection, storage, and serving
//!
//! Parses METRIC lines from kernel output, stores in ring buffers with
//! downsampling (LTTB), and serves via REST API and WebSocket batching.

pub mod parser;
pub mod series;
pub mod store;

pub use parser::MetricParser;
pub use series::{MetricPoint, MetricSeries, SeriesStats};
pub use store::MetricsStore;
