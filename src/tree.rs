use std::mem;

use crate::{
    error::InsertError,
    parser::{Segment, SegmentsIter},
    Params, SmallVec,
};

#[derive(Debug, Clone)]
struct StaticChildren<T> {
    indices: Vec<u8>,
    children: Vec<Node<T>>,
}

#[derive(Debug, Clone)]
struct Endpoint<T> {
    value: T,
    param_mapping: Vec<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct ParamNode<T> {
    endpoint: Option<Endpoint<T>>,
    child: Option<Box<Node<T>>>,
}

#[derive(Debug, Clone)]
pub struct CatchAllNode<T> {
    endpoint: Endpoint<T>,
}

#[derive(Debug, Clone)]
pub struct Node<T> {
    endpoint: Option<Endpoint<T>>,
    matching: Vec<u8>,
    static_children: StaticChildren<T>,
    param_child: Option<ParamNode<T>>,
    catch_all_child: Option<CatchAllNode<T>>,
    // regices: Vec<>
}

#[derive(Debug, Clone)]
pub struct Tree<T> {
    static_children: StaticChildren<T>,
}

impl<T> Default for Tree<T> {
    #[inline(always)]
    fn default() -> Self {
        Self {
            static_children: StaticChildren::new(),
        }
    }
}

impl<T> Endpoint<T> {
    fn remapping<'n, 'p>(
        &'n self,
        mut input: SmallVec<(&'n [u8], &'p [u8])>,
    ) -> SmallVec<(&'n [u8], &'p [u8])> {
        for (i, x) in input.iter_mut().enumerate() {
            x.0 = &self.param_mapping[i];
        }
        input
    }
}

impl<T> StaticChildren<T> {
    #[inline(always)]
    const fn new() -> Self {
        Self {
            indices: Vec::new(),
            children: Vec::new(),
        }
    }

    #[inline(always)]
    fn get(&self, byte: u8) -> Option<&Node<T>> {
        let idx = memchr::memchr(byte, &self.indices)?;
        Some(unsafe { self.children.get_unchecked(idx) })
    }

    /// # Safety
    /// `segment_path` can not be empty.
    #[inline(always)]
    unsafe fn get_mut_or_insert_unchecked(&mut self, segment_path: &[u8]) -> &mut Node<T> {
        let first = *segment_path.first().unwrap_unchecked();
        if let Some(idx) = memchr::memchr(first, &self.indices) {
            return self.children.get_unchecked_mut(idx);
        };
        self.insert_unchecked(first, Node::new(segment_path))
    }

    /// # Safety
    /// Must make sure that `byte` is not in `self.indices`.
    #[inline(always)]
    unsafe fn insert_unchecked(&mut self, byte: u8, node: Node<T>) -> &mut Node<T> {
        debug_assert_eq!(self.indices.len(), self.children.len());
        debug_assert!(!self.indices.contains(&byte));
        self.indices.push(byte);
        self.children.push(node);
        self.children.last_mut().unwrap_unchecked()
    }
}

impl<T> Node<T> {
    #[inline(always)]
    fn new(path: &[u8]) -> Self {
        Self {
            endpoint: None,
            matching: path.to_vec(),
            static_children: StaticChildren::new(),
            param_child: None,
            catch_all_child: None,
        }
    }

