use crate::event::{Event, Metric, MetricValue};
use crate::metrics::{capture_metrics, get_controller};
use async_graphql::{validators::IntRange, Interface, Object, Subscription};
use async_stream::stream;
use chrono::{DateTime, Utc};
use tokio::stream::{Stream, StreamExt};
use tokio::time::Duration;

pub struct Uptime(Metric);

#[Object]
impl Uptime {
    /// Metric timestamp
    async fn timestamp(&self) -> Option<DateTime<Utc>> {
        self.0.timestamp
    }

    /// Number of seconds the Vector instance has been alive
    async fn seconds(&self) -> f64 {
        match self.0.value {
            MetricValue::Gauge { value } => value,
            _ => 0.00,
        }
    }
}

impl From<Metric> for Uptime {
    fn from(m: Metric) -> Self {
        Self(m)
    }
}

pub struct EventsProcessed(Metric);

#[Object]
impl EventsProcessed {
    /// Metric timestamp
    async fn timestamp(&self) -> Option<DateTime<Utc>> {
        self.0.timestamp
    }

    /// Number of events processed
    async fn events_processed(&self) -> f64 {
        match self.0.value {
            MetricValue::Counter { value } => value,
            _ => 0.00,
        }
    }
}

impl From<Metric> for EventsProcessed {
    fn from(m: Metric) -> Self {
        Self(m)
    }
}

pub struct BytesProcessed(Metric);

#[Object]
impl BytesProcessed {
    /// Metric timestamp
    async fn timestamp(&self) -> Option<DateTime<Utc>> {
        self.0.timestamp
    }

    /// Number of bytes processed
    async fn bytes_processed(&self) -> f64 {
        match self.0.value {
            MetricValue::Counter { value } => value,
            _ => 0.00,
        }
    }
}

impl From<Metric> for BytesProcessed {
    fn from(m: Metric) -> Self {
        Self(m)
    }
}

#[Interface(field(name = "timestamp", type = "Option<DateTime<Utc>>"))]
pub enum MetricType {
    Uptime(Uptime),
    EventsProcessed(EventsProcessed),
    BytesProcessed(BytesProcessed),
}

#[derive(Default)]
pub struct MetricsSubscription;

#[Subscription]
impl MetricsSubscription {
    /// Returns all Vector metrics, aggregated at the provided millisecond interval
    async fn metrics(
        &self,
        #[arg(validator(IntRange(min = "100", max = "60_000")))] interval: i32,
    ) -> impl Stream<Item = MetricType> {
        get_metrics(interval).filter_map(|m| match m.name.as_str() {
            "uptime_seconds" => Some(MetricType::Uptime(m.into())),
            "events_processed" => Some(MetricType::EventsProcessed(m.into())),
            "bytes_processed" => Some(MetricType::BytesProcessed(m.into())),
            _ => None,
        })
    }
}

/// Returns a stream of `Metric`s, collected at the provided millisecond interval
fn get_metrics(interval: i32) -> impl Stream<Item = Metric> {
    let controller = get_controller().unwrap();
    let mut interval = tokio::time::interval(Duration::from_millis(interval as u64));

    stream! {
        loop {
            interval.tick().await;
            for ev in capture_metrics(&controller) {
                if let Event::Metric(m) = ev {
                    yield m;
                }
            }
        }
    }
}
