//! Spatial indices for fast raycasting.
//!
//! This module contains the spatial indices used for fast raycasting. The
//! spatial indices are used to quickly find objects that intersect with a ray.

mod bvh;

pub use bvh::*;

use std::ops::Range;

use crate::math::Ray;

/// A hierarchical spatial index that spatially sorts objects into a tree structure.
pub trait HierarchicalIndex {
    type Volume: RayIntersectionTest;
    type Node: HierarchicalNode<Volume = Self::Volume>;

    /// Returns the nodes of the hierarchical index.
    /// The nodes are stored in a flat array, where the children of a node are
    /// stored in a range after the node itself.
    /// NOTE: The first node is the root node.
    fn nodes(&self) -> &[Self::Node];

    /// Returns the indices of the objects that are spatially sorted.
    fn object_indices(&self) -> &[usize];
}

pub trait HierarchicalNode: Sized {
    /// The type of the bounding volume used by the node.
    type Volume: RayIntersectionTest;

    /// Returns the range of the children of the node within the nodes array.
    fn children(&self) -> Range<usize>;

    /// Returns the range of the objects that are stored in the node.
    fn objects(&self) -> Range<usize>;

    /// Returns the bounding volume of the node.
    fn bounding_volume(&self) -> &Self::Volume;

    /// Tests the children of the node for intersection with the ray.
    /// The function returns the number of children that intersect with the ray.
    /// The indices of the children that intersect with the ray are stored in the
    /// children_indices vector ordered by the distance to the ray origin.
    ///
    /// # Arguments
    /// * `ray` - The ray to test the intersection with.
    /// * `children_indices` - Reference for reusing the children indices vector.
    /// * `nodes` - The nodes of the hierarchical index.
    /// * `max_depth` - Optionally, a value can be provided to limit the intersection. This value
    ///                 usually comes previous intersection tests and can be used to reduce the
    ///                 search space.
    fn intersect_children(
        &self,
        ray: &Ray,
        children_indices: &mut [usize],
        nodes: &[Self],
        max_depth: Option<f32>,
    ) -> usize;
}

/// A trait to enable intersection tests with rays.
pub trait RayIntersectionTest {
    /// Tests the intersection of the ray with the object.
    /// Returns the distance to the intersection point if the ray intersects
    /// with the object, otherwise None.
    ///
    /// # Arguments
    /// * `ray` - The ray to test the intersection with.
    /// * `max_depth` - Optionally, a value can be provided to limit the intersection. This value
    ///             usually comes previous intersection tests and can be used to reduce the
    ///             search space.
    fn intersects_ray(&self, ray: &Ray, max_depth: Option<f32>) -> Option<f32>;
}
