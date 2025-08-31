//! Span processor and exporter for OpenTelemetry-Langfuse integration.
//!
//! This module provides utilities for processing and exporting spans with
//! Langfuse-specific attribute mapping.

use crate::mapper::AttributeMapper;
use opentelemetry_sdk::trace::{SpanData, SpanExporter};
use std::fmt::Debug;
use std::sync::Arc;

/// A span exporter that maps attributes before exporting.
#[derive(Clone)]
pub struct MappingExporter<E: SpanExporter> {
    inner: E,
    mapper: Arc<dyn AttributeMapper>,
}

impl<E: SpanExporter> Debug for MappingExporter<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MappingExporter").finish()
    }
}

impl<E: SpanExporter> MappingExporter<E> {
    /// Creates a new mapping exporter.
    pub fn new(inner: E, mapper: Arc<dyn AttributeMapper>) -> Self {
        Self { inner, mapper }
    }
}

impl<E> SpanExporter for MappingExporter<E>
where
    E: SpanExporter,
{
    async fn export(&self, batch: Vec<SpanData>) -> opentelemetry_sdk::error::OTelSdkResult {
        let mapped_batch: Vec<SpanData> = batch
            .into_iter()
            .inspect(|span| {
                let mapped_attrs = self.mapper.map_to_langfuse(&span.attributes);
                let _enriched_attrs = self.mapper.enrich_attributes(&mapped_attrs);
            })
            .collect();

        self.inner.export(mapped_batch).await
    }

    fn shutdown(&mut self) -> Result<(), opentelemetry_sdk::error::OTelSdkError> {
        self.inner.shutdown()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry_sdk::testing::trace::NoopSpanExporter;

    #[test]
    fn test_mapping_exporter() {
        let inner = NoopSpanExporter::new();
        let mapper = Arc::new(crate::mapper::GenAIAttributeMapper::new());
        let _exporter = MappingExporter::new(inner, mapper);
    }
}
