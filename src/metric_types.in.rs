#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct LogLine {
    pub time: i64,
    pub path: String,
    pub value: String,
    pub tags: TagMap,
}

#[derive(PartialEq, Serialize, Deserialize, Clone)]
pub struct Metric<'a> {
    pub kind: MetricKind,
    pub name: Cow<'a, str>,
    pub time: i64,
    pub tags: TagMap,
    value: CKMS<f64>,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub enum Event<'a> {
    Statsd(Metric<'a>),
    Graphite(Metric<'a>),
    Log(Vec<LogLine>),
    TimerFlush,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MetricKind {
    Counter,
    Gauge,
    DeltaGauge,
    Timer,
    Histogram,
    Raw,
}
