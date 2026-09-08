//! The Burkhard-Keller tree is a discrete metric tree data structure designed for fast, approximate string matching
//! and fuzzy searching.
//!
//! This module holds all relevant structs and functions to make a BK-tree work.
//!
//! BK-trees live in a discrete metric space,
//! and this implementation require distances to be of the Natural Numbers, incl. 0,
//! the values an unsigned integer, like [`usize`], can hold[^impl].
//! They use discrete distance metrics like the [`levenshtein`] edit distance formula.
//!
//! [^impl]: See [`DistanceMetric`] for more information.
//!
//! # Examples
//!
//! Create a simple [`BkTree`] using [`levenshtein`] distance.
//!
//! ```rust
//! # use tobys_lib::trees::bk_tree::*;
//! let mut tree = BkTree::new(levenshtein);
//! tree.extend(vec!["She", "sells", "sea", "shells"]);
//! # assert_eq!(tree.find_best(&"she"), Some((1, &"She")));
//! ```
//!
//! Create a [`BkTree`] using a custom distance formula.
//!
//! ```rust
//! # use tobys_lib::trees::bk_tree::*;
//! fn dist(a: &usize, b: &usize) -> usize { a.abs_diff(*b) }
//! let tree = BkTree::new_with_items(dist, vec![2, 3, 5, 7, 11, 13, 17, 19]);
//! assert_eq!(tree.find_best(&16), Some((1, &17)));
//! ```
//!
//!
//! # Creator Appreciation
//!
//! This data structure was suggested by Walter A. Burkhard and Robert M. Keller in 1973, in their article
//! "Some approaches to best-match file searching".
//! Their paper is available [here](https://dl.acm.org/doi/10.1145/362003.362025).

use core::{
    fmt,
    iter::{ExactSizeIterator, Extend, IntoIterator},
};

use crate::alias::{Box, Entry, HashMap, VecDeque};

/// The Levenshtein distance, or edit distance, is the minimum amount of edits
/// to turn string `a` into string `b`.
///
/// This formula is flattened to use only `θ(m)` space, `m` being the length of the string `b`.
///
/// # Panics
///
/// This uses the [`char_indices`] method to go over all characters in the string.
/// To make sure a single character change is the same as what a human would perceive it to be,
/// all characters must be valid ASCII, and not just UTF-8.
///
/// [`char_indices`]: str::char_indices
#[expect(clippy::arithmetic_side_effects, clippy::indexing_slicing)]
pub fn levenshtein<T: AsRef<str>>(a: &T, b: &T) -> usize {
    let (a_chars, b_chars) = (
        a.as_ref().char_indices().collect::<Vec<_>>(),
        b.as_ref().char_indices().collect::<Vec<_>>(),
    );
    if a_chars == b_chars {
        return 0;
    }

    let mut prev: Vec<_> = (0..=b_chars.len()).collect();

    for i in 1..=a_chars.len() {
        let mut curr = vec![0; b_chars.len() + 1];
        curr[0] = i;

        for j in 1..=b_chars.len() {
            assert!(
                a_chars[i - 1].1.is_ascii(),
                "Char '{}' (pos: {}) is not ASCII in string '{}'",
                a_chars[i - 1].1,
                a_chars[i - 1].0,
                a.as_ref()
            );
            assert!(
                b_chars[j - 1].1.is_ascii(),
                "Char '{}' (pos: {}) is not ASCII in string '{}'",
                b_chars[j - 1].1,
                b_chars[j - 1].0,
                b.as_ref()
            );

            let d1 = prev[j] + 1; // deletion
            let d2 = curr[j - 1] + 1; // insertion
            let mut d3 = prev[j - 1]; // substitution
            if a_chars[i - 1].1 != b_chars[j - 1].1 {
                d3 += 1; // substitution cost
            }
            curr[j] = d1.min(d2.min(d3));
        }

        prev = curr;
    }

    prev[b_chars.len()]
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct Node<T> {
    elem: T,
    children: HashMap<usize, Node<T>>,
}

impl<T> Node<T> {
    pub fn new(elem: T) -> Self {
        Self {
            elem,
            children: HashMap::new(),
        }
    }

    fn get_root_and_list<I: IntoIterator<Item = T>>(
        dist: &impl Fn(&T, &T) -> usize,
        items: I,
    ) -> Option<(T, HashMap<usize, Vec<T>>)> {
        // TODO?; make it choose the one that has the lowest std. dev.?
        let mut items = items.into_iter();
        let mut res: HashMap<usize, Vec<T>> = HashMap::new();
        let root = items.next()?;

        for item in items {
            // TODO; enable rayon parallelization
            res.entry((dist)(&item, &root)).or_default().push(item);
        }

        Some((root, res))
    }

    pub fn build_node<I: IntoIterator<Item = T>>(
        dist: &impl Fn(&T, &T) -> usize,
        items: I,
    ) -> Option<Self> {
        let (root, res) = Self::get_root_and_list(dist, items)?;

        let mut children = HashMap::with_capacity(res.len());
        for (d, list) in res {
            if !list.is_empty() {
                children.insert(d, Node::build_node(dist, list)?);
            }
        }

        Some(Self {
            elem: root,
            children,
        })
    }

    pub fn find_best<'a>(
        &'a self,
        query: &T,
        dist: impl Fn(&T, &T) -> usize,
    ) -> (usize, &'a T) {
        let mut best = &self.elem;
        let mut best_dist = dist(query, best);

        let mut stack = VecDeque::new();
        stack.push_back(self);

        while let Some(node) = stack.pop_back() {
            let j = dist(query, &node.elem);
            if j == 0 {
                return (0, &node.elem);
            }
            if j < best_dist {
                (best, best_dist) = (&node.elem, j);
            }
            stack.extend(
                node.children.iter().filter_map(|(k, v)| {
                    (k.abs_diff(j) < best_dist).then_some(v)
                }),
            );
        }

        (best_dist, best)
    }

    pub fn len(&self) -> usize {
        self.children
            .values()
            .map(Node::len)
            .fold(1, usize::saturating_add)
        // starts at 1, because count this node as well
    }
}

