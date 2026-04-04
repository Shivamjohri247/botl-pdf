use rstar::{RTree, RTreeObject, AABB};

use crate::geometry::BBox;

/// A spatially indexed item with an associated BBox.
#[derive(Debug, Clone)]
pub struct SpatialItem<T> {
    pub bbox: BBox,
    pub data: T,
}

impl<T> RTreeObject for SpatialItem<T> {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners(
            [self.bbox.x0, self.bbox.y0],
            [self.bbox.x1, self.bbox.y1],
        )
    }
}

/// R-tree based spatial index for efficient bbox queries.
pub struct SpatialIndex<T> {
    tree: RTree<SpatialItem<T>>,
}

impl<T: Clone> SpatialIndex<T> {
    pub fn new(items: Vec<SpatialItem<T>>) -> Self {
        Self {
            tree: RTree::bulk_load(items),
        }
    }

    /// Find all items whose bboxes overlap with the query bbox.
    pub fn query(&self, bbox: &BBox) -> Vec<&SpatialItem<T>> {
        let envelope = AABB::from_corners(
            [bbox.x0, bbox.y0],
            [bbox.x1, bbox.y1],
        );
        self.tree
            .locate_in_envelope_intersecting(&envelope)
            .collect()
    }

    /// Find all items fully contained within the query bbox.
    pub fn query_contained(&self, bbox: &BBox) -> Vec<&SpatialItem<T>> {
        self.query(bbox)
            .into_iter()
            .filter(|item| bbox.contains(&item.bbox))
            .collect()
    }

    /// Returns the number of items in the index.
    pub fn len(&self) -> usize {
        self.tree.size()
    }

    /// Returns true if the index is empty.
    pub fn is_empty(&self) -> bool {
        self.tree.size() == 0
    }
}
