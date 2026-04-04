//! Integration tests for geometry types.
//!
//! Tests BBox operations, Matrix transformations, and SpatialIndex queries.

use botl_pdf_core::geometry::bbox::{BBox, Point};
use botl_pdf_core::geometry::matrix::Matrix;
use botl_pdf_core::geometry::spatial::{SpatialIndex, SpatialItem};

// ===========================================================================
// BBox tests
// ===========================================================================

#[test]
fn test_bbox_new() {
    let bbox = BBox::new(1.0, 2.0, 3.0, 4.0);
    assert!((bbox.x0 - 1.0).abs() < f64::EPSILON);
    assert!((bbox.y0 - 2.0).abs() < f64::EPSILON);
    assert!((bbox.x1 - 3.0).abs() < f64::EPSILON);
    assert!((bbox.y1 - 4.0).abs() < f64::EPSILON);
}

#[test]
fn test_bbox_width() {
    let bbox = BBox::new(10.0, 20.0, 50.0, 80.0);
    assert!((bbox.width() - 40.0).abs() < f64::EPSILON);
}

#[test]
fn test_bbox_height() {
    let bbox = BBox::new(10.0, 20.0, 50.0, 80.0);
    assert!((bbox.height() - 60.0).abs() < f64::EPSILON);
}

#[test]
fn test_bbox_area() {
    let bbox = BBox::new(0.0, 0.0, 10.0, 5.0);
    assert!((bbox.area() - 50.0).abs() < f64::EPSILON);
}

#[test]
fn test_bbox_center() {
    let bbox = BBox::new(0.0, 0.0, 10.0, 20.0);
    let (cx, cy) = bbox.center();
    assert!((cx - 5.0).abs() < f64::EPSILON);
    assert!((cy - 10.0).abs() < f64::EPSILON);
}

#[test]
fn test_bbox_center_asymmetric() {
    let bbox = BBox::new(10.0, 20.0, 30.0, 60.0);
    let (cx, cy) = bbox.center();
    assert!((cx - 20.0).abs() < f64::EPSILON);
    assert!((cy - 40.0).abs() < f64::EPSILON);
}

// ---------------------------------------------------------------------------
// BBox contains
// ---------------------------------------------------------------------------

#[test]
fn test_bbox_contains_self() {
    let bbox = BBox::new(0.0, 0.0, 10.0, 10.0);
    assert!(bbox.contains(&bbox));
}

#[test]
fn test_bbox_contains_smaller() {
    let outer = BBox::new(0.0, 0.0, 10.0, 10.0);
    let inner = BBox::new(2.0, 2.0, 8.0, 8.0);
    assert!(outer.contains(&inner));
}

#[test]
fn test_bbox_does_not_contain_larger() {
    let inner = BBox::new(2.0, 2.0, 8.0, 8.0);
    let outer = BBox::new(0.0, 0.0, 10.0, 10.0);
    assert!(!inner.contains(&outer));
}

#[test]
fn test_bbox_does_not_contain_overlapping() {
    let a = BBox::new(0.0, 0.0, 10.0, 10.0);
    let b = BBox::new(5.0, 5.0, 15.0, 15.0);
    assert!(!a.contains(&b));
    assert!(!b.contains(&a));
}

#[test]
fn test_bbox_contains_edge_touching() {
    let outer = BBox::new(0.0, 0.0, 10.0, 10.0);
    let edge = BBox::new(0.0, 0.0, 10.0, 10.0);
    assert!(outer.contains(&edge));
}

// ---------------------------------------------------------------------------
// BBox overlaps
// ---------------------------------------------------------------------------

#[test]
fn test_bbox_overlaps_adjacent() {
    let a = BBox::new(0.0, 0.0, 10.0, 10.0);
    let b = BBox::new(5.0, 5.0, 15.0, 15.0);
    assert!(a.overlaps(&b, 0.0));
    assert!(b.overlaps(&a, 0.0));
}

#[test]
fn test_bbox_overlaps_no_overlap() {
    let a = BBox::new(0.0, 0.0, 10.0, 10.0);
    let b = BBox::new(20.0, 20.0, 30.0, 30.0);
    assert!(!a.overlaps(&b, 0.0));
}

