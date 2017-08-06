use geom::prim;
use geom::util;

use core;

use std;

/// Note: the implementation of a bounding-volume hierarchy in this file is taken from
/// PBRT, 3rd edition, section 4.3 (starting around page 256).

struct BvhComponentInfo {
    prim_index: usize,
    component_index: usize,
    bbox: core::BBox,
    centroid: core::Vec
}

impl BvhComponentInfo {
    pub fn from_prim_component(prim: usize, component: usize, bbox: core::BBox)
        -> BvhComponentInfo
    {
        let centroid = &(0.5 * &bbox.min) + &(0.5 * &bbox.max);
        BvhComponentInfo {
            prim_index: prim,
            component_index: component,
            bbox: bbox,
            centroid: centroid
        }
    }
}

#[derive(Clone, Copy)]
struct BucketInfo {
    count: usize,
    bbox: core::BBox
}

impl BucketInfo {
    pub fn empty() -> BucketInfo {
        BucketInfo {count: 0, bbox: core::BBox::empty()}
    }
}

struct BvhBuildNode {
    bbox: core::BBox,
    children: [usize; 2],
    split_axis: usize,
    first_component_offset: usize,
    num_components: usize
}

type BvhBuildNodeArena = std::vec::Vec<BvhBuildNode>;

impl BvhBuildNode {
    pub fn new_leaf(arena: &mut BvhBuildNodeArena, first: usize, n: usize, bbox: core::BBox)
        -> usize
    {
        arena.push(BvhBuildNode {
            bbox: bbox,
            children: [0, 0],
            split_axis: 0,
            first_component_offset: first,
            num_components: n
        });
        arena.len() - 1
    }

    pub fn new_interior(arena: &mut BvhBuildNodeArena, axis: usize, c0: usize, c1: usize)
        -> usize
    {
        let bbox = arena[c0].bbox.combine_with(&arena[c1].bbox);
        arena.push(BvhBuildNode {
            bbox: bbox,
            children: [c0, c1],
            split_axis: axis,
            first_component_offset: 0,
            num_components: 0
        });
        arena.len() - 1
    }
}

struct BvhLinearNode {
    bbox: core::BBox,
    offset: usize,
    num_components: usize,
    axis: usize
}

type BvhLinearNodeArena = std::vec::Vec<BvhLinearNode>;

impl BvhLinearNode {
    pub fn new(arena: &mut BvhLinearNodeArena) -> usize {
        arena.push(BvhLinearNode {
            bbox: core::BBox::empty(),
            offset: 0,
            num_components: 0,
            axis: 0
        });
        arena.len() - 1
    }
}

pub enum Intersection<'a> {
    Hit {
        dist: f32,
        normal: core::Vec,
        prim: &'a Box<prim::Prim>
    },
    NoHit
}

impl<'a> Intersection<'a> {
    pub fn hit(dist: f32, normal: core::Vec, prim: &'a Box<prim::Prim>)
        -> Intersection<'a>
    {
        Intersection::Hit {
            dist: dist,
            normal: normal,
            prim: prim
        }
    }

    pub fn no_hit() -> Intersection<'a> {
        Intersection::NoHit
    }
}

pub struct Bvh {
    prims: std::vec::Vec<Box<prim::Prim>>,
    components: std::vec::Vec<(usize, usize)>,
    nodes: BvhLinearNodeArena
}