/// This is the trait that any distance metric function must implement.
///
/// The trait is automatically implemented for all defined functions that are
/// ```rust
/// fn dist<T>(a: &T, b: &T) -> usize
/// # { 0 }
/// ```
/// or where `T` is a specific type, like
/// ```rust
/// # #[expect(clippy::trivially_copy_pass_by_ref)]
/// fn dist(a: &usize, b: &usize) -> usize
/// # { a.abs_diff(*b) }
/// ```
///
/// # Implementation requirements
///
/// Even if the trait is automatically implemented for most functions,
/// for it to be a valid implementation the following list of properties
/// must be guaranteed by the user.
///
/// - The return must only depend on the inputs[^pure].
/// - `dist(a, b) = dist(b, a)` (the function is commutative)
/// - `dist(a, b) = 0 => a = b` (0 distance means equality)
/// - it must satisfy the triangle inequality;
///   for every `a, b, c`: `dist(a, c) <= dist(a, b) + dist(b, c)`
///
/// If any are broken, the [`BkTree`] is likely to fail or even panic.
///
/// [^pure]: The function does not need to be pure, but it is recommended to keep it pure.
/// If side effects happen, they must not have any effect on the returned value. (e.g. logging is fine)
pub trait DistanceMetric<T>: Fn(&T, &T) -> usize + Clone + Copy {}

impl<T, F> DistanceMetric<T> for F where F: Fn(&T, &T) -> usize + Clone + Copy {}
/// The Burkhard-Keller tree is a metric tree data structure designed for fast, approximate string matching
/// and fuzzy searching.
///
/// [`serde`] serialization can be enabled with `serde` feature flag,
/// however it is currently only implemented with the [`levenshtein`] distance function.
///
/// # Examples
///
/// Create a simple [`BkTree`] using [`levenshtein`] distance.
///
/// ```rust
/// # use tobys_lib::trees::bk_tree::*;
/// let mut tree = BkTree::new(levenshtein);
/// tree.extend(vec!["She", "sells", "sea", "shells"]);
/// # assert_eq!(tree.find_best(&"she"), Some((1, &"She")));
/// ```
///
/// Create a [`BkTree`] using a custom distance formula.
///
/// ```rust
/// # use tobys_lib::trees::bk_tree::*;
/// fn dist(a: &usize, b: &usize) -> usize { a.abs_diff(*b) }
/// let tree = BkTree::new_with_items(dist, vec![2, 3, 5, 7, 11, 13, 17, 19]);
/// assert_eq!(tree.find_best(&16), Some((1, &17)));
/// ```
#[must_use = "a bk-tree does nothing on its own"]
pub struct BkTree<T, F: DistanceMetric<T>> {
    root: Option<Node<T>>,
    dist: Box<F>,
}

