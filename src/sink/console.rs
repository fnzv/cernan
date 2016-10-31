use sink::Sink;
use buckets::Buckets;
use metric::{Metric, LogLine};
use chrono;

pub struct Console<'a> {
    aggrs: Buckets<'a>,
}

impl<'a> Console<'a> {
    pub fn new(bin_width: i64) -> Console<'a> {
        Console { aggrs: Buckets::new(bin_width) }
    }
}

impl<'a> Default for Console<'a> {
    fn default() -> Self {
        Self::new(1)
    }
}

/// Print a single stats line.
fn fmt_line(key: &str, value: f64) {
    println!("    {}: {}", key, value)
}


impl<'a> Sink for Console<'a> {
    fn deliver(&mut self, point: Metric<'a>) {
        self.aggrs.add(point);
    }

    fn deliver_lines(&mut self, _: Vec<LogLine>) {
        // nothing, intentionally
    }

    fn flush(&mut self) {
        let now = chrono::UTC::now();
        println!("Flushing metrics: {}", now.to_rfc3339());

        println!("  counters:");
        for (key, value) in self.aggrs.counters() {
            for m in value {
                if let Some(f) = m.value() {
                    fmt_line(key, f)
                }
            }
        }

        println!("  gauges:");
        for (key, value) in self.aggrs.gauges() {
            for m in value {
                if let Some(f) = m.value() {
                    fmt_line(key, f)
                }
            }
        }

        println!("  raws:");
        for (key, value) in self.aggrs.raws() {
            for m in value {
                if let Some(f) = m.value() {
                    fmt_line(key, f)
                }
            }
        }

        println!("  histograms:");
        for (key, hists) in self.aggrs.histograms() {
            for h in hists {
                for tup in &[("min", 0.0),
                             ("max", 1.0),
                             ("50", 0.5),
                             ("90", 0.90),
                             ("99", 0.99),
                             ("999", 0.999)] {
                    let stat: &str = tup.0;
                    let quant: f64 = tup.1;
                    println!("    {}: {} {}", key, stat, h.query(quant).unwrap());
                }
            }
        }

        println!("  timers:");
        for (key, tmrs) in self.aggrs.timers() {
            for tmr in tmrs {
                for tup in &[("min", 0.0),
                             ("max", 1.0),
                             ("50", 0.5),
                             ("90", 0.90),
                             ("99", 0.99),
                             ("999", 0.999)] {
                    let stat: &str = tup.0;
                    let quant: f64 = tup.1;
                    println!("    {}: {} {}", key, stat, tmr.query(quant).unwrap());
                }
            }
        }

        self.aggrs.reset();
    }
}
