use std::{
    collections::HashMap,
    fmt::Display,
    sync::{Arc, Mutex},
    time::Instant,
};

use lazy_static::lazy_static;

pub type StatsNode = Arc<Mutex<Stats>>;

lazy_static! {
    static ref ROOT_STATS: StatsNode = Arc::new(Mutex::new(Stats::new(1)));
}

pub struct Stats {
    /// The hierarchical depth of the stats node
    depth: usize,

    /// The node specific timings in nanoseconds
    timings_ns: u128,

    /// Further children timings
    children: HashMap<String, StatsNode>,
}

pub struct TimeRecording {
    dst_node: StatsNode,
    t0: Instant,
}

pub trait StatsNodeTrait {
    fn register_timing(&self) -> TimeRecording;

    fn get_child(&self, name: &str) -> StatsNode;
}

impl TimeRecording {
    pub fn new(dst_node: StatsNode) -> Self {
        let t0 = Instant::now();

        Self { dst_node, t0 }
    }
}

impl Drop for TimeRecording {
    #[inline]
    fn drop(&mut self) {
        let ns = self.t0.elapsed().as_nanos();
        self.dst_node.lock().unwrap().timings_ns += ns;
    }
}

impl Stats {
    /// Returns the root stats node
    #[inline]
    pub fn root() -> StatsNode {
        ROOT_STATS.clone()
    }

    /// Returns a children time node for the given identifier.
    ///
    /// # Arguments
    /// * `name` - The name of the children time.
    #[inline]
    pub fn get_child(&mut self, name: String) -> StatsNode {
        let node = self
            .children
            .entry(name)
            .or_insert(Arc::new(Mutex::new(Stats::new(self.depth + 1))));

        node.clone()
    }

    /// Returns the elapsed time of the node in nano-seconds
    #[inline]
    pub fn as_nanos(&self) -> u128 {
        self.timings_ns
    }

    /// Returns the elapsed time of the node in nano-seconds
    #[inline]
    pub fn as_millis(&self) -> u128 {
        self.timings_ns / 1000000u128
    }

    /// Internal function for creating a new time node.
    fn new(depth: usize) -> Self {
        Self {
            depth,
            timings_ns: 0u128,
            children: HashMap::new(),
        }
    }
}

impl Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.children.is_empty() {
            writeln!(f, "{} ms,", self.as_millis())
        } else {
            if self.timings_ns == 0u128 {
                writeln!(f, "{{")?;
            } else {
                writeln!(f, "{} ms {{", self.as_millis())?;
            }

            for (name, child) in self.children.iter() {
                // add indenting
                for _ in 0..(self.depth * 2) {
                    write!(f, " ")?;
                }

                write!(f, "{}: ", name)?;
                child.lock().unwrap().fmt(f)?;
            }

            writeln!(f, "}},")
        }
    }
}

impl StatsNodeTrait for StatsNode {
    #[inline]
    fn register_timing(&self) -> TimeRecording {
        TimeRecording::new(self.clone())
    }

    #[inline]
    fn get_child(&self, name: &str) -> StatsNode {
        self.lock().unwrap().get_child(name.to_owned())
    }
}