impl<T, F: DistanceMetric<T>> BkTree<T, F> {
    /// Create a BK-tree with the specified distance metric.
    ///
    /// It is recommended to use [`BkTree::new_with_items`] instead of this function,
    /// as that function has some—and may gain more—inbuilt optimizations that will give a faster
    /// practical speed than manually inserting all of them.
    ///
    /// # Distance metric
    ///
    /// Please see [`DistanceMetric`] for how to make a custom distance metric function.
    /// # Example
    ///
    /// Create a simple [`BkTree`] using [`levenshtein`] distance.
    /// ```rust
    /// # use tobys_lib::trees::bk_tree::*;
    /// let mut tree = BkTree::new(levenshtein);
    /// tree.extend(vec!["She", "sells", "sea", "shells"]);
    /// # assert_eq!(tree.find_best(&"she"), Some((1, &"She")));
    /// ```
    pub fn new(dist: F) -> Self {
        Self {
            root: None,
            dist: Box::new(dist),
        }
    }

    /// Create a BK-tree with the specified distance metric, and adds the given items.
    ///
    /// This function may take longer on the initial setup, as it is set to run some optimizations.
    /// However, these will improve the practical speed during lookup.
    ///
    /// # Distance metric
    ///
    /// The distance metric must have the following properties;
    /// - No distance is negative (output is [usize]).
    /// - `dist(a,b) = dist(b,a)` (the function is commutative)
    /// - The return must only depend on the inputs[^pure].
    ///
    /// [^pure]: The function does not need to be pure, but it is recommended to keep it pure.
    ///
    /// # Example
    ///
    /// Create a simple [`BkTree`] using [`levenshtein`] distance.
    /// ```rust
    /// # use tobys_lib::trees::bk_tree::*;
    /// let tree = BkTree::new_with_items(levenshtein, vec!["She", "sells", "sea", "shells"]);
    /// # assert_eq!(tree.find_best(&"she"), Some((1, &"She")));
    /// ```
    pub fn new_with_items<I: IntoIterator<Item = T>>(
        dist: F,
        items: I,
    ) -> Self {
        Self {
            root: Node::build_node(&dist, items),
            dist: Box::new(dist),
        }
    }