impl Bvh {
    fn recurse_build(
        arena: &mut BvhBuildNodeArena,
        component_info: &mut [BvhComponentInfo],
        ordered_components: &mut std::vec::Vec<(usize, usize)>)
        -> usize
    {
        let mut bbox = core::BBox::empty();
        for ci in &component_info[..] {
            bbox = bbox.combine_with(&ci.bbox);
        }

        let num_components = component_info.len();
        if num_components == 1 {
            // Create a leaf node with one component.
            let first_component_offset = ordered_components.len();
            let ci = &component_info[0];
            ordered_components.push((ci.prim_index, ci.component_index));
            BvhBuildNode::new_leaf(arena, first_component_offset, 1, bbox)
        }
        else {
            // Partition the current node into two child subtrees.
            let mut centroid_bbox = core::BBox::empty();
            for ci in &component_info[..] {
                centroid_bbox = centroid_bbox.union_with(&ci.centroid);
            }
            let dim = centroid_bbox.maximum_extent();
            if centroid_bbox.min[dim] == centroid_bbox.max[dim] {
                // Cannot partition properly (components overlay one another).
                // Create a leaf node with multiple components.
                let first_component_offset = ordered_components.len();
                for ci in &component_info[..] {
                    ordered_components.push((ci.prim_index, ci.component_index));
                }
                BvhBuildNode::new_leaf(arena, first_component_offset, num_components, bbox)
            }
            else {
                let mid: usize;
                if num_components <= 4 {
                    // Partition into equally-sized subsets if too small to use SAH.
                    mid = num_components / 2;
                    util::nth_element(component_info, mid, &|lhs, rhs| {
                        lhs.centroid[dim] < rhs.centroid[dim]
                    });
                }
                else {
                    // Use the surface-area heuristic.
                    // Allocate BucketInfo for SAH partition buckets.
                    const NUM_BUCKETS: usize = 12;
                    let mut buckets = [BucketInfo::empty(); NUM_BUCKETS];

                    // Initialize BucketInfo for SAH partition buckets.
                    for ci in &component_info[..] {
                        let rel = centroid_bbox.relative_offset(&ci.centroid);
                        let b = core::clamp(
                                (NUM_BUCKETS as f32 * rel[dim]) as usize,
                                0, NUM_BUCKETS - 1);
                        buckets[b].count += 1;
                        buckets[b].bbox = buckets[b].bbox.combine_with(&ci.bbox);
                    }

                    // Compute costs for splitting after each bucket.
                    let mut cost = [0.0; NUM_BUCKETS - 1];
                    for i in 0..(NUM_BUCKETS - 1) {
                        // "Left" side of split.
                        let mut b0 = core::BBox::empty();
                        let mut count0 = 0;
                        for j in 0..(i + 1) {
                            b0 = b0.combine_with(&buckets[j].bbox);
                            count0 += buckets[j].count;
                        }

                        // "Right" side of split.
                        let mut b1 = core::BBox::empty();
                        let mut count1 = 0;
                        for j in (i + 1)..NUM_BUCKETS {
                            b1 = b1.combine_with(&buckets[j].bbox);
                            count1 += buckets[j].count;
                        }

                        cost[i] = 1.0 + (count0 as f32 * b0.surface_area()
                                + count1 as f32 * b1.surface_area()) / bbox.surface_area();
                    }

                    // Find bucket to split at that minimizes SAH metric.
                    let mut min_cost = cost[0];
                    let mut min_cost_split_bucket = 0;
                    for i in 1..(NUM_BUCKETS - 1) {
                        if cost[i] < min_cost {
                            min_cost = cost[i];
                            min_cost_split_bucket = i;
                        }
                    }

                    // Either create leaf or split primitives at selected SAH bucket.
                    // (Leaf might be cheaper.)
                    let leaf_cost = num_components as f32;

                    const MAX_COMPONENTS_PER_NODE: usize = 255;
                    if num_components > MAX_COMPONENTS_PER_NODE || min_cost < leaf_cost {
                        // Interior node.
                        mid = util::partition(component_info, &|ci| {
                            let rel = centroid_bbox.relative_offset(&ci.centroid);
                            let b = core::clamp(
                                    (NUM_BUCKETS as f32 * rel[dim]) as usize,
                                    0, NUM_BUCKETS - 1);
                            b <= min_cost_split_bucket
                        });
                    }
                    else {
                        // Leaf node.
                        mid = num_components / 2;
                        util::nth_element(component_info, mid, &|lhs, rhs| {
                            lhs.centroid[dim] < rhs.centroid[dim]
                        });
                    }
                }

                debug_assert!(mid > 0);
                debug_assert!(mid < num_components);
                let c0 = Bvh::recurse_build(arena, &mut component_info[0..mid],
                        ordered_components);
                let c1 = Bvh::recurse_build(arena, &mut component_info[mid..num_components],
                        ordered_components);
                BvhBuildNode::new_interior(arena, dim, c0, c1)
            }
        }
    }

