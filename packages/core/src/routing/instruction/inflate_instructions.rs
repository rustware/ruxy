use std::collections::VecDeque;

use crate::routing::instruction::{MatchInstruction, InstructionKind};

/// Takes a vector of routes that are represented as a vector of instructions,
/// and creates a nested structure with common ancestors. This can also be
/// understood as a reverse effect of flattening.
pub fn inflate_instructions(routes: Vec<Vec<MatchInstruction>>) -> MatchInstruction {
  let mut root = MatchInstruction { kind: InstructionKind::Skip, ..Default::default() };

  for route in routes {
    inflate_instructions_recursive(&mut root, VecDeque::from(route));
  }

  root
}

fn inflate_instructions_recursive(current: &mut MatchInstruction, mut route: VecDeque<MatchInstruction>) {
  let Some(sequence) = route.pop_front() else {
    return;
  };

  if let Some(child) = current.next.iter_mut().find(|seq| **seq == sequence) {
    // The instruction already exists in the tree, so we just pass the pointer to it for the next instruction
    return inflate_instructions_recursive(child, route);
  }

  // The sequence does not exist, so we push it to the current node's children
  current.next.push(sequence);

  // ...and pass a pointer to it for the next route sequence
  let inserted_ref = current.next.last_mut().unwrap();
  inflate_instructions_recursive(inserted_ref, route);
}
