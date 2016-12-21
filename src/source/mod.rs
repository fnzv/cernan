mod graphite;
mod statsd;
mod file;
mod flush;
mod federation_receiver;

pub use self::federation_receiver::{FederationReceiver, FederationReceiverConfig};
pub use self::file::{FileServer, FileServerConfig};
pub use self::flush::FlushTimer;
pub use self::graphite::{Graphite, GraphiteConfig};
pub use self::statsd::{Statsd, StatsdConfig};

pub trait Source {
    fn run(&mut self) -> ();
}
