/*
 *
 * Octree and Nearest Neighbor (nn) Implementation
 * 
 */

// Max point for octree and nn list. Should be changed to be a dynamic size later
const MAX_POINTS: usize = 1000;

#[derive(Resource, Default)]
struct NearestNeightborList {
    // list: [Particle; MAX_POINTS]   // Figure out how to satisfy the derived default trait for the list
    list: Vec<Particle>, // Playing with Vecs for now
}

// Fill up the octree with particles from the environment
fn populate_octree(
    mut octree: ResMut<Octree>,
    particle_query: Query<(&mut Particle, &Transform, Entity)>,
) {
    let mut new_octree = Octree::new(10);

    for (_particle, transform, entity) in &particle_query {
        let point = Point3D {
            x: transform.translation.x,
            y: transform.translation.y,
            z: transform.translation.z,
            entity,
        };
        new_octree.insert(point);
    }

    *octree = new_octree;
    //somehow create nearest neighbors list global variable here
}

// Particle Point that holds the xyz location alongside the particle ID
#[derive(Clone, Copy)]
struct Point3D {
    x: f32,
    y: f32,
    z: f32,
    entity: Entity,
}

// An octree node could be either a branch (another octree) or it could be a leaf (which holds the particle data)
enum OctreeNode {
    Branch([Option<Box<OctreeNode>>; 8]),
    Leaf(Vec<Point3D>),
}

// Octree Stucture
#[derive(Resource)]
struct Octree {
    root: OctreeNode,
    max_depth: usize,
    length: f32,
    center: Vec3,
}

fn initialize_octree(mut commands: Commands) {
    commands.insert_resource(Octree::new(10));
}

// Functions for OctreeNode
// Insert(): Insert a point/particle into the octree
// get_point_by_index(): Given an idex, return all the branches in the node
impl OctreeNode {
    // Insert a new point into the octree
    fn insert(
        &mut self,
        point: Point3D,
        depth: usize,
        max_depth: usize,
        length: f32,
        center: Vec3,
    ) {
        // Checks to see if the current node is a branch node or a leaf
        match self {
            // Handles the case when the current node is a branch node
            // Perform checks and insert when the current node is a branch
            OctreeNode::Branch(children) => {
                let mut new_center = center;

                let x = point.x;
                let y = point.y;
                let z = point.z;

                let mut index = 0;

                // Checks to see which octant to put the new point into
                if x > center.x {
                    index |= 1;
                    new_center.x += length;
                } else {
                    new_center.x -= length;
                }
                if y > center.y {
                    index |= 2;
                    new_center.y += length;
                } else {
                    new_center.y -= length;
                }
                if z > center.z {
                    index |= 4;
                    new_center.z += length;
                } else {
                    new_center.z -= length;
                }

                // Check to see if there is a child node and create one if there isn't
                if children[index].is_none() {
                    //Is this where we give the entity and store it?
                    children[index] = Some(Box::new(OctreeNode::Leaf(Vec::new())));
                }

                // If there is a child node, then get the branch and insert the new point into the branch
                if let Some(mut child) = children[index].take() {
                    child.insert(point, depth + 1, max_depth, length / 2., new_center);
                    children[index] = Some(child);
                }
            }
            // Handles the case when the current node is a leaf node
            OctreeNode::Leaf(points) => {
                points.push(point);
                // If pushing the new point into the leaf node causes a size greater than 8 and max depth hasn't been hit,
                // Split the leaf into 8 smaller leaf nodes
                if points.len() > 8 && depth < max_depth {
                    let mut children = [None, None, None, None, None, None, None, None];
                    // Checks to see which octant to put the new point into
                    for point in points.drain(..) {
                        let mut new_center = center;

                        let x = point.x;
                        let y = point.y;
                        let z = point.z;

                        let mut index = 0;

                        // Checks to see which octant to put the new point into
                        if x > center.x {
                            index |= 1;
                            new_center.x += length;
                        } else {
                            new_center.x -= length;
                        }
                        if y > center.y {
                            index |= 2;
                            new_center.y += length;
                        } else {
                            new_center.y -= length;
                        }
                        if z > center.z {
                            index |= 4;
                            new_center.z += length;
                        } else {
                            new_center.z -= length;
                        }

                        // Check to see if there is a child node and create one if there isn't
                        if children[index].is_none() {
                            children[index] = Some(Box::new(OctreeNode::Leaf(Vec::new())));
                        }
                        // If there is a child node, then get the branch and insert the new point into the branch
                        if let Some(mut child) = children[index].take() {
                            child.insert(point, depth + 1, max_depth, length / 2., new_center);
                            children[index] = Some(child);
                        }
                    }
                    // Once leaf node is split into more nodes, change itself into a branch
                    *self = OctreeNode::Branch(children);
                }
            }
        }
    }

