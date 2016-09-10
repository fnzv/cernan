use std::fs;
use std::path::{Path,PathBuf};
use std::io;
use std::io::{ErrorKind,Write,Read,SeekFrom,Seek};
use metric::Event;
use bincode::serde::{serialize,deserialize};
use bincode::SizeLimit;
use std::sync::{atomic,Arc};
use std::{thread,time};

#[inline]
fn u32tou8abe(v: u32) -> [u8; 4] {
    [
        v as u8,
        (v >> 8) as u8,
        (v >> 24) as u8,
        (v >> 16) as u8,
    ]
}

#[inline]
fn u8tou32abe(v: &[u8]) -> u32 {
    (v[3] as u32) +
        ((v[2] as u32) << 8) +
        ((v[1] as u32) << 24) +
        ((v[0] as u32) << 16)
}

#[derive(Debug)]
pub struct Sender {
    root: PathBuf, // directory we store our queues in
    path: PathBuf, // active fp filename
    fp: fs::File,  // active fp
    bytes_written: usize,
    max_bytes: usize,
    global_seq_num: Arc<atomic::AtomicUsize>,
    seq_num: usize,
}

impl Clone for Sender {
    fn clone(&self) -> Sender {
        Sender::new(&self.root, self.max_bytes, self.global_seq_num.clone())
    }
}

#[derive(Debug)]
pub struct Receiver {
    root: PathBuf, // directory we store our queues in
    fp: fs::File,  // active fp
    seq_num: usize,
}

// Here's the plan. The filesystem structure will look like so:
//
//   data-dir/
//      sink-name0/
//         0
//         1
//      sink-name1/
//         0
//
// There will be one Receiver / Sender file. The Sender is responsible for
// switching over to a new file. That means marking the current file read-only
// and dinking up on the name of the next. The Receiver is responsible for
// knowing that if it hits EOF on a read-only file that it should unlink its
// existing file and move on to the next file.
//
// File names always increase.

pub fn channel(name: &str, data_dir: &Path) -> (Sender, Receiver) {
    let root = data_dir.join(name);
    let snd_root = root.clone();
    let rcv_root = root.clone();
    if !root.is_dir() {
        debug!("MKDIR {:?}", root);
        fs::create_dir_all(root).expect("could not create directory");
    }

    let sender = Sender::new(&snd_root, 1_048_576 * 10, Arc::new(atomic::AtomicUsize::new(0)));
    let receiver = Receiver::new(&rcv_root);
    (sender, receiver)
}

impl Receiver {
    pub fn new(root: &Path) -> Receiver {
        let (_, queue_file) = fs::read_dir(root).unwrap().map(|de| {
            let d = de.unwrap();
            let created = d.metadata().unwrap().created().unwrap();
            (created, d.path())
        }).max().unwrap();

        let queue_file1 = queue_file.clone();
        let seq_file_path = queue_file1.file_name().unwrap();

        let seq_num : usize = seq_file_path.to_str().unwrap().parse::<usize>().unwrap();
        let mut fp = fs::OpenOptions::new().read(true).open(queue_file).expect("RECEIVER could not open file");
        fp.seek(SeekFrom::End(0)).expect("could not get to end of file");

        Receiver {
            root: root.to_path_buf(),
            fp: fp,
            seq_num: seq_num,
        }
    }
}


impl Iterator for Receiver {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        let mut sz_buf = [0; 4];
        let mut payload_buf = [0; 1_048_576]; // TODO 1MB hard-limit is a rough business

