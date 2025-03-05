use crate::Visibility;

/// Computes the visibility based on the given ids in the framebuffer.
///
/// # Arguments
/// * `visibility` - The visibility to update.
/// * `id_buffer` - The buffer containing the ids of the objects.
/// * `num_objects` - The number of objects in the scene.
pub fn compute_visibility_from_id_buffer(
    visibility: &mut Visibility,
    id_buffer: &[Option<u32>],
    num_objects: usize,
) {
    // first create a histogram of the rendered ids
    let mut histogram = vec![0u32; num_objects];
    for id in id_buffer.iter() {
        match id {
            Some(id) => {
                histogram[*id as usize] += 1;
            }
            None => {}
        }
    }

    // make sure that the visibility has the correct size
    visibility.resize(num_objects, (0, 0f32));

    // now fill the visibility based on the histogram, but not order yet
    for ((object_id, count), v) in histogram.iter().enumerate().zip(visibility.iter_mut()) {
        v.0 = object_id as u32;
        v.1 = *count as f32 / id_buffer.len() as f32;
    }

    // sort the visibility based on the visibility
    visibility.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
}