    /// Returns the amount of items in the [`BkTree`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # use tobys_lib::trees::bk_tree::*;
    /// let mut tree = BkTree::new(levenshtein);
    /// tree.insert("She");
    /// tree.insert("sells");
    /// tree.insert("sea");
    /// tree.insert("shells");
    /// # assert_eq!(tree.len(), 4);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        match &self.root {
            Some(n) => n.len(),
            None => 0,
        }
    }

    /// Indicates whether [`BkTree`] is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use tobys_lib::trees::bk_tree::*;
    /// let mut tree = BkTree::new(levenshtein);
    /// tree.insert("She");
    /// assert!(!tree.is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterate over references of the items in this [`BkTree`], in no particular order.
    pub fn iter(&self) -> Iter<'_, T> {
        let Some(root) = &self.root else {
            return Iter::default();
        };

        let len = self.len();
        let mut queue = Vec::with_capacity(len);
        queue.push(root);

        Iter { queue, len }
    }

    /// Inserts the given element into this [`BkTree`].
    ///
    /// Duplicates are silently dropped.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use tobys_lib::trees::bk_tree::*;
    /// let mut tree = BkTree::new(levenshtein);
    /// tree.insert("She");
    /// tree.insert("sells");
    /// tree.insert("sea");
    /// tree.insert("shells");
    /// # assert_eq!(tree.find(&"sells", 3).len(), 3);
    /// ```
    #[expect(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub fn insert(&mut self, elem: T) {
        match self.root {
            None => {
                self.root = Some(Node::new(elem));
            }
            Some(ref mut root_node) => {
                let mut n = root_node;
                loop {
                    let d = (self.dist)(&n.elem, &elem);
                    if d == 0 {
                        return;
                    }

                    if let Entry::Vacant(e) = n.children.entry(d) {
                        e.insert(Node::new(elem));
                        return;
                    }

                    n = n.children.get_mut(&d).unwrap();
                }
            }
        }
    }

    /// Inserts all the elements in the iterator into this [`BkTree`].
    ///
    /// The same as [`Extend`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # use tobys_lib::trees::bk_tree::*;
    /// let mut tree = BkTree::new(levenshtein);
    /// tree.insert_all(vec!["She", "sells", "sea", "shells"]);
    /// tree.extend(vec!["by", "the", "seashore"]);
    /// assert_eq!(tree.find(&"sells", 3).len(), 3);
    /// ```
    #[inline]
    pub fn insert_all<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for elem in iter {
            self.insert(elem);
        }
    }

    /// Finds all elements that satisfies `dist(query, x) <= max_dist`.
    ///
    /// The returned value is a [`Vec`] of references to items, that satisfy prior inequality,
    /// alongside their distance, as reported by the given [`DistanceMetric`] function, to the `query`.
    /// The order of the results is not stable.
    ///
    /// If you wish to only find the best match, see [`find_best`] method,
    /// as that may find it quicker than having a small `max_dist` and searching
    /// for the minimum in the given result of this method.
    ///
    /// [`find_best`]: Self::find_best
    ///
    /// # Example
    ///
    /// ```rust
    /// # use tobys_lib::trees::bk_tree::*;
    /// let tree = BkTree::new_with_items(levenshtein, vec!["She", "sells", "sea", "shells"]);
    /// assert_eq!(tree.find(&"sells", 3).len(), 3);
    /// ```
    pub fn find(&self, query: &T, max_dist: usize) -> Vec<(usize, &T)> {
        let Some(root) = &self.root else {
            return Vec::new();
        };

        let mut found = Vec::new();
        let mut candidates = VecDeque::new();
        candidates.push_back(root);

        while let Some(n) = candidates.pop_front() {
            let d = (self.dist)(&n.elem, query);
            if d <= max_dist {
                found.push((d, &n.elem));
            }

            candidates.extend(
                n.children
                    .iter()
                    .filter(|(d1, _)| {
                        d1.abs_diff(d) <= max_dist // |d1 - d| <= max_dist
                    })
                    .map(|(_, n)| n),
            );
        }

        found
    }

    /// Finds a best closest match to the `query` in this [`BkTree`].
    ///
    /// If multiple values have the same distance, the returned value may be *any* of them.
    ///
    /// This is often more performant than running [`find`] with a small max distance
    /// and looking for the smallest distance.
    ///
    /// [`find`]: Self::find
    ///
    /// # Example
    ///
    /// ```rust
    /// # use tobys_lib::trees::bk_tree::*;
    /// let tree = BkTree::new_with_items(levenshtein, vec!["She", "sells", "sea", "shells"]);
    /// assert_eq!(tree.find_best(&"she"), Some((1, &"She")));
    /// ```
    pub fn find_best<'a>(&'a self, query: &T) -> Option<(usize, &'a T)> {
        self.root
            .as_ref()
            .map(|root| root.find_best(query, &self.dist))
    }

    /// Indicates whether this [`BkTree`] contains the given `query`.
    ///
    /// This operation is on average `O(log n)`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use tobys_lib::trees::bk_tree::*;
    /// let tree = BkTree::new_with_items(levenshtein, vec!["She", "sells", "sea", "shells"]);
    /// assert!(tree.contains(&"sea"));
    /// ```
    pub fn contains(&self, query: &T) -> bool {
        !self.find(query, 0).is_empty()
    }
}