    pub fn at<'n, 'p>(&'n self, path: &'p [u8]) -> Option<(&'n T, Params<'n, 'p>)> {
        // Skipped saves the parent node's information.
        enum Skipped<'n, 'p, T> {
            Param {
                p_path: &'p [u8],
                p_node: &'n Node<T>,
                valid_p: usize,
            },
            CatchAll {
                f_path: &'p [u8],
                f_node: &'n Node<T>,
                valid_p: usize,
            },
        }

        let mut node = self;
        let mut params: SmallVec<(&[u8], &[u8])> = SmallVec::new();
        let mut path = path;

        type BigVec<S> = smallvec::SmallVec<[S; 8]>;
        let mut skipped = BigVec::<Skipped<'n, 'p, T>>::new();

        macro_rules! push_skipped_param {
            ($node:expr) => {
                skipped.push(Skipped::Param {
                    p_path: path,
                    p_node: $node,
                    valid_p: params.len(),
                });
            };
        }
        macro_rules! push_skipped_catch_all {
            ($node:expr) => {
                skipped.push(Skipped::CatchAll {
                    f_path: path,
                    f_node: $node,
                    valid_p: params.len(),
                });
            };
        }

        'main: loop {
            macro_rules! backtrack {
                () => {{
                    'bt: while let Some(skipped) = skipped.pop() {
                        match skipped {
                            Skipped::Param {
                                p_path,
                                p_node,
                                valid_p,
                            } => {
                                params.truncate(valid_p);
                                let (param_data, new_path) = next_param(p_path);
                                params.push((&[], param_data));
                                let pc = unsafe { p_node.param_child.as_ref().unwrap_unchecked() };
                                if new_path.is_empty() {
                                    if let Some(ep) = &pc.endpoint {
                                        return Some((&ep.value, ep.remapping(params)));
                                    }
                                    continue 'bt;
                                }
                                if let Some(pcc) = pc.child.as_ref() {
                                    node = pcc;
                                    path = new_path;
                                    continue 'main;
                                }
                                continue 'bt;
                            }
                            Skipped::CatchAll {
                                f_path,
                                f_node,
                                valid_p,
                            } => {
                                params.truncate(valid_p);
                                params.push((&[], f_path));
                                let node =
                                    unsafe { f_node.catch_all_child.as_ref().unwrap_unchecked() };
                                let endpoint = &node.endpoint;
                                return Some((&endpoint.value, endpoint.remapping(params)));
                            }
                        }
                    }
                    return None;
                }};
            }

            let path_len = path.len();
            let matching_len = node.matching.len();

            if path_len > matching_len {
                let (prefix, rest) = unsafe { path.split_at_unchecked(matching_len) };
                if prefix == node.matching {
                    path = rest;
                    // # Safety
                    // Path is longer than prefix, so after split, rest is not empty.
                    let first = unsafe { *rest.first().unwrap_unchecked() };
                    let Some(new_node) = node.static_children.get(first) else {
                        let param = match (&node.param_child, &node.catch_all_child) {
                            (None, None) => backtrack!(),
                            (None, Some(catch_all)) => {
                                // enter catch all
                                params.push((&[], rest));
                                return Some((
                                    &catch_all.endpoint.value,
                                    catch_all.endpoint.remapping(params),
                                ));
                            }
                            (Some(param), None) => param,
                            (Some(param), Some(_)) => {
                                // push skipped catch all and enter param
                                push_skipped_catch_all!(node);
                                param
                            }
                        };
                        // enter param
                        let (param_data, new_rest) = next_param(rest);
                        params.push((&[], param_data));

                        if new_rest.is_empty() {
                            if let Some(ep) = &param.endpoint {
                                return Some((&ep.value, ep.remapping(params)));
                            }
                        } else if let Some(pcc) = param.child.as_ref() {
                            node = pcc;
                            path = new_rest;
                            continue;
                        }
                        backtrack!();
                    };

                    // we found static child now
                    // push skipped nodes and enter next node
                    if node.catch_all_child.is_some() {
                        push_skipped_catch_all!(node);
                    }
                    if node.param_child.is_some() {
                        push_skipped_param!(node);
                    }
                    node = new_node;
                    continue;
                }
            }
            if path == node.matching {
                if let Some(endpoint) = &node.endpoint {
                    return Some((&endpoint.value, endpoint.remapping(params)));
                }
                if let Some(catch_all) = &node.catch_all_child {
                    params.push((&[], b""));
                    return Some((&catch_all.endpoint.value, catch_all.endpoint.remapping(params)));
                }
                backtrack!();
            }
            backtrack!();
        }
    }

    pub fn insert(&mut self, segments: SegmentsIter<'_>, value: T) -> Result<(), InsertError> {
        enum Status {
            Match,
            SkipMatching,
            Param,
        }

        let mut status = Status::Match;
        let mut node = self;
        let mut param_mapping = Vec::new();

        macro_rules! set_endpoint {
            ($endpoint:expr) => {
                if $endpoint.is_some() {
                    return Err(InsertError::new());
                }
                $endpoint = Some(Endpoint {
                    value,
                    param_mapping,
                })
            };
        }

        'main: for seg in segments {
            let seg = seg?;
            if let Some(name) = seg.name() {
                param_mapping.push(name.to_vec());
            }
            match seg {
                Segment::Static(mut path) => {
                    match status {
                        Status::SkipMatching => {
                            node =
                                unsafe { node.static_children.get_mut_or_insert_unchecked(path) };
                        }
                        Status::Param => {
                            let pcc = &mut node.param_child.as_mut().unwrap().child;
                            match pcc {
                                Some(inner) => node = inner,
                                None => {
                                    node = pcc.insert(Box::new(Node::new(path)));
                                    status = Status::SkipMatching;
                                    continue;
                                }
                            }
                        }
                        _ => (),
                    }

                    // loop to insert static path
                    loop {
                        let common_len = common_prefix(path, &node.matching);
                        if common_len < node.matching.len() {
                            // split node
                            unsafe {
                                let (common, rest) = node.matching.split_at_unchecked(common_len);
                                let first = *rest.first().unwrap_unchecked();
                                let rest = rest.to_vec();
                                let old_node = mem::replace(node, Node::new(common));
                                let old_node =
                                    node.static_children.insert_unchecked(first, old_node);
                                old_node.matching = rest;
                            }
                        }
                        if common_len == path.len() {
                            status = Status::SkipMatching;
                            continue 'main;
                        }

                        // insert new node
                        unsafe {
                            let (_, rest) = path.split_at_unchecked(common_len);
                            path = rest;
                            node = node.static_children.get_mut_or_insert_unchecked(path);
                            continue;
                        }
                    }
                }
                Segment::Param(_) => {
                    if matches!(status, Status::Param) {
                        return Err(InsertError::new());
                    }
                    if node.param_child.is_none() {
                        node.param_child = Some(ParamNode {
                            endpoint: None,
                            child: None,
                        });
                    }
                    status = Status::Param;
                    continue;
                }

                Segment::CatchAll(_) => {
                    if node.catch_all_child.is_some() {
                        return Err(InsertError::new());
                    }
                    node.catch_all_child = Some(CatchAllNode {
                        endpoint: Endpoint {
                            value,
                            param_mapping,
                        },
                    });
                    return Ok(());
                }
            }
        }
        // insert endpoint
        match status {
            Status::Match => {
                unreachable!()
            }
            Status::SkipMatching => {
                set_endpoint!(node.endpoint);
            }
            Status::Param => {
                let pc = node.param_child.as_mut().unwrap();
                set_endpoint!(pc.endpoint);
            }
        }
        Ok(())
    }
}

