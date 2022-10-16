use crate::types::HasPosition;

const dimension: usize = 2;

pub struct KdTree<T> {
    arena: Vec<KdTreeNode<T>>,
}

pub fn construct_kdtree<T: HasPosition + Copy>(items: &Vec<T>) -> KdTree<T> {
    let mut tree = KdTree { arena: Vec::new() };
    tree.arena.reserve(items.len());
    fill_kdtree(items, &mut tree.arena, 0);
    tree
}

fn fill_kdtree<T: HasPosition + Copy>(
    items: &[T],
    arena: &mut Vec<KdTreeNode<T>>,
    depth: usize,
) -> Option<usize> {
    if items.len() == 0 {
        return None;
    }

    let axis = depth % dimension;

    let mut items_copy = items.to_vec();
    items_copy.sort_by(|a, b| {
        let apos = a.get_position();
        let bpos = b.get_position();
        (apos[axis]).partial_cmp(&bpos[axis]).unwrap()
    });

    let middle = items_copy.len() / 2;
    let items_left = &items_copy[..middle];
    let items_right = &items_copy[middle..];

    let node = KdTreeNode {
        item: items_copy[middle],
        left: fill_kdtree(items_left, arena, depth + 1),
        right: fill_kdtree(items_right, arena, depth + 1),
    };
    arena.push(node);
    Some(arena.len())
}

#[derive(Default)]
struct KdTreeNode<T> {
    pub item: T,
    pub left: Option<usize>,  // Index of right node in arena
    pub right: Option<usize>, // Index of left node in arena
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_tree() {
        // let tree = construct_kdtree();
    }
}
