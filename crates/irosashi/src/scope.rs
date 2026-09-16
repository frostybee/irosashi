use std::collections::HashMap;
use std::sync::Arc;

/// An interned scope name such as `keyword.control.go`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(pub(crate) u32);

impl ScopeId {
    pub fn as_u32(self) -> u32 {
        self.0
    }
}

/// An interned scope stack, outermost scope first.
///
/// Ids are only meaningful within the `ScopeInterner` (and therefore the `Session`)
/// that produced them. `ScopeListId::EMPTY` is the empty stack in every interner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeListId(pub(crate) u32);

impl ScopeListId {
    pub const EMPTY: ScopeListId = ScopeListId(0);

    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// Hash-consed scope stacks: every non-empty list is a node `(parent, scope)`, so
/// extending a stack by one scope is a single map lookup and every token stores one
/// `u32`.
#[derive(Debug)]
pub struct ScopeInterner {
    names: Vec<Arc<str>>,
    name_ids: HashMap<Arc<str>, ScopeId>,
    nodes: Vec<Option<(ScopeListId, ScopeId)>>,
    node_ids: HashMap<(ScopeListId, ScopeId), ScopeListId>,
}

impl Default for ScopeInterner {
    fn default() -> Self {
        Self::new()
    }
}

impl ScopeInterner {
    pub fn new() -> Self {
        Self {
            names: Vec::new(),
            name_ids: HashMap::new(),
            nodes: vec![None],
            node_ids: HashMap::new(),
        }
    }

    pub fn intern_name(&mut self, name: &str) -> ScopeId {
        if let Some(&id) = self.name_ids.get(name) {
            return id;
        }
        let id = ScopeId(self.names.len() as u32);
        let name: Arc<str> = Arc::from(name);
        self.names.push(Arc::clone(&name));
        self.name_ids.insert(name, id);
        id
    }

    pub fn name(&self, id: ScopeId) -> &str {
        &self.names[id.0 as usize]
    }

    pub fn push(&mut self, parent: ScopeListId, scope: ScopeId) -> ScopeListId {
        let key = (parent, scope);
        if let Some(&id) = self.node_ids.get(&key) {
            return id;
        }
        let id = ScopeListId(self.nodes.len() as u32);
        self.nodes.push(Some(key));
        self.node_ids.insert(key, id);
        id
    }

    /// Extends `parent` with every whitespace-separated scope in `names`, in order.
    /// Returns `parent` unchanged when `names` has no scopes.
    pub fn push_names(&mut self, parent: ScopeListId, names: &str) -> ScopeListId {
        let mut list = parent;
        for name in names.split_whitespace() {
            let scope = self.intern_name(name);
            list = self.push(list, scope);
        }
        list
    }

    /// Number of scope lists interned so far (including the empty one); the upper
    /// bound for dense caches indexed by `ScopeListId`.
    pub fn list_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn len(&self, list: ScopeListId) -> usize {
        let mut count = 0;
        let mut current = list;
        while let Some((parent, _)) = self.nodes[current.index()] {
            count += 1;
            current = parent;
        }
        count
    }

    /// Scope names of `list`, outermost first.
    pub fn names(&self, list: ScopeListId) -> Vec<&str> {
        let mut out = Vec::with_capacity(self.len(list));
        let mut current = list;
        while let Some((parent, scope)) = self.nodes[current.index()] {
            out.push(self.name(scope));
            current = parent;
        }
        out.reverse();
        out
    }

    pub fn names_owned(&self, list: ScopeListId) -> Vec<String> {
        self.names(list).into_iter().map(str::to_owned).collect()
    }

    /// Scope names of `list` sharing the interner's allocations.
    pub fn names_shared(&self, list: ScopeListId) -> Box<[Arc<str>]> {
        let mut out = Vec::with_capacity(self.len(list));
        let mut current = list;
        while let Some((parent, scope)) = self.nodes[current.index()] {
            out.push(Arc::clone(&self.names[scope.0 as usize]));
            current = parent;
        }
        out.reverse();
        out.into_boxed_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_are_hash_consed_and_outermost_first() {
        let mut interner = ScopeInterner::new();
        let root = interner.push_names(ScopeListId::EMPTY, "source.go");
        let a = interner.push_names(root, "meta.block keyword");
        let b = interner.push_names(root, "meta.block keyword");
        assert_eq!(a, b);
        assert_eq!(interner.names(a), ["source.go", "meta.block", "keyword"]);
        assert_eq!(interner.len(a), 3);
        assert_eq!(interner.list_count(), 4);
    }

    #[test]
    fn empty_names_leave_the_parent_unchanged() {
        let mut interner = ScopeInterner::new();
        let root = interner.push_names(ScopeListId::EMPTY, "source.go");
        assert_eq!(interner.push_names(root, "   "), root);
        assert_eq!(
            interner.push_names(ScopeListId::EMPTY, ""),
            ScopeListId::EMPTY
        );
        assert!(interner.names(ScopeListId::EMPTY).is_empty());
        assert_eq!(interner.len(ScopeListId::EMPTY), 0);
    }
}
