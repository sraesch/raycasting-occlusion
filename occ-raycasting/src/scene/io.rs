use std::{collections::HashMap, path::Path};

use cad_import::{
    loader::Manager,
    structure::{CADData, IndexData, Node, Point3D, Shape},
    ID,
};
use log::{debug, error};
use nalgebra_glm::{Mat4, Vec3};

use crate::{math::mat4_to_mat3x4, Error, Mesh as SceneMesh, Result};

use super::{io_utils::TriangleIterator, Scene, Triangle};

/// Tries to load the scene from the given path.
///
/// # Arguments
/// * `scene` - The scene to load the data into.
/// * `path` - The path to load the scene from.
pub fn load_into_scene<P: AsRef<Path>>(scene: &mut Scene, path: P) -> Result<()> {
    let cad_data = load_cad_data(path.as_ref())?;
    add_cad_data_to_scene(scene, &cad_data);

    Ok(())
}

/// Tries to load the cad data from the given path
///
/// # Arguments
/// * `file_path` - The path to load the CAD data from.
fn load_cad_data(file_path: &Path) -> Result<CADData> {
    let manager = Manager::new();

    let mime_types = determine_mime_types(&manager, file_path)?;

    for mime_type in mime_types.iter() {
        if let Some(loader) = manager.get_loader_by_mime_type(mime_type.as_str()) {
            let cad_data = loader
                .read_file(file_path, mime_type)
                .map_err(Error::CadImport)?;

            return Ok(cad_data);
        }
    }

    error!("Cannot find loader for the input file {:?}", file_path);
    Err(Error::NoLoaderFound)
}

/// Tries to find the mime types for the given file based on the file extension.
///
/// # Arguments
/// * `input_file` - The input file whose extension will be used
pub fn determine_mime_types(manager: &Manager, input_file: &Path) -> Result<Vec<String>> {
    match input_file.extension() {
        Some(ext) => match ext.to_str() {
            Some(ext) => Ok(manager.get_mime_types_for_extension(ext)),
            None => Err(Error::InvalidFileExtension),
        },
        None => Err(Error::InvalidFileExtension),
    }
}

/// Adds the given CAD data to the given scene by traversing over the node structure.
///
/// # Arguments
/// * `scene` - The scene to which the data will be added.
/// * `cad_data` - The CAD data to convert to a scene.
pub fn add_cad_data_to_scene(scene: &mut Scene, cad_data: &CADData) {
    let root_node = cad_data.get_root_node();
    let traversal_context = TraversalContext::new(root_node);
    let mut traversal_data = TraversalData::new();

    traverse(scene, root_node, traversal_context, &mut traversal_data);
}

/// Internal function for traversing over the node structure and copying all data to GPU.
///
/// # Arguments
/// * `scene` - The scene to which the data will be added.
/// * `node` - The currently visited node.
/// * `traversal_context` - Additional information used during traversal.
/// * `traversal_data` - Additional data used during traversal.
fn traverse(
    scene: &mut Scene,
    node: &Node,
    traversal_context: TraversalContext,
    traversal_data: &mut TraversalData,
) {
    let transform = mat4_to_mat3x4(&traversal_context.transform);

    // iterate over the shapes referenced by the current node and create corresponding meshes and objects
    let shapes: &[std::rc::Rc<Shape>] = node.get_shapes();
    for shape in shapes {
        let mesh_index = create_or_get_mesh(scene, shape, traversal_data);

        scene.objects.push(super::Object {
            mesh_index,
            transform,
        });
    }

    // traverse over the children of the current node
    for child in node.get_children().iter() {
        // compute new transform
        let child_traversal_context = traversal_context.derive(child);

        traverse(scene, child, child_traversal_context, traversal_data);
    }
}

