use cad_import::structure::PrimitiveType;

/// An iterator over the triangles of a page.
pub struct TriangleIterator<I: Iterator<Item = u32>> {
    /// The primitive type for the triangles, i.e., Triangles, TrianglesFan or TrianglesStrip.
    primitive: PrimitiveType,

    /// The raw underlying index iterator.
    indices: I,

    /// For TriangleStrip, we need to flip the orientation of the triangles.
    flip_triangle: bool,

    /// Depending on the primitive type, we need to store previous indices to construct triangles.
    v: [u32; 2],
}

impl<I: Iterator<Item = u32>> TriangleIterator<I> {
    /// Creates a new triangle iterator and returns none if the primitive type is neither
    /// Triangles, TrianglesFan nor TrianglesStrip.
    ///
    /// # Arguments
    /// * `primitive` - The primitive type for the triangles.
    /// * `indices` - The raw underlying index iterator.
    pub fn new(primitive: PrimitiveType, mut indices: I) -> Option<Self> {
        let v = match primitive {
            PrimitiveType::Triangles => [0, 0],
            PrimitiveType::TriangleFan => [
                indices.next().unwrap_or_default(),
                indices.next().unwrap_or_default(),
            ],
            PrimitiveType::TriangleStrip => [
                indices.next().unwrap_or_default(),
                indices.next().unwrap_or_default(),
            ],
            _ => return None,
        };

        Some(Self {
            primitive,
            indices,
            flip_triangle: false,
            v,
        })
    }
}

impl<I: Iterator<Item = u32>> Iterator for TriangleIterator<I> {
    type Item = [u32; 3];

    fn next(&mut self) -> Option<Self::Item> {
        match self.primitive {
            PrimitiveType::Triangles => {
                let v0 = self.indices.next()?;
                let v1 = self.indices.next()?;
                let v2 = self.indices.next()?;

                Some([v0, v1, v2])
            }
            PrimitiveType::TriangleFan => {
                if let Some(v2) = self.indices.next() {
                    let v0 = self.v[0];
                    let v1 = self.v[1];

                    self.v[1] = v2;

                    Some([v0, v1, v2])
                } else {
                    None
                }
            }
            PrimitiveType::TriangleStrip => {
                if let Some(v2) = self.indices.next() {
                    let (v0, v1) = if self.flip_triangle {
                        (self.v[1], self.v[0])
                    } else {
                        (self.v[0], self.v[1])
                    };

                    self.v[0] = self.v[1];
                    self.v[1] = v2;

                    self.flip_triangle = !self.flip_triangle;

                    Some([v0, v1, v2])
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_triangle_iterator_triangles() {
        let indices = vec![0, 1, 2, 3, 4, 5];
        let mut iterator =
            TriangleIterator::new(PrimitiveType::Triangles, indices.into_iter()).unwrap();

        assert_eq!(iterator.next(), Some([0, 1, 2]));
        assert_eq!(iterator.next(), Some([3, 4, 5]));
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn test_triangle_iterator_triangle_fan() {
        let indices = vec![0, 1, 2, 3, 4, 5];
        let mut iterator =
            TriangleIterator::new(PrimitiveType::TriangleFan, indices.into_iter()).unwrap();

        assert_eq!(iterator.next(), Some([0, 1, 2]));
        assert_eq!(iterator.next(), Some([0, 2, 3]));
        assert_eq!(iterator.next(), Some([0, 3, 4]));
        assert_eq!(iterator.next(), Some([0, 4, 5]));
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn test_triangle_iterator_triangle_strip() {
        let indices = vec![0, 1, 2, 3, 4];
        let mut iterator =
            TriangleIterator::new(PrimitiveType::TriangleStrip, indices.into_iter()).unwrap();

        assert_eq!(iterator.next(), Some([0, 1, 2]));
        assert_eq!(iterator.next(), Some([2, 1, 3]));
        assert_eq!(iterator.next(), Some([2, 3, 4]));

        assert_eq!(iterator.next(), None);
    }
}