impl<T: fmt::Debug, F: DistanceMetric<T>> fmt::Debug for BkTree<T, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.root {
            Some(v) => f.write_str(&format!("{v:?}")),
            None => f.write_str("[]"),
        }
    }
}
impl<T: AsRef<str>> Default for BkTree<T, fn(&T, &T) -> usize> {
    fn default() -> Self {
        Self::new(levenshtein::<T>)
    }
}
impl<T, F: DistanceMetric<T>> Extend<T> for BkTree<T, F> {
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.insert_all(iter);
    }
}
impl<T: AsRef<str>> FromIterator<T> for BkTree<T, fn(&T, &T) -> usize> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::new_with_items(levenshtein, iter)
    }
}
impl<T, F: DistanceMetric<T>> IntoIterator for BkTree<T, F> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let len = self.len();
        let Some(root) = self.root else {
            return IntoIter::default();
        };

        let mut queue = Vec::with_capacity(len / 2);
        queue.push(root);
        IntoIter { queue, len }
    }
}
impl<'a, T, F: DistanceMetric<T>> IntoIterator for &'a BkTree<T, F> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<T: PartialEq, F: DistanceMetric<T>> PartialEq for BkTree<T, F> {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}
impl<T: Eq, F: DistanceMetric<T>> Eq for BkTree<T, F> {}
#[cfg(feature = "serde")]
impl<T, F> serde::Serialize for BkTree<T, F>
where
    T: serde::Serialize,
    F: DistanceMetric<T>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("BkTree", 1)?;
        s.serialize_field("root", &self.root)?;
        s.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, T> serde::Deserialize<'de> for BkTree<T, fn(&T, &T) -> usize>
where
    T: AsRef<str> + serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use core::marker::PhantomData;

        use serde::{
            Deserialize,
            de::{self, MapAccess, Visitor},
        };

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Root,
        }

        struct BkTreeVisitor<T>(PhantomData<T>);

        impl<'de, T: AsRef<str> + Deserialize<'de>> Visitor<'de> for BkTreeVisitor<T> {
            type Value = BkTree<T, fn(&T, &T) -> usize>;

            fn expecting(
                &self,
                formatter: &mut fmt::Formatter<'_>,
            ) -> fmt::Result {
                formatter.write_str("struct BkTree")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut root = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Root => {
                            if root.is_some() {
                                return Err(de::Error::duplicate_field("root"));
                            }
                            root = Some(map.next_value()?);
                        }
                    }
                }
                let root =
                    root.ok_or_else(|| de::Error::missing_field("root"))?;
                Ok(BkTree {
                    root,
                    dist: Box::new(levenshtein),
                })
            }
        }

        const FIELDS: &[&str] = &["root"];
        deserializer.deserialize_struct(
            "BkTree",
            FIELDS,
            BkTreeVisitor::<T>(PhantomData),
        )
    }
}

/// Iterator over a [`BkTree`]'s values.
#[must_use = "iterators are—by design—lazy and must be used"]
#[derive(Debug, Clone)]
pub struct IntoIter<T> {
    queue: Vec<Node<T>>,
    len: usize,
}
impl<T> Default for IntoIter<T> {
    fn default() -> Self {
        Self {
            queue: Vec::default(),
            len: usize::default(),
        }
    }
}
impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.queue.pop().map(|n| {
            self.len = self.len.saturating_sub(1);
            self.queue.extend(n.children.into_values());
            n.elem
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}
impl<T> ExactSizeIterator for IntoIter<T> {}

/// Iterator over references of a [`BkTree`]'s values.
#[must_use = "iterators are—by design—lazy and must be used"]
#[derive(Debug, Clone)]
pub struct Iter<'a, T> {
    queue: Vec<&'a Node<T>>,
    len: usize,
}
impl<T> Default for Iter<'_, T> {
    fn default() -> Self {
        Self {
            queue: Vec::default(),
            len: usize::default(),
        }
    }
}
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.queue.pop().map(|n| {
            self.len = self.len.saturating_sub(1);
            self.queue.extend(n.children.values());
            &n.elem
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}
impl<T> ExactSizeIterator for Iter<'_, T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let tree = BkTree::new_with_items(
            levenshtein,
            vec![
                "She",
                "sells",
                "seashells",
                "by",
                "the",
                "seashore",
                "The",
                "shells",
                "she",
                "sells",
                "are",
                "seashells",
                "I'm",
                "sure",
            ]
            .into_iter()
            .map(String::from),
        );

        assert_eq!(tree.find_best(&"she".into()), Some((0, &"she".into())));
        assert_eq!(tree.find_best(&"sh".into()), Some((1, &"she".into())));
        assert_eq!(tree.find_best(&"Sh".into()), Some((1, &"She".into())));
    }
}
