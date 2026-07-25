//! Structural diff iteration between copy-on-write trees.
//!
//! When a tree is derived from another via copy-on-write updates, unchanged subtrees are
//! shared by `Arc` pointer. Walking both trees in lockstep and skipping pointer-equal
//! subtrees visits only the changed elements, in O(changed) rather than O(len).
//!
//! Unrelated trees (no sharing) degrade gracefully to a full pairwise walk.
use crate::{Arc, Error, List, Tree, UpdateMap, Value};
use typenum::Unsigned;

impl<T: Value, N: Unsigned, U: UpdateMap<T>> List<T, N, U> {
    /// Visit `(index, base_value, new_value)` for every element of `self` that differs
    /// from `base`, including elements appended beyond `base`'s length (with `None` as
    /// the base value).
    ///
    /// Elements removed relative to `base` are NOT reported; callers that forbid
    /// deletions should check lengths themselves.
    ///
    /// Both lists must have no pending updates (see `apply_updates`).
    pub fn for_each_changed(
        &self,
        base: &Self,
        f: &mut impl FnMut(usize, Option<&T>, &T),
    ) -> Result<(), Error> {
        if self.interface.has_pending_updates() || base.interface.has_pending_updates() {
            return Err(Error::PendingUpdates);
        }
        let depth = self.interface.backing.depth + self.interface.backing.packing_depth;
        for_each_changed_node(
            Some(&base.interface.backing.tree),
            &self.interface.backing.tree,
            depth,
            0,
            f,
        );
        Ok(())
    }

    /// Copy all elements into a `Vec` by walking tree leaves directly.
    ///
    /// Equivalent to `to_vec` but faster for packed element types, as it avoids
    /// per-element iterator machinery. Falls back to `to_vec` if there are pending
    /// updates.
    pub fn to_vec_leaves(&self) -> Vec<T> {
        if self.interface.has_pending_updates() {
            return self.to_vec();
        }
        let mut out = Vec::with_capacity(self.len());
        collect_leaves(&self.interface.backing.tree, &mut out);
        out
    }
}

fn for_each_changed_node<T: Value>(
    base: Option<&Arc<Tree<T>>>,
    new: &Arc<Tree<T>>,
    depth: usize,
    offset: usize,
    f: &mut impl FnMut(usize, Option<&T>, &T),
) {
    if let Some(base_tree) = base {
        if Arc::ptr_eq(base_tree, new) {
            return;
        }
    }
    match new.as_ref() {
        Tree::Zero(_) => {}
        Tree::Node { left, right, .. } => {
            let (base_left, base_right) = match base.map(|b| b.as_ref()) {
                Some(Tree::Node { left, right, .. }) => (Some(left), Some(right)),
                // Zero base or no base: everything under `new` is fresh.
                _ => (None, None),
            };
            let half = 1usize << depth.saturating_sub(1);
            for_each_changed_node(base_left, left, depth.saturating_sub(1), offset, f);
            for_each_changed_node(
                base_right,
                right,
                depth.saturating_sub(1),
                offset + half,
                f,
            );
        }
        Tree::Leaf(leaf) => {
            let base_value = match base.map(|b| b.as_ref()) {
                Some(Tree::Leaf(base_leaf)) => Some(base_leaf.value.as_ref()),
                _ => None,
            };
            if base_value != Some(leaf.value.as_ref()) {
                f(offset, base_value, leaf.value.as_ref());
            }
        }
        Tree::PackedLeaf(packed) => {
            let base_values: &[T] = match base.map(|b| b.as_ref()) {
                Some(Tree::PackedLeaf(base_packed)) => &base_packed.values,
                _ => &[],
            };
            for (i, value) in packed.values.iter().enumerate() {
                let base_value = base_values.get(i);
                if base_value != Some(value) {
                    f(offset + i, base_value, value);
                }
            }
        }
    }
}

fn collect_leaves<T: Value>(tree: &Arc<Tree<T>>, out: &mut Vec<T>) {
    match tree.as_ref() {
        Tree::Zero(_) => {}
        Tree::Node { left, right, .. } => {
            collect_leaves(left, out);
            collect_leaves(right, out);
        }
        Tree::Leaf(leaf) => out.push(leaf.value.as_ref().clone()),
        Tree::PackedLeaf(packed) => out.extend_from_slice(&packed.values),
    }
}
