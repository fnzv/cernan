use metric::{Metric,Event};
use mpsc;
//use chrono;

/// A 'sink' is a sink for metrics.
pub trait Sink {
    fn flush(&mut self) -> ();
    fn snapshot(&mut self) -> ();
    fn deliver(&mut self, point: Metric) -> ();
    fn run(&mut self, mut recv: mpsc::Receiver) {
//        let mut start = chrono::UTC::now();
        while let Some(event) = recv.next() {
            // let pres = chrono::UTC::now();
            // let diff : i64 = (pres - start).num_nanoseconds().unwrap();
            // start = pres;
            // debug!("{} NANOSEC / LOOP", diff);
            match event {
                Event::TimerFlush => self.flush(),
                Event::Snapshot => self.snapshot(),
                Event::Graphite(metric) => self.deliver(metric),
                Event::Statsd(metric) => self.deliver(metric),
            }
        }
        panic!("OOPS FELL OFF THE END");
    }
}
