#![feature(test)]

extern crate test;
extern crate cernan;

use self::test::Bencher;

use cernan::metric::{Metric,TagMap};

#[bench]
fn bench_overlay_tags_from_map(b: &mut Bencher) {
    let mut tag_map = TagMap::default();
    tag_map.insert("foo".into(), "bar".into());
    tag_map.insert("oof".into(), "rab".into());
    b.iter(|| {
        let mut m = Metric::new("foo", 1.1).overlay_tag("foo", "22");
        m.overlay_tags_from_map(&tag_map);
    });
}

#[bench]
fn bench_merge_tags_from_map(b: &mut Bencher) {
    let mut tag_map = TagMap::default();
    tag_map.insert("foo".into(), "bar".into());
    tag_map.insert("oof".into(), "rab".into());
    b.iter(|| {
        let mut m = Metric::new("foo", 1.1).overlay_tag("foo", "22");
        m.merge_tags_from_map(&tag_map);
    });
}

#[bench]
fn bench_statsd_incr_gauge_no_sample(b: &mut Bencher) {
    b.iter(|| {
        Metric::parse_statsd("a.b:+12.1|g\n").unwrap();
    });
}

#[bench]
fn bench_statsd_incr_gauge_with_sample(b: &mut Bencher) {
    b.iter(|| {
        Metric::parse_statsd("a.b:+12.1|g@2.2\n").unwrap();
    });
}

#[bench]
fn bench_statsd_gauge_no_sample(b: &mut Bencher) {
    b.iter(|| {
        Metric::parse_statsd("a.b:12.1|g\n").unwrap();
    });
}

#[bench]
fn bench_statsd_gauge_mit_sample(b: &mut Bencher) {
    b.iter(|| {
        Metric::parse_statsd("a.b:12.1|g@0.22\n").unwrap();
    });
}

#[bench]
fn bench_statsd_counter_no_sample(b: &mut Bencher) {
    b.iter(|| {
        Metric::parse_statsd("a.b:12.1|c\n").unwrap();
    });
}

#[bench]
fn bench_statsd_counter_with_sample(b: &mut Bencher) {
    b.iter(|| {
        Metric::parse_statsd("a.b:12.1|c@1.0\n").unwrap();
    });
}

#[bench]
fn bench_statsd_timer(b: &mut Bencher) {
    b.iter(|| {
        Metric::parse_statsd("a.b:12.1|ms\n").unwrap();
    });
}

#[bench]
fn bench_statsd_histogram(b: &mut Bencher) {
    b.iter(|| {
        Metric::parse_statsd("a.b:12.1|h\n").unwrap();
    });
}

#[bench]
fn bench_graphite(b: &mut Bencher) {
    b.iter(|| {
        Metric::parse_graphite("fst 1 101\n").unwrap();
    });
}