#[test]
fn test_bbox_overlaps_touching_edge() {
    let a = BBox::new(0.0, 0.0, 10.0, 10.0);
    let b = BBox::new(10.0, 0.0, 20.0, 10.0);
    // Edges touching: ix0 == ix1, so no overlap
    assert!(!a.overlaps(&b, 0.0));
}

#[test]
fn test_bbox_overlaps_with_threshold() {
    // Two boxes with a small overlap area
    let a = BBox::new(0.0, 0.0, 10.0, 10.0); // area = 100
    let b = BBox::new(9.0, 0.0, 19.0, 10.0); // overlap area = 1x10 = 10
                                             // 10 / min(100, 100) = 0.1
    assert!(a.overlaps(&b, 0.0), "Should overlap with threshold 0");
    assert!(a.overlaps(&b, 0.05), "Should overlap with threshold 0.05");
    assert!(
        !a.overlaps(&b, 0.15),
        "Should not overlap with threshold 0.15"
    );
}

// ---------------------------------------------------------------------------
// BBox merge
// ---------------------------------------------------------------------------

#[test]
fn test_bbox_merge_adjacent() {
    let a = BBox::new(0.0, 0.0, 10.0, 10.0);
    let b = BBox::new(5.0, 5.0, 15.0, 15.0);
    let merged = a.merge(&b);
    assert!((merged.x0 - 0.0).abs() < f64::EPSILON);
    assert!((merged.y0 - 0.0).abs() < f64::EPSILON);
    assert!((merged.x1 - 15.0).abs() < f64::EPSILON);
    assert!((merged.y1 - 15.0).abs() < f64::EPSILON);
}

#[test]
fn test_bbox_merge_disjoint() {
    let a = BBox::new(0.0, 0.0, 5.0, 5.0);
    let b = BBox::new(10.0, 10.0, 20.0, 20.0);
    let merged = a.merge(&b);
    assert!((merged.x0 - 0.0).abs() < f64::EPSILON);
    assert!((merged.y0 - 0.0).abs() < f64::EPSILON);
    assert!((merged.x1 - 20.0).abs() < f64::EPSILON);
    assert!((merged.y1 - 20.0).abs() < f64::EPSILON);
}

#[test]
fn test_bbox_merge_with_self() {
    let a = BBox::new(1.0, 2.0, 3.0, 4.0);
    let merged = a.merge(&a);
    assert_eq!(merged, a);
}

// ---------------------------------------------------------------------------
// BBox intersect
// ---------------------------------------------------------------------------

#[test]
fn test_bbox_intersect_overlapping() {
    let a = BBox::new(0.0, 0.0, 10.0, 10.0);
    let b = BBox::new(5.0, 5.0, 15.0, 15.0);
    let intersection = a.intersect(&b);
    assert!(intersection.is_some());
    let i = intersection.unwrap();
    assert!((i.x0 - 5.0).abs() < f64::EPSILON);
    assert!((i.y0 - 5.0).abs() < f64::EPSILON);
    assert!((i.x1 - 10.0).abs() < f64::EPSILON);
    assert!((i.y1 - 10.0).abs() < f64::EPSILON);
}

#[test]
fn test_bbox_intersect_no_overlap() {
    let a = BBox::new(0.0, 0.0, 10.0, 10.0);
    let b = BBox::new(20.0, 20.0, 30.0, 30.0);
    assert!(a.intersect(&b).is_none());
}

#[test]
fn test_bbox_intersect_touching_edge() {
    let a = BBox::new(0.0, 0.0, 10.0, 10.0);
    let b = BBox::new(10.0, 0.0, 20.0, 10.0);
    // x0 == x1, so no intersection
    assert!(a.intersect(&b).is_none());
}

#[test]
fn test_bbox_intersect_contained() {
    let outer = BBox::new(0.0, 0.0, 20.0, 20.0);
    let inner = BBox::new(5.0, 5.0, 10.0, 10.0);
    let intersection = outer.intersect(&inner);
    assert!(intersection.is_some());
    let i = intersection.unwrap();
    assert_eq!(i, inner);
}

#[test]
fn test_bbox_intersect_commutative() {
    let a = BBox::new(0.0, 0.0, 10.0, 10.0);
    let b = BBox::new(5.0, 5.0, 15.0, 15.0);
    assert_eq!(a.intersect(&b), b.intersect(&a));
}

// ---------------------------------------------------------------------------
// BBox display
// ---------------------------------------------------------------------------