        loop {
            match self.fp.read_exact(&mut sz_buf) {
                Ok(()) => {
                    let payload_size_in_bytes = u8tou32abe(&sz_buf);
                    match self.fp.read_exact(&mut payload_buf[..payload_size_in_bytes as usize]) {
                        Ok(()) => {
                            match deserialize(&payload_buf[..payload_size_in_bytes as usize]) {
                                Ok(event) => return Some(event),
                                Err(e) => panic!("Failed decoding. Skipping {:?}", e),
                            }
                        }
                        Err(e) => {
                            panic!("Error while recv'ing. Will not retry {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    match e.kind() {
                        ErrorKind::UnexpectedEof => {
                            let metadata = self.fp.metadata().unwrap();
                            if metadata.permissions().readonly() {
                                let old_log = self.root.join(format!("{}", self.seq_num));
                                fs::remove_file(old_log).expect("could not remove log");
                                self.seq_num = self.seq_num.wrapping_add(1);
                                let lg = self.root.join(format!("{}", self.seq_num));
                                loop {
                                    match fs::OpenOptions::new().read(true).open(&lg) {
                                        Ok(fp) => { self.fp = fp; break; }
                                        Err(_) => continue
                                    }
                                }
                            }
                            else {
                                let dur = time::Duration::from_millis(10);
                                thread::sleep(dur);
                            }
                        }
                        _ => {
                            panic!("unable to cope");
                        }
                    }
                }
            }
        }
    }
}

impl Sender {
    pub fn new(data_dir: &Path, max_bytes: usize, global_seq_num: Arc<atomic::AtomicUsize>) -> Sender {
        let seq_num = match fs::read_dir(data_dir).unwrap().map(|de| {
            let d = de.unwrap();
            let created = d.metadata().unwrap().created().unwrap();
            (created, d.path())
        }).max() {
            Some((_, queue_file)) => {
                let seq_file_path = queue_file.file_name().unwrap();
                seq_file_path.to_str().unwrap().parse::<usize>().unwrap()
            },
            None => 0,
        };

        let log = data_dir.join(format!("{}", seq_num));
        let snd_log = log.clone();
        let fp = fs::OpenOptions::new().append(true).create(true).open(log).expect("SENDER NEW could not open file");
        Sender {
            root: data_dir.to_path_buf(),
            path: snd_log,
            fp: fp,
            bytes_written: 0,
            max_bytes: max_bytes,
            global_seq_num: global_seq_num,
            seq_num: seq_num,
        }
    }

    // send writes data out in chunks, like so:
    //
    //  u32: payload_size
    //  [u8] payload
    //
    pub fn send(&mut self, event: &Event) -> Result<usize, io::Error> {
        let mut t = serialize(event, SizeLimit::Infinite).expect("could not serialize");
        // NOTE The conversion of t.len to u32 and usize is _only_ safe when u32
        // <= usize. That's very likely to hold true for machines--for
        // now?--that cernan will run on. However! Once the u32 atomics land in
        // stable we'll be in business.
        let pyld_sz_bytes : [u8; 4] = u32tou8abe(t.len() as u32);
        self.bytes_written = self.bytes_written + t.len();
        let global_seq_num = (*self.global_seq_num).load(atomic::Ordering::Relaxed);
        if self.seq_num < global_seq_num {
            self.seq_num = global_seq_num;
            self.path = self.root.join(format!("{}", self.seq_num));
            self.fp = fs::OpenOptions::new().append(true).create(true).open(&self.path).expect("catching up");
            self.bytes_written = 0;
        }
        if self.bytes_written > self.max_bytes {
            self.seq_num = self.seq_num.wrapping_add(1);
            // set path to read-only
            let mut permissions = self.fp.metadata().unwrap().permissions();
            permissions.set_readonly(true);
            fs::set_permissions(&self.path, permissions).expect("could not set read-only"); // ignore
            // open new fp
            self.path = self.root.join(format!("{}", self.seq_num));
            loop {
                match fs::OpenOptions::new().append(true).create(true).open(&self.path) {
                    Ok(fp) => { self.fp = fp; break; },
                    Err(_) => continue,
                }
            }
            self.bytes_written = 0;
            (*self.global_seq_num).fetch_add(1, atomic::Ordering::AcqRel);
        }
        t.insert(0, pyld_sz_bytes[0]);
        t.insert(0, pyld_sz_bytes[1]);
        t.insert(0, pyld_sz_bytes[2]);
        t.insert(0, pyld_sz_bytes[3]);

        let ref mut fp = self.fp;
        fp.write(&t[..])
    }
}