impl<T> Tree<T> {
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            static_children: StaticChildren::new(),
        }
    }

    #[inline]
    pub fn at<'n, 'p>(&'n self, path: &'p [u8]) -> Option<(&'n T, Params<'n, 'p>)> {
        let first = *path.first()?;
        self.static_children
            .get(first)
            .and_then(|node| node.at(path))
    }

    #[inline]
    pub fn insert(&mut self, route: &[u8], val: T) -> Result<(), InsertError> {
        let Some(Ok(Segment::Static(p))) = SegmentsIter::new(route).next() else {
            // The first segment must be static
            return Err(InsertError::new());
        };
        let child = unsafe { self.static_children.get_mut_or_insert_unchecked(p) };
        child.insert(SegmentsIter::new(route), val)
    }
}

#[inline(always)]
pub(crate) fn next_param(path: &[u8]) -> (&[u8], &[u8]) {
    if let Some(idx) = memchr::memchr(b'/', path) {
        return unsafe { path.split_at_unchecked(idx) };
    }
    (path, &[])
}

#[inline(always)]
fn common_prefix(x: &[u8], y: &[u8]) -> usize {
    // Borrowed from https://users.rust-lang.org/t/how-to-find-common-prefix-of-two-byte-slices-effectively/25815
    #[inline(always)]
    fn inner<const N: usize>(xs: &[u8], ys: &[u8]) -> usize {
        let off = std::iter::zip(xs.chunks_exact(N), ys.chunks_exact(N))
            .take_while(|(x, y)| x == y)
            .count()
            * N;
        off + std::iter::zip(&xs[off..], &ys[off..])
            .take_while(|(x, y)| x == y)
            .count()
    }
    inner::<128>(x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_at {
        ($tree: expr, $path:expr, $expected_val:expr, $expected_params:expr) => {{
            let (val, params) = $tree.at($path).unwrap();
            assert_eq!(val, &$expected_val);
            assert_eq!(params.len(), $expected_params.len());
            for (i, (name, value)) in $expected_params.iter().enumerate() {
                assert_eq!(&params[i].0, name);
                assert_eq!(&params[i].1, value);
            }
        }};
        ($tree: expr, $path:expr, $expected_val:expr) => {
            assert_eq!($tree.at($path).unwrap().0, &$expected_val);
        };
    }

    macro_rules! params {
        ($($name:expr => $value:expr),+) => {
            &[$(($name.as_slice(), $value.as_slice())),+]
        };
    }

    #[test]
    fn simple_static() {
        let mut tree = Tree::new();
        tree.insert(b"/a/b/c", 1).unwrap();

        assert_eq!(tree.at(b"/a/b/c").unwrap().0, &1);
    }

    #[test]
    fn simple_static2() {
        let mut tree = Tree::new();
        tree.insert(b"/a/b/c", 1).unwrap();
        assert!(tree.insert(b"/a/b/c", 1).is_err());
        tree.insert(b"/a/b/c/d", 2).unwrap();
        tree.insert(b"/a/b/d", 3).unwrap();
        tree.insert(b"/a/b", 4).unwrap();
        tree.insert(b"/q", 5).unwrap();

        assert_eq!(tree.at(b"/a/b/c").unwrap().0, &1);
        assert_eq!(tree.at(b"/a/b/c/d").unwrap().0, &2);
        assert_eq!(tree.at(b"/a/b/d").unwrap().0, &3);
        assert_eq!(tree.at(b"/a/b").unwrap().0, &4);
        assert_eq!(tree.at(b"/q").unwrap().0, &5);
    }

    #[test]
    fn simple_param() {
        let mut tree = Tree::new();
        tree.insert(b"/a/b/c", 1).unwrap();
        tree.insert(b"/a/b/:name", 2).unwrap();

        assert_eq!(tree.at(b"/a/b/c").unwrap().0, &1);
        assert_at!(tree, b"/a/b/d", 2, params!(b"name" => b"d"));
    }

    #[test]
    fn simple_param2() {
        let mut tree = Tree::new();
        tree.insert(b"/a/b/c", 1).unwrap();
        tree.insert(b"/a/b/:name/:id/X", 2).unwrap();
        tree.insert(b"/a/b/:nick/:country/Y", 3).unwrap();

        assert_eq!(tree.at(b"/a/b/c").unwrap().0, &1);
        assert_at!(
            tree,
            b"/a/b/chihai/ihciah/X",
            2,
            params!(b"name" => b"chihai", b"id" => b"ihciah")
        );
        assert_at!(
            tree,
            b"/a/b/ihc/CN/Y",
            3,
            params!(b"nick" => b"ihc", b"country" => b"CN")
        );
    }

    #[test]
    fn simple_catch_all() {
        let mut tree = Tree::new();
        tree.insert(b"/a/b/*any", 1).unwrap();

        assert_at!(
            tree,
            b"/a/b/chihai/ihciah/X",
            1,
            params!(b"any" => b"chihai/ihciah/X")
        );
    }

    #[test]
    fn simple_catch_all2() {
        let mut tree = Tree::new();
        tree.insert(b"/a/b/c/*any", 1).unwrap();
        tree.insert(b"/a/b/c/:name/X", 2).unwrap();
        tree.insert(b"/a/b/c/ihciah", 3).unwrap();
        tree.insert(b"/a/b/c/ihciah/X", 4).unwrap();

        tree.insert(b"/a/:name/c", 5).unwrap();
        tree.insert(b"/a/b/*path", 6).unwrap();

        assert_at!(tree, b"/a/b/c/ihciah", 3);
        assert_at!(tree, b"/a/b/c/ihciah/X", 4);
        assert_at!(tree, b"/a/b/c/chihai/X", 2, params!(b"name" => b"chihai"));
        assert_at!(tree, b"/a/b/c/chihai/Y", 1, params!(b"any" => b"chihai/Y"));

        assert_at!(tree, b"/a/b/c", 6, params!(b"path" => b"c"));
        assert_at!(tree, b"/a/x/c", 5, params!(b"name" => b"x"));
    }

    #[test]
    fn catch_all_with_single_slash() {
        let mut tree = Tree::new();
        tree.insert(b"/*path", 1).unwrap();
    
        // 测试单个斜杆的情况
        assert_at!(
            tree,
            b"/",
            1,
            params!(b"path" => b"")
        );
    
        // 测试正常路径的情况，确保不影响其他匹配
        assert_at!(
            tree,
            b"/users",
            1,
            params!(b"path" => b"users")
        );
    
        // 测试多个斜杆的情况
        assert_at!(
            tree,
            b"/users/123",
            1,
            params!(b"path" => b"users/123")
        );
    }
}
