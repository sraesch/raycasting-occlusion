use crate::math::{aabb_ray, AABB};

use super::{HierarchicalIndex, HierarchicalNode, RayIntersectionTest};

/// Bounding Volume Hierarchy
pub struct BVH {
    /// The nodes of the BVH.
    nodes: Vec<Node>,

    /// The objects of the BVH.
    objects: Vec<usize>,
}

impl HierarchicalIndex for BVH {
    type Volume = AABB;
    type Node = Node;

    #[inline]
    fn nodes(&self) -> &[Self::Node] {
        &self.nodes
    }

    #[inline]
    fn object_indices(&self) -> &[usize] {
        self.objects.as_slice()
    }
}

pub struct Node {
    /// The bounding volume of the node.
    volume: AABB,

    /// The index of the first child node. 0 if the node has no children.
    children: u32,

    /// The range of the objects stored in the node.
    objects: std::ops::Range<u32>,
}

impl HierarchicalNode for Node {
    type Volume = AABB;

    #[inline]
    fn children(&self) -> std::ops::Range<usize> {
        if self.children == 0 {
            0..0
        } else {
            (self.children as usize)..(self.children as usize + 2)
        }
    }

    #[inline]
    fn objects(&self) -> std::ops::Range<usize> {
        self.objects.start as usize..self.objects.end as usize
    }

    #[inline]
    fn bounding_volume(&self) -> &Self::Volume {
        &self.volume
    }

    fn intersect_children(
        &self,
        ray: &crate::math::Ray,
        children_indices: &mut [usize],
        nodes: &[Self],
        max_depth: Option<f32>,
    ) -> usize {
        let mut count = 0;

        let mut f = [0f32; 2];
        for (f, i) in f.iter_mut().zip(self.children()) {
            if let Some(t) = nodes[i].bounding_volume().intersects_ray(ray, max_depth) {
                children_indices[count] = i;
                count += 1;

                *f = t;
            }
        }

        // Sort the children by distance to the ray origin.
        if count == 2 && f[0] > f[1] {
            children_indices.swap(0, 1);
        }

        count
    }
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

impl RayIntersectionTest for AABB {
    fn intersects_ray(&self, ray: &crate::math::Ray, max_depth: Option<f32>) -> Option<f32> {
        aabb_ray(self, ray, max_depth)
    }
}