/// Returns the corresponding mesh index for the given shape.
/// If the a mesh for the shape does not exist yet, it will be created.
///
/// # Arguments
/// * `scene` - The scene to which the shape belongs.
/// * `shape` - The shape for which to get the mesh index.
/// * `traversal_data` - Additional data used during traversal.
fn create_or_get_mesh(scene: &mut Scene, shape: &Shape, traversal_data: &mut TraversalData) -> u32 {
    let shape_id = shape.get_id();

    // check if a mesh for this shape already exists
    if let Some(index) = traversal_data.shape_map.get(&shape_id) {
        return *index;
    }

    let mesh_index = scene.meshes.len() as u32;

    let mesh = create_mesh_from_shape(shape);
    scene.meshes.push(mesh);

    traversal_data.shape_map.insert(shape_id, mesh_index);

    mesh_index
}

/// Creates a mesh based on the given shape.
///
/// # Arguments
/// * `shape` - The shape to create the mesh from.
fn create_mesh_from_shape(shape: &Shape) -> SceneMesh {
    let mut mesh = SceneMesh::default();

    // iterate over the parts of the shape and append them to the mesh if they are triangles
    for part in shape.get_parts() {
        let in_mesh = part.get_mesh();
        let positions = in_mesh.get_vertices().get_positions().as_slice();
        let in_primitive_data = in_mesh.get_primitives();
        let primitive_type = in_primitive_data.get_primitive_type();

        // append the triangles to the mesh
        match in_primitive_data.get_raw_index_data() {
            IndexData::Indices(indices) => {
                let triangle_iterator =
                    TriangleIterator::new(primitive_type, indices.iter().copied());

                if let Some(triangle_iterator) = triangle_iterator {
                    append_to_mesh(&mut mesh, positions, triangle_iterator);
                } else {
                    debug!("Primitive type {:?} is not triangle", primitive_type);
                }
            }
            IndexData::NonIndexed(n) => {
                let n = *n as u32;
                let indices = 0..n;
                let triangle_iterator = TriangleIterator::new(primitive_type, indices);

                if let Some(triangle_iterator) = triangle_iterator {
                    append_to_mesh(&mut mesh, positions, triangle_iterator);
                } else {
                    debug!("Primitive type {:?} is not triangle", primitive_type);
                }
            }
        }
    }

    mesh
}

/// Appends the given triangles to the mesh.
///
/// # Arguments
/// * `mesh` - The mesh to which the triangles will be appended.
/// * `pos` - The positions of the vertices of the triangles.
/// * `triangles` - The triangles to append to the mesh.
fn append_to_mesh<I>(mesh: &mut SceneMesh, pos: &[Point3D], triangles: TriangleIterator<I>)
where
    I: Iterator<Item = u32>,
{
    let index_offset = mesh.vertices.len() as u32;

    // add positions to the mesh
    mesh.vertices
        .extend(pos.iter().map(|p| Vec3::from_row_slice(p.0.as_slice())));

    // add triangles to the mesh
    mesh.indices.extend(triangles.map(|t| {
        Triangle::new(
            t[0] + index_offset,
            t[1] + index_offset,
            t[2] + index_offset,
        )
    }));
}

/// Contextual data used during traversing the node data.
#[derive(Clone)]
struct TraversalContext {
    /// The current transformation matrix
    transform: Mat4,
}

impl TraversalContext {
    /// Returns a new empty traversal context.
    pub fn new(root_node: &Node) -> Self {
        let transform: Mat4 = match root_node.get_transform() {
            Some(t) => Mat4::from_column_slice(t.as_slice()),
            None => Mat4::identity(),
        };

        Self { transform }
    }

    /// Returns a new traversal context by visiting the given node.
    ///
    /// # Arguments
    /// * `node` - The node to visit based on the current traversal context
    pub fn derive(&self, node: &Node) -> Self {
        let mut result = self.clone();

        if let Some(t) = node.get_transform() {
            result.transform *= Mat4::from_column_slice(t.as_slice());
        }

        result
    }
}

struct TraversalData {
    pub shape_map: HashMap<ID, u32>,
}

impl TraversalData {
    pub fn new() -> Self {
        Self {
            shape_map: HashMap::new(),
        }
    }
}
