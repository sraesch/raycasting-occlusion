use std::{
    collections::BTreeMap,
    fmt::Display,
    sync::{Arc, Mutex},
    time::Instant,
};

use lazy_static::lazy_static;
use log::error;
use serde::Serialize;

use crate::{Error, Result};

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
    children: BTreeMap<String, StatsNode>,

    /// The values inside the node
    values: BTreeMap<String, f64>,
}

pub struct TimeRecording {
    dst_node: StatsNode,
    t0: Instant,
}

pub trait StatsNodeTrait {
    /// Creates an object that records the time until it is dropped.
    fn register_timing(&self) -> TimeRecording;

    /// Returns a child node with the given name.
    /// If the child node does not exist, it will be created.
    ///
    /// # Arguments
    /// * `name` - The name of the child node.
    fn get_child(&self, name: &str) -> StatsNode;

    /// Returns a mutable reference to the value with the given name.
    /// If the value does not exist, it will be created.
    ///
    /// # Arguments
    /// * `name` - The name of the value.
    /// * `value` - The value to add.
    fn add_value(&self, name: &str, value: f64);

    /// Returns the stats node as a string.
    fn to_string(&self) -> Result<String>;
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
            children: BTreeMap::new(),
            values: BTreeMap::new(),
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

            // dump the values associated with the node
            for (name, value) in self.values.iter() {
                // add indenting
                for _ in 0..(self.depth * 2) {
                    write!(f, " ")?;
                }

                writeln!(f, "{}: {},", name, value)?;
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

    #[inline]
    fn add_value(&self, name: &str, value: f64) {
        self.lock()
            .unwrap()
            .values
            .entry(name.to_owned())
            .and_modify(|v| *v += value)
            .or_insert(value);
    }

    fn to_string(&self) -> Result<String> {
        let serialized = SerializedPrintStats::from_stats(self.clone());
        serde_json::to_string_pretty(&serialized).map_err(|err| {
            error!("Failed to serialize stats: {:?}", err);

            Error::SerializationError(Box::new(err))
        })
    }
}

#[derive(Serialize)]
struct StatsValue {
    /// The name of the value
    pub name: String,

    /// The value
    pub value: f64,
}

#[derive(Serialize)]
struct SerializedPrintStats {
    /// The time in milliseconds
    pub timings_ms: f64,

    /// The value assigned to the node
    pub values: Vec<StatsValue>,

    /// The children nodes
    pub children: BTreeMap<String, Self>,
}

impl SerializedPrintStats {
    pub fn from_stats(stats: StatsNode) -> Self {
        let s = stats.lock().unwrap();

        Self {
            timings_ms: s.as_millis() as f64,
            values: s
                .values
                .iter()
                .map(|(name, value)| StatsValue {
                    name: name.clone(),
                    value: *value,
                })
                .collect(),
            children: s
                .children
                .iter()
                .map(|(name, child)| (name.clone(), Self::from_stats(child.clone())))
                .collect(),
        }
    }
}
