use crate::cluster::coord::HasPosition2d;
use std::marker::PhantomData;



fn build_kdtree<T: HasPosition2d>(items: &Vec<T>) -> Node<2,T>
{
    
    
}

struct Node<N,T> {
    pub item: Box<T>,
    pub left: Box<Node<N,T>>,
    pub right: Box<Node<N,T>>,

    __phantom: PhantomData<N>,
}