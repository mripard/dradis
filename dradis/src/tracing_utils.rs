use std::time::Instant;

use tracing::{debug, span::Id, Subscriber};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

struct Timings {
    started_at: Instant,
}

pub struct SpanDurationLayer;

impl<S> Layer<S> for SpanDurationLayer
where
    S: Subscriber,
    S: for<'lookup> LookupSpan<'lookup>,
{
    fn on_enter(&self, id: &Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).unwrap();

        span.extensions_mut().insert(Timings {
            started_at: Instant::now(),
        });
    }

    fn on_exit(&self, id: &Id, ctx: Context<'_, S>) {
        let span = ctx.span(&id).unwrap();

        let exts = span.extensions();
        let timings = exts.get::<Timings>().unwrap();

        debug!(
            "Span {} execution took {} ms",
            span.metadata().name(),
            timings.started_at.elapsed().as_millis(),
        );
    }
}
