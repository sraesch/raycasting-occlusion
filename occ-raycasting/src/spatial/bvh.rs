use crate::math::AABB;

/// Bounding Volume Hierarchy
pub struct BVH {
    /// The nodes of the BVH.
    nodes: Vec<Node>,

    /// The objects of the BVH.
    objects: Vec<usize>,
}

pub struct Node {
    /// The bounding volume of the node.
    volume: AABB,

    /// The index of the first child node. 0 if the node has no children.
    children: u32,

    /// The range of the objects stored in the node.
    objects: std::ops::Range<u32>,
}

pub struct Builder {
    nodes: Vec<Node>,
    objects: Vec<usize>,
    options: BVHOptions,
}

impl Builder {
    /// Creates a new BVH builder with the provided options.
    ///
    /// # Arguments
    /// * `options` - The options for the BVH.
    pub fn new(options: BVHOptions) -> Self {
        Self {
            options,
            nodes: Vec::new(),
            objects: Vec::new(),
        }
    }

    /// Builds the BVH from the provided objects.
    ///
    /// # Arguments
    /// * `objects` - The objects to build the BVH from.
    pub fn build<Object>(mut self, objects: &[Object]) -> BVH {
        self.objects = (0..objects.len()).collect();

        BVH {
            nodes: self.nodes,
            objects: self.objects,
        }
    }
}

pub struct BVHOptions {
    pub max_depth: usize,
    pub max_objects_per_node: usize,
}