    // Given an idex, return all the branches in the node
    fn get_points_by_index() {
        //return all of the points in a single leaf
        //recursive, possibly return empty list of neighbors when next would be out of bounds
    }
}

// Function for Octree
// new(): Initiate the tree
// nearest_neighbor_list(): Create a list that is comprised of tuples of points that are connected to eachother
// insert(): Insert a point into the current octree
// tranverse(): Traverse the tree to print out all the points in it
// search(): Given a point, radius, and octree, search for particles within the radius
// search_recursive(): Helper function for search()
impl Octree {
    // Create a new and empty octree object
    fn new(max_depth: usize) -> Self {
        Octree {
            root: OctreeNode::Leaf(Vec::new()),
            max_depth: max_depth,
            length: 400.,
            center: Vec3::new(0., 0., 0.),
        }
    }

    // Create a list that is comprised of tuples of points that are connected to eachother
    fn nearest_neighbor_list(&self) {
        // Perform bitwise operation to get neighboring cells (size of particle)
        neighbors_offset_list();
        neighbors_list = Vec::new();

        //indexs is a list of byte indices for each leaf
        for index in indexs {
            //get points by index
            points = get_points_by_index(index);
            for (i, point) in points.enumerate() {
                //start_at might not be correct, but start at index i
                for p in points.start_at(i) {
                    neighbor_list.push(point, p);
                }
            }

            // Push neighboring particles into list (offseted amount, the 7 neighboring cells)
            // TODO: Handle out of bounds neighbor
            for neighbor_offset in neighbor_offset_list {
                neighbor_index = neighbor_offset + index;
                neighbor_points = get_points_by_index(neighbor_index);
                for point in points {
                    for neighbor_point in neighbor_points {
                        neighbor_list.push((point, neighbor_point));
                    }
                }
            }
        }
        return neighbor_list;
    }
    // Insert a point into the current octree
    fn insert(&mut self, point: Point3D) {
        self.root
            .insert(point, 0, self.max_depth, self.length, self.center);
    }

    // Traverse the tree to print out all the points in it
    fn traverse(&self, node: &OctreeNode) {
        // Check to see if current node is a branch or leaf
        match node {
            // For branches, recursively call the traverse function on all children that isn't none
            OctreeNode::Branch(children) => {
                for child in children.iter() {
                    if let Some(child) = child {
                        self.traverse(child);
                    }
                }
            }
            // For leafs, iterate through all the points in the leaf and print out the coordinate
            OctreeNode::Leaf(points) => {
                for point in points.iter() {
                    println!("Point: ({}, {}, {})", point.x, point.y, point.z);
                }
            }
        }
    }

    // Given a point, radius, and octree, search for particles within the radius
    fn search(&self, center: Point3D, radius: f32) -> Vec<Point3D> {
        let mut points = Vec::new();
        self.search_recursive(&self.root, &center, radius, &mut points);
        return points;
    }

    // Helper function for recursively searching the tree
    fn search_recursive(
        &self,
        node: &OctreeNode,
        center: &Point3D,
        radius: f32,
        points: &mut Vec<Point3D>,
    ) {
        match node {
            // For branches, recursively call the search function on all children that isn't none
            OctreeNode::Branch(children) => {
                for child in children.iter() {
                    if let Some(child) = child {
                        self.search_recursive(child, center, radius, points);
                    }
                }
            }
            // For leafs, iterate through all the points in the leaf and check if they are within the given radius from the center point
            OctreeNode::Leaf(child_points) => {
                for point in child_points.iter() {
                    let dist_x = center.x - point.x;
                    let dist_y = center.y - point.y;
                    let dist_z = center.z - point.z;
                    let dist_sq = dist_x.powi(2) + dist_y.powi(2) + dist_z.powi(2);
                    if dist_sq <= radius.powi(2) {
                        // Add point to the list of found points
                        points.push(*point);
                    }
                }
            }
        }
    }
}