#[test]
fn test_bbox_display() {
    let bbox = BBox::new(1.0, 2.0, 3.0, 4.0);
    let display = format!("{}", bbox);
    assert!(display.contains("1.0"));
    assert!(display.contains("3.0"));
}

// ---------------------------------------------------------------------------
// BBox equality
// ---------------------------------------------------------------------------

#[test]
fn test_bbox_equality() {
    let a = BBox::new(1.0, 2.0, 3.0, 4.0);
    let b = BBox::new(1.0, 2.0, 3.0, 4.0);
    let c = BBox::new(1.0, 2.0, 3.0, 5.0);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

// ===========================================================================
// Point tests
// ===========================================================================

#[test]
fn test_point_new() {
    let p = Point::new(3.0, 4.0);
    assert!((p.x - 3.0).abs() < f64::EPSILON);
    assert!((p.y - 4.0).abs() < f64::EPSILON);
}

#[test]
fn test_point_equality() {
    let a = Point::new(1.0, 2.0);
    let b = Point::new(1.0, 2.0);
    let c = Point::new(1.0, 3.0);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

// ===========================================================================
// Matrix tests
// ===========================================================================

#[test]
fn test_matrix_identity() {
    let id = Matrix::IDENTITY;
    assert!((id.a - 1.0).abs() < f64::EPSILON);
    assert!((id.b - 0.0).abs() < f64::EPSILON);
    assert!((id.c - 0.0).abs() < f64::EPSILON);
    assert!((id.d - 1.0).abs() < f64::EPSILON);
    assert!((id.e - 0.0).abs() < f64::EPSILON);
    assert!((id.f - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_matrix_new() {
    let m = Matrix::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
    assert!((m.a - 1.0).abs() < f64::EPSILON);
    assert!((m.b - 2.0).abs() < f64::EPSILON);
    assert!((m.c - 3.0).abs() < f64::EPSILON);
    assert!((m.d - 4.0).abs() < f64::EPSILON);
    assert!((m.e - 5.0).abs() < f64::EPSILON);
    assert!((m.f - 6.0).abs() < f64::EPSILON);
}

#[test]
fn test_matrix_from_slice() {
    let vals = [1.0, 0.0, 0.0, 1.0, 10.0, 20.0];
    let m = Matrix::from_slice(&vals).unwrap();
    assert!((m.e - 10.0).abs() < f64::EPSILON);
    assert!((m.f - 20.0).abs() < f64::EPSILON);
}

#[test]
fn test_matrix_from_slice_wrong_length() {
    assert!(Matrix::from_slice(&[1.0, 2.0, 3.0]).is_none());
    assert!(Matrix::from_slice(&[]).is_none());
}

#[test]
fn test_matrix_identity_transform_point() {
    let id = Matrix::IDENTITY;
    let p = Point::new(5.0, 10.0);
    let result = id.transform_point(&p);
    assert!((result.x - 5.0).abs() < f64::EPSILON);
    assert!((result.y - 10.0).abs() < f64::EPSILON);
}

#[test]
fn test_matrix_translate() {
    let t = Matrix::translate(10.0, 20.0);
    let p = Point::new(5.0, 5.0);
    let result = t.transform_point(&p);
    assert!((result.x - 15.0).abs() < f64::EPSILON);
    assert!((result.y - 25.0).abs() < f64::EPSILON);
}

#[test]
fn test_matrix_scale() {
    let s = Matrix::scale(2.0, 3.0);
    let p = Point::new(5.0, 4.0);
    let result = s.transform_point(&p);
    assert!((result.x - 10.0).abs() < f64::EPSILON);
    assert!((result.y - 12.0).abs() < f64::EPSILON);
}

#[test]
fn test_matrix_scale_negative() {
    let s = Matrix::scale(-1.0, 1.0);
    let p = Point::new(5.0, 10.0);
    let result = s.transform_point(&p);
    assert!((result.x - (-5.0)).abs() < f64::EPSILON);
    assert!((result.y - 10.0).abs() < f64::EPSILON);
}

#[test]
fn test_matrix_rotate_90_degrees() {
    let r = Matrix::rotate(std::f64::consts::FRAC_PI_2); // 90 degrees
    let p = Point::new(1.0, 0.0);
    let result = r.transform_point(&p);
    // cos(90)=0, sin(90)=1
    // x' = 0*1 + (-1)*0 + 0 = 0
    // y' = 1*1 + 0*0 + 0 = 1
    assert!((result.x - 0.0).abs() < 1e-10);
    assert!((result.y - 1.0).abs() < 1e-10);
}

#[test]
fn test_matrix_rotate_180_degrees() {
    let r = Matrix::rotate(std::f64::consts::PI);
    let p = Point::new(1.0, 0.0);
    let result = r.transform_point(&p);
    assert!((result.x - (-1.0)).abs() < 1e-10);
    assert!((result.y - 0.0).abs() < 1e-10);
}

#[test]
fn test_matrix_multiply_identity() {
    let m = Matrix::new(2.0, 3.0, 4.0, 5.0, 6.0, 7.0);
    let result = m.multiply(&Matrix::IDENTITY);
    assert_eq!(result, m);
}

#[test]
fn test_matrix_multiply_identity_reversed() {
    let m = Matrix::new(2.0, 3.0, 4.0, 5.0, 6.0, 7.0);
    let result = Matrix::IDENTITY.multiply(&m);
    assert_eq!(result, m);
}

#[test]
fn test_matrix_multiply_translate_then_scale() {
    let t = Matrix::translate(10.0, 0.0);
    let s = Matrix::scale(2.0, 2.0);
    // First translate, then scale: scale * translate
    let combined = s.multiply(&t);
    let p = Point::new(5.0, 5.0);
    let result = combined.transform_point(&p);
    // Translate: (15, 5), then scale: (30, 10)
    assert!((result.x - 30.0).abs() < f64::EPSILON);
    assert!((result.y - 10.0).abs() < f64::EPSILON);
}

#[test]
fn test_matrix_mul_operator() {
    let a = Matrix::translate(5.0, 0.0);
    let b = Matrix::scale(2.0, 2.0);
    let result = a * b;
    // a * b means: apply b first, then a (self * other)
    // b scales (1,1) -> (2,2), then a translates -> (7,2)
    let p = Point::new(1.0, 1.0);
    let transformed = result.transform_point(&p);
    assert!((transformed.x - 7.0).abs() < 1e-10);
    assert!((transformed.y - 2.0).abs() < 1e-10);
}

#[test]
fn test_matrix_transform_vector() {
    let m = Matrix::new(2.0, 0.0, 0.0, 3.0, 100.0, 200.0);
    let (dx, dy) = m.transform_vector(5.0, 10.0);
    // No translation for vectors
    assert!((dx - 10.0).abs() < f64::EPSILON);
    assert!((dy - 30.0).abs() < f64::EPSILON);
}

#[test]
fn test_matrix_scaling_factor_identity() {
    let id = Matrix::IDENTITY;
    assert!((id.scaling_factor() - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_matrix_scaling_factor_scaled() {
    let s = Matrix::scale(3.0, 4.0);
    assert!((s.scaling_factor() - 12.0_f64.sqrt()).abs() < 1e-10);
}

#[test]
fn test_matrix_effective_font_size() {
    let m = Matrix::IDENTITY;
    let effective = m.effective_font_size(12.0);
    assert!((effective - 12.0).abs() < f64::EPSILON);
}

#[test]
fn test_matrix_effective_font_size_scaled() {
    let s = Matrix::scale(1.0, 2.0);
    let effective = s.effective_font_size(12.0);
    assert!((effective - 24.0).abs() < f64::EPSILON);
}

#[test]
fn test_matrix_equality() {
    let a = Matrix::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
    let b = Matrix::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
    let c = Matrix::new(1.0, 2.0, 3.0, 4.0, 5.0, 7.0);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn test_matrix_compose_multiple_transforms() {
    // Realistic PDF scenario: CTM with translation, then text matrix
    let ctm = Matrix::translate(100.0, 700.0);
    let text_matrix = Matrix::new(12.0, 0.0, 0.0, 12.0, 0.0, 0.0);

    // Combined = CTM * text_matrix
    let combined = ctm.multiply(&text_matrix);

    // The (0,0) origin in text space should map to (100, 700) in page space
    let origin = combined.transform_point(&Point::new(0.0, 0.0));
    assert!((origin.x - 100.0).abs() < 1e-10);
    assert!((origin.y - 700.0).abs() < 1e-10);
}

// ===========================================================================
// SpatialIndex tests
// ===========================================================================

#[test]
fn test_spatial_index_empty() {
    let index: SpatialIndex<i32> = SpatialIndex::new(vec![]);
    assert!(index.is_empty());
    assert_eq!(index.len(), 0);
}

#[test]
fn test_spatial_index_single_item() {
    let items = vec![SpatialItem {
        bbox: BBox::new(0.0, 0.0, 10.0, 10.0),
        data: 42,
    }];
    let index = SpatialIndex::new(items);
    assert!(!index.is_empty());
    assert_eq!(index.len(), 1);
}

#[test]
fn test_spatial_index_query_hit() {
    let items = vec![
        SpatialItem {
            bbox: BBox::new(0.0, 0.0, 10.0, 10.0),
            data: 1,
        },
        SpatialItem {
            bbox: BBox::new(20.0, 20.0, 30.0, 30.0),
            data: 2,
        },
    ];
    let index = SpatialIndex::new(items);

    let query = BBox::new(5.0, 5.0, 15.0, 15.0);
    let results = index.query(&query);
    assert_eq!(
        results.len(),
        1,
        "Should find exactly 1 item overlapping the query"
    );
    assert_eq!(results[0].data, 1);
}

#[test]
fn test_spatial_index_query_multiple_hits() {
    let items = vec![
        SpatialItem {
            bbox: BBox::new(0.0, 0.0, 10.0, 10.0),
            data: 1,
        },
        SpatialItem {
            bbox: BBox::new(5.0, 5.0, 15.0, 15.0),
            data: 2,
        },
        SpatialItem {
            bbox: BBox::new(20.0, 20.0, 30.0, 30.0),
            data: 3,
        },
    ];
    let index = SpatialIndex::new(items);

    let query = BBox::new(0.0, 0.0, 15.0, 15.0);
    let results = index.query(&query);
    assert_eq!(
        results.len(),
        2,
        "Should find 2 items overlapping the query"
    );

    let found_data: Vec<i32> = results.iter().map(|r| r.data).collect();
    assert!(found_data.contains(&1));
    assert!(found_data.contains(&2));
}

#[test]
fn test_spatial_index_query_no_hit() {
    let items = vec![SpatialItem {
        bbox: BBox::new(0.0, 0.0, 10.0, 10.0),
        data: 1,
    }];
    let index = SpatialIndex::new(items);

    let query = BBox::new(20.0, 20.0, 30.0, 30.0);
    let results = index.query(&query);
    assert!(results.is_empty());
}

#[test]
fn test_spatial_index_query_contained() {
    let items = vec![
        SpatialItem {
            bbox: BBox::new(2.0, 2.0, 8.0, 8.0), // fully inside
            data: 1,
        },
        SpatialItem {
            bbox: BBox::new(5.0, 5.0, 15.0, 15.0), // partially outside
            data: 2,
        },
    ];
    let index = SpatialIndex::new(items);

    let query = BBox::new(0.0, 0.0, 10.0, 10.0);
    let results = index.query_contained(&query);
    assert_eq!(results.len(), 1, "Only fully contained items should match");
    assert_eq!(results[0].data, 1);
}

#[test]
fn test_spatial_index_many_items() {
    // Build a grid of items
    let items: Vec<SpatialItem<usize>> = (0..100)
        .map(|i| {
            let row = (i / 10) as f64;
            let col = (i % 10) as f64;
            SpatialItem {
                bbox: BBox::new(col * 20.0, row * 20.0, col * 20.0 + 15.0, row * 20.0 + 15.0),
                data: i,
            }
        })
        .collect();

    let index = SpatialIndex::new(items);
    assert_eq!(index.len(), 100);

    // Query a region that should cover 4 cells
    let query = BBox::new(25.0, 25.0, 55.0, 55.0);
    let results = index.query(&query);
    assert!(
        results.len() >= 2,
        "Should find at least 2 items, found {}",
        results.len()
    );
}

#[test]
fn test_spatial_index_query_exact_match() {
    let bbox = BBox::new(5.0, 5.0, 15.0, 15.0);
    let items = vec![SpatialItem {
        bbox,
        data: "exact",
    }];
    let index = SpatialIndex::new(items);

    let results = index.query(&bbox);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].data, "exact");
}
