/// A node in a Radix Trie.
///
/// `Prefix` holds a substring and a list of child nodes.
/// `Item` holds the actual value associated with a complete key.
#[derive(Debug)]
pub enum RadixTrie<Item> {
  Prefix(String, Vec<RadixTrie<Item>>),
  Item(Item),
}

impl<Item> RadixTrie<Item> {
  /// Inserts a key-value pair into the Radix Trie.
  ///
  /// If the key shares a prefix with an existing node, the node is split and reused.
  pub fn insert(nodes: &mut Vec<RadixTrie<Item>>, key: &str, item: Item)
  where
    Item: Clone,
  {
    for node in nodes.iter_mut() {
      match node {
        RadixTrie::Prefix(label, children) => {
          let prefix_len = common_prefix_len(&label, key);

          if prefix_len == 0 {
            continue;
          }

          if prefix_len < label.len() {
            // Split this node
            let suffix = label[prefix_len..].to_string();
            let old_children = std::mem::take(children);
            let mut new_child = vec![RadixTrie::Prefix(suffix, old_children)];
            
            if prefix_len == key.len() {
              new_child.push(RadixTrie::Item(item));
            } else {
              RadixTrie::insert(&mut new_child, &key[prefix_len..], item);
            }
            
            *label = label[..prefix_len].to_string();
            *children = new_child;
            
            return;
          } else {
            // Exact match on current prefix
            RadixTrie::insert(children, &key[prefix_len..], item);
            return;
          }
        }
        RadixTrie::Item(_) => {
          // Skip terminal nodes — items do not contain children
        }
      }
    }

    // No match found: insert a new branch
    if key.is_empty() {
      nodes.push(RadixTrie::Item(item));
    } else {
      nodes.push(RadixTrie::Prefix(key.to_string(), vec![RadixTrie::Item(item)]));
    }
  }

  /// Builds a Radix Trie from a list of (key, item) pairs.
  ///
  /// Returns the top-level nodes representing the root of the trie.
  pub fn build(items: Vec<(&str, Item)>) -> Vec<RadixTrie<Item>>
  where
    Item: Clone,
  {
    let mut root = Vec::new();
    for (key, item) in items {
      RadixTrie::insert(&mut root, key, item);
    }
    root
  }
}

/// Returns the length of the common prefix (in characters) between two strings.
fn common_prefix_len(a: &str, b: &str) -> usize {
  a.chars().zip(b.chars()).take_while(|(c1, c2)| c1 == c2).count()
}