    fn flatten_tree(
        arena: &BvhBuildNodeArena,
        nodes: &mut BvhLinearNodeArena,
        root: usize)
        -> usize
    {
        let build_node = &arena[root];
        let linear_node_index = BvhLinearNode::new(nodes);
        if build_node.num_components > 0 {
            // Leaf node.
            let linear_node = &mut nodes[linear_node_index];
            linear_node.bbox = build_node.bbox;
            linear_node.offset = build_node.first_component_offset;
            linear_node.num_components = build_node.num_components;
            linear_node_index
        }
        else {
            // Interior node.
            Bvh::flatten_tree(arena, nodes, build_node.children[0]);
            let second_child_offset = Bvh::flatten_tree(arena, nodes, build_node.children[1]);

            let linear_node = &mut nodes[linear_node_index];
            linear_node.bbox = build_node.bbox;
            linear_node.offset = second_child_offset;
            linear_node.axis = build_node.split_axis;
            linear_node_index
        }
    }

    pub fn build(prims: std::vec::Vec<Box<prim::Prim>>) -> Bvh {
        // Initialize BvhComponentInfo by scanning all prims for components.
        let mut component_info = std::vec::Vec::<BvhComponentInfo>::new();
        for prim_index in 0..prims.len() {
            let prim = &prims[prim_index];
            for component_index in 0..prim.num_components() {
                component_info.push(BvhComponentInfo::from_prim_component(
                    prim_index,
                    component_index,
                    prim.bbox_world(component_index)));
            }
        };

        // Build BVH tree for components from the BvhComponentInfo.
        // This will also create a lookup table of all components.
        let mut arena = BvhBuildNodeArena::with_capacity(
                1024 * 1024 / std::mem::size_of::<BvhBuildNode>()); // Roughly 1 MB allocation.
        let mut ordered_components = std::vec::Vec::<(usize, usize)>::with_capacity(
                component_info.len());
        let root = Bvh::recurse_build(&mut arena, &mut component_info, &mut ordered_components);

        // Compute representation of depth-first traversal of BVH tree.
        let mut nodes = BvhLinearNodeArena::with_capacity(arena.len());
        Bvh::flatten_tree(&arena, &mut nodes, root);

        Bvh {
            prims: prims,
            components: ordered_components,
            nodes: nodes
        }
    }

    // Returns the intersection of the ray with the components included in this Bvh.
    // NOTE: The ray should be unit-length to ensure that the right computation is provided,
    // although non-unit-length should work in theory if all the shapes are returning
    // parametric distances.
    pub fn intersect(&self, ray: &core::Ray) -> Intersection {
        let mut closest_dist = std::f32::MAX;
        let mut closest: Intersection = Intersection::no_hit();
        let isect_data = ray.compute_intersection_data();

        // Follow ray through BVH nodes to component intersections.
        let mut current_node_index = 0;
        let mut nodes_to_visit = Vec::<usize>::with_capacity(64);
        loop {
            let node = &self.nodes[current_node_index];

            // Check ray against BVH node.
            if node.bbox.intersect(&ray, &isect_data, closest_dist) {
                if node.num_components > 0 {
                    // Intersect ray with components in leaf.
                    for i in node.offset..(node.offset + node.num_components) {
                        let (prim_index, component_index) = self.components[i];
                        let prim = &self.prims[prim_index];
                        let (dist, normal) = prim.intersect_world(&ray, component_index);
                        if dist != 0.0 && dist < closest_dist {
                            closest = Intersection::hit(dist, normal, prim);
                            closest_dist = dist;
                        }
                    }
                    match nodes_to_visit.pop() {
                        Some(i) => current_node_index = i,
                        None => break
                    }
                }
                else {
                    // Put far BVH node on nodes_to_visit stack, advance to near node.
                    if isect_data.dir_is_neg[node.axis] {
                        nodes_to_visit.push(current_node_index + 1);
                        current_node_index = node.offset;
                    }
                    else {
                        nodes_to_visit.push(node.offset);
                        current_node_index = current_node_index + 1;
                    }
                }
            }
            else {
                match nodes_to_visit.pop() {
                    Some(i) => current_node_index = i,
                    None => break
                }
            }
        }

        closest
    }
}
