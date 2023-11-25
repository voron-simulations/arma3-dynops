struct KdNode<T> {
    point: Vec<T>,
    left: Option<Box<KdNode<T>>>,
    right: Option<Box<KdNode<T>>>,
}

impl<T: PartialOrd + Copy> KdNode<T> {
    fn new(point: Vec<T>) -> Self {
        Self {
            point,
            left: None,
            right: None,
        }
    }

    fn insert(&mut self, point: Vec<T>, depth: usize) {
        let k = self.point.len();
        let axis = depth % k;

        if point[axis] < self.point[axis] {
            if let Some(left) = self.left.as_mut() {
                left.insert(point, depth + 1);
            } else {
                self.left = Some(Box::new(Self::new(point)));
            }
        } else if let Some(right) = self.right.as_mut() {
            right.insert(point, depth + 1);
        } else {
            self.right = Some(Box::new(Self::new(point)));
        }
    }

    fn search(&self, point: &[T], depth: usize) -> Option<&Vec<T>> {
        let k = self.point.len();
        let axis = depth % k;

        if self.point == point {
            return Some(&self.point);
        }

        if point[axis] < self.point[axis] {
            if let Some(left) = self.left.as_ref() {
                left.search(point, depth + 1)
            } else {
                None
            }
        } else if let Some(right) = self.right.as_ref() {
            right.search(point, depth + 1)
        } else {
            None
        }
    }
}

struct KdTree<T> {
    root: Option<Box<KdNode<T>>>,
}

impl<T: PartialOrd + Copy> KdTree<T> {
    fn new() -> Self {
        Self { root: None }
    }

    fn insert(&mut self, point: Vec<T>) {
        if let Some(root) = self.root.as_mut() {
            root.insert(point, 0);
        } else {
            self.root = Some(Box::new(KdNode::new(point)));
        }
    }

    fn search(&self, point: &[T]) -> Option<&Vec<T>> {
        if let Some(root) = self.root.as_ref() {
            root.search(point, 0)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut tree = KdTree::new();
        tree.insert(vec![1, 2]);
        tree.insert(vec![3, 4]);
        tree.insert(vec![5, 6]);
        tree.insert(vec![7, 8]);
        assert_eq!(tree.search(&[1, 2]), Some(&vec![1, 2]));
        assert_eq!(tree.search(&[3, 4]), Some(&vec![3, 4]));
        assert_eq!(tree.search(&[5, 6]), Some(&vec![5, 6]));
        assert_eq!(tree.search(&[7, 8]), Some(&vec![7, 8]));
    }

    #[test]
    fn test_search() {
        let mut tree = KdTree::new();
        tree.insert(vec![1, 2]);
        tree.insert(vec![3, 4]);
        tree.insert(vec![5, 6]);
        tree.insert(vec![7, 8]);
        assert_eq!(tree.search(&[1, 2]), Some(&vec![1, 2]));
        assert_eq!(tree.search(&[3, 4]), Some(&vec![3, 4]));
        assert_eq!(tree.search(&[5, 6]), Some(&vec![5, 6]));
        assert_eq!(tree.search(&[7, 8]), Some(&vec![7, 8]));
        assert_eq!(tree.search(&[9, 10]), None);
    }
}
