/// Ruxy's implementation of Radix Trie.
#[derive(Debug, Clone)]
pub struct RadixTrie<Item> where Item: Clone {
  nodes: Vec<RadixTrieNode<Item>>,
}

impl<Item> RadixTrie<Item>
where
  Item: Clone,
{
  pub fn new() -> Self {
    Self { nodes: vec![] }
  }

  /// Inserts a key-value pair into the Radix Trie.
  ///
  /// If the key shares a prefix with an existing node, the node is split and reused.
  pub fn insert(&mut self, key: &str, item: Item) {
    for node in self.nodes.iter_mut() {
      match node {
        RadixTrieNode::Prefix(label, children) => {
          let prefix_len = common_prefix_len(label, key);

          if prefix_len == 0 {
            continue;
          }

          if prefix_len < label.len() {
            // Split this node
            let suffix = label[prefix_len..].to_string();
            let old_children = std::mem::take(children);
            let mut new_child = Self { nodes: vec![RadixTrieNode::Prefix(suffix, old_children)] };

            if prefix_len == key.len() {
              new_child.nodes.push(RadixTrieNode::Item(item));
            } else {
              new_child.insert(&key[prefix_len..], item);
            }

            *label = label[..prefix_len].to_string();
            *children = new_child;

            return;
          } else {
            // Exact match on current prefix
            children.insert(&key[prefix_len..], item);
            return;
          }
        }
        RadixTrieNode::Item(_) => {
          // Skip terminal nodes — items do not contain children
        }
      }
    }

    // No match found: insert a new branch
    if key.is_empty() {
      self.nodes.push(RadixTrieNode::Item(item));
    } else {
      self.nodes.push(RadixTrieNode::Prefix(key.to_string(), Self { nodes: vec![RadixTrieNode::Item(item)] }));
    }
  }

  /// Exports the Radix trie, returning a vector of nodes
  pub fn to_nodes(&self) -> &[RadixTrieNode<Item>] {
    &self.nodes
  }

  /// Extracts all key-value pairs from the trie as a flat vector of references.
  pub fn to_flat(&self) -> Vec<(String, &Item)> {
    let mut pairs = Vec::new();
    self.to_flat_recursive(&mut pairs, String::new());
    pairs
  }

  fn to_flat_recursive<'a>(&'a self, pairs: &mut Vec<(String, &'a Item)>, current_prefix: String) {
    for node in &self.nodes {
      match node {
        RadixTrieNode::Prefix(label, children) => {
          let mut new_prefix = current_prefix.clone();
          new_prefix.push_str(label);
          children.to_flat_recursive(pairs, new_prefix);
        }
        RadixTrieNode::Item(item) => {
          pairs.push((current_prefix.clone(), item));
        }
      }
    }
  }

  /// Consumes the trie and extracts all key-value pairs.
  pub fn into_flat(self) -> Vec<(String, Item)> {
    let mut pairs = Vec::new();
    self.into_flat_recursive(&mut pairs, String::new());
    pairs
  }

  fn into_flat_recursive(self, pairs: &mut Vec<(String, Item)>, current_prefix: String) {
    for node in self.nodes {
      match node {
        RadixTrieNode::Prefix(label, children) => {
          let mut new_prefix = current_prefix.clone();
          new_prefix.push_str(&label);
          children.into_flat_recursive(pairs, new_prefix);
        }
        RadixTrieNode::Item(item) => {
          pairs.push((current_prefix.clone(), item));
        }
      }
    }
  }

  /// Extends this trie by consuming another trie and merging its items.
  pub fn extend(&mut self, other: RadixTrie<Item>) {
    for (key, item) in other.into_flat() {
      self.insert(&key, item);
    }
  }

  /// Inserts another RadixTrie under a specified prefix.
  pub fn extend_with_prefix(&mut self, prefix: &str, other: RadixTrie<Item>) {
    for (key, item) in other.into_flat() {
      let full_key = format!("{}{}", prefix, key);
      self.insert(&full_key, item);
    }
  }

  /// Creates a new RadixTrie with all entries prefixed with a given prefix,
  /// consuming the current RadixTrie on which this method was called.
  pub fn with_prefix(self, prefix: &str) -> Self {
    let mut trie = RadixTrie::new();
    trie.extend_with_prefix(prefix, self);
    trie
  }
}

impl<Key, Item, Iter> From<Iter> for RadixTrie<Item>
where
  Key: AsRef<str>,
  Item: Clone,
  Iter: IntoIterator<Item = (Key, Item)>,
{
  fn from(iter: Iter) -> Self {
    let mut trie = Self::new();

    for (prefix, target) in iter {
      trie.insert(prefix.as_ref(), target);
    }

    trie
  }
}

impl<Item> Default for RadixTrie<Item>
where
  Item: Clone,
{
  fn default() -> Self {
    Self::new()
  }
}

/// A node in a Radix Trie.
///
/// `Prefix` holds a substring and a list of child nodes.
/// `Item` holds the actual value associated with a complete key.
#[derive(Debug, Clone)]
pub enum RadixTrieNode<Item> where Item: Clone {
  Prefix(String, RadixTrie<Item>),
  Item(Item),
}

impl<Item> Default for RadixTrieNode<Item>
where
  Item: Clone,
{
  fn default() -> Self {
    RadixTrieNode::Prefix(String::new(), RadixTrie::new())
  }
}

/// Returns the length of the common prefix (in characters) between two strings.
fn common_prefix_len(a: &str, b: &str) -> usize {
  a.chars().zip(b.chars()).take_while(|(c1, c2)| c1 == c2).count()
}
