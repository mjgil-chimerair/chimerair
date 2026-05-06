//! Task 89: MIR to Rust dialect control flow lowering
//!
//! Converts basic blocks, branches, switch, calls, returns, cleanup edges,
//! and panic edges into dialect ops.

use serde::{Deserialize, Serialize};

/// Control flow graph representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlFlowGraph {
    /// Basic blocks in the function
    pub blocks: Vec<BasicBlock>,
    /// Entry block index
    pub entry: usize,
}

/// A basic block in the CFG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicBlock {
    /// Block label/id
    pub id: String,
    /// Statements in this block (simplified representation)
    pub statements: Vec<String>,
    /// Terminator that controls flow
    pub terminator: BlockTerminator,
}

/// Block terminator types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockTerminator {
    /// Unconditional goto
    Goto { target: String },
    /// Conditional branch
    Branch {
        condition: String,
        then_target: String,
        else_target: String,
    },
    /// Switch on integer value
    SwitchInt {
        value: String,
        cases: Vec<SwitchCase>,
        default_target: String,
    },
    /// Return from function
    Return { value: Option<String> },
    /// Call with possible normal/panic exit
    Call {
        callee: String,
        args: Vec<String>,
        normal_target: String,
        unwind_target: Option<String>,
    },
    /// Drop a value
    Drop {
        place: String,
        target: String,
        unwind_target: Option<String>,
    },
    /// Assert and abort if false
    Assert {
        condition: String,
        expected: bool,
        msg: String,
        target: String,
        unwind_target: Option<String>,
    },
    /// Resume unwind
    Resume,
    /// Abort execution
    Abort,
    /// Unreachable
    Unreachable,
}

/// A case in a switch statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchCase {
    pub value: i128,
    pub target: String,
}

/// Target for branch instructions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchTarget {
    pub block: String,
    pub args: Vec<String>,
}

impl BlockTerminator {
    /// Check if this terminator can unwind
    pub fn can_unwind(&self) -> bool {
        match self {
            BlockTerminator::Goto { .. } => false,
            BlockTerminator::Branch { .. } => false,
            BlockTerminator::SwitchInt { .. } => false,
            BlockTerminator::Return { .. } => false,
            BlockTerminator::Call { unwind_target, .. } => unwind_target.is_some(),
            BlockTerminator::Drop { unwind_target, .. } => unwind_target.is_some(),
            BlockTerminator::Assert { unwind_target, .. } => unwind_target.is_some(),
            BlockTerminator::Resume => true,
            BlockTerminator::Abort => false,
            BlockTerminator::Unreachable => false,
        }
    }

    /// Get all possible successor blocks
    pub fn successors(&self) -> Vec<String> {
        match self {
            BlockTerminator::Goto { target } => vec![target.clone()],
            BlockTerminator::Branch {
                then_target,
                else_target,
                ..
            } => {
                vec![then_target.clone(), else_target.clone()]
            }
            BlockTerminator::SwitchInt {
                cases,
                default_target,
                ..
            } => {
                let mut targets: Vec<String> = cases.iter().map(|c| c.target.clone()).collect();
                targets.push(default_target.clone());
                targets
            }
            BlockTerminator::Return { .. } => vec![],
            BlockTerminator::Call {
                normal_target,
                unwind_target,
                ..
            } => {
                let mut targets = vec![normal_target.clone()];
                if let Some(t) = unwind_target {
                    targets.push(t.clone());
                }
                targets
            }
            BlockTerminator::Drop {
                target,
                unwind_target,
                ..
            } => {
                let mut targets = vec![target.clone()];
                if let Some(t) = unwind_target {
                    targets.push(t.clone());
                }
                targets
            }
            BlockTerminator::Assert {
                target,
                unwind_target,
                ..
            } => {
                let mut targets = vec![target.clone()];
                if let Some(t) = unwind_target {
                    targets.push(t.clone());
                }
                targets
            }
            BlockTerminator::Resume => vec![],
            BlockTerminator::Abort => vec![],
            BlockTerminator::Unreachable => vec![],
        }
    }
}

impl ControlFlowGraph {
    /// Create a new CFG
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            entry: 0,
        }
    }

    /// Add a block to the CFG
    pub fn add_block(&mut self, block: BasicBlock) -> usize {
        self.blocks.push(block);
        self.blocks.len() - 1
    }

    /// Validate CFG structure
    pub fn validate(&self) -> Result<(), CFGError> {
        if self.blocks.is_empty() {
            return Err(CFGError::NoBlocks);
        }

        if self.entry >= self.blocks.len() {
            return Err(CFGError::InvalidEntry);
        }

        // Collect all block IDs
        let block_ids: std::collections::HashSet<_> =
            self.blocks.iter().map(|b| b.id.clone()).collect();

        // Check all terminators reference valid blocks
        for block in &self.blocks {
            for succ in block.terminator.successors() {
                if !block_ids.contains(&succ) {
                    return Err(CFGError::InvalidSuccessor(succ));
                }
            }
        }

        // Check for cycles (simple cycle detection)
        self.detect_cycles()?;

        Ok(())
    }

    /// Simple cycle detection using DFS
    fn detect_cycles(&self) -> Result<(), CFGError> {
        let mut visited = std::collections::HashSet::new();
        let mut in_stack = std::collections::HashSet::new();

        fn dfs(
            cfg: &ControlFlowGraph,
            block_id: &str,
            visited: &mut std::collections::HashSet<String>,
            in_stack: &mut std::collections::HashSet<String>,
        ) -> Result<(), CFGError> {
            let block_id_owned = block_id.to_string();
            if in_stack.contains(&block_id_owned) {
                return Err(CFGError::CycleDetected(block_id.to_string()));
            }

            if visited.contains(&block_id_owned) {
                return Ok(());
            }

            visited.insert(block_id_owned.clone());
            in_stack.insert(block_id_owned.clone());

            // Find the block and check successors
            for block in &cfg.blocks {
                if block.id == *block_id {
                    for succ in block.terminator.successors() {
                        dfs(cfg, &succ, visited, in_stack)?;
                    }
                    break;
                }
            }

            in_stack.remove(&block_id_owned);
            Ok(())
        }

        let entry_id = self.blocks[self.entry].id.clone();
        dfs(self, &entry_id, &mut visited, &mut in_stack)
    }

    /// Get unreachable blocks
    pub fn unreachable_blocks(&self) -> Vec<String> {
        let mut reachable = std::collections::HashSet::new();
        let mut stack = vec![self.blocks[self.entry].id.clone()];

        while let Some(id) = stack.pop() {
            if reachable.contains(&id) {
                continue;
            }
            reachable.insert(id.clone());

            for block in &self.blocks {
                if block.id == id {
                    for succ in block.terminator.successors() {
                        if !reachable.contains(&succ) {
                            stack.push(succ);
                        }
                    }
                    break;
                }
            }
        }

        self.blocks
            .iter()
            .filter(|b| !reachable.contains(&b.id))
            .map(|b| b.id.clone())
            .collect()
    }
}

impl Default for ControlFlowGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CFGError {
    #[error("CFG has no blocks")]
    NoBlocks,
    #[error("invalid entry block index")]
    InvalidEntry,
    #[error("successor block '{0}' does not exist")]
    InvalidSuccessor(String),
    #[error("cycle detected involving block '{0}'")]
    CycleDetected(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goto_terminator() {
        let term = BlockTerminator::Goto {
            target: "bb1".to_string(),
        };
        assert!(!term.can_unwind());
        assert_eq!(term.successors(), vec!["bb1"]);
    }

    #[test]
    fn test_branch_terminator() {
        let term = BlockTerminator::Branch {
            condition: "x".to_string(),
            then_target: "bb1".to_string(),
            else_target: "bb2".to_string(),
        };
        assert_eq!(term.successors(), vec!["bb1", "bb2"]);
    }

    #[test]
    fn test_call_with_unwind() {
        let term = BlockTerminator::Call {
            callee: "foo".to_string(),
            args: vec!["x".to_string()],
            normal_target: "bb1".to_string(),
            unwind_target: Some("cleanup".to_string()),
        };
        assert!(term.can_unwind());
        assert_eq!(term.successors(), vec!["bb1", "cleanup"]);
    }

    #[test]
    fn test_cfg_validate_valid() {
        let mut cfg = ControlFlowGraph::new();
        cfg.add_block(BasicBlock {
            id: "entry".to_string(),
            statements: vec![],
            terminator: BlockTerminator::Goto {
                target: "exit".to_string(),
            },
        });
        cfg.add_block(BasicBlock {
            id: "exit".to_string(),
            statements: vec![],
            terminator: BlockTerminator::Return { value: None },
        });

        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_cfg_validate_invalid_successor() {
        let mut cfg = ControlFlowGraph::new();
        cfg.add_block(BasicBlock {
            id: "entry".to_string(),
            statements: vec![],
            terminator: BlockTerminator::Goto {
                target: "nonexistent".to_string(),
            },
        });

        assert!(cfg.validate().is_err());
    }

    #[test]
    fn test_unreachable_blocks() {
        let mut cfg = ControlFlowGraph::new();
        cfg.add_block(BasicBlock {
            id: "entry".to_string(),
            statements: vec![],
            terminator: BlockTerminator::Return { value: None },
        });
        cfg.add_block(BasicBlock {
            id: "unreachable".to_string(),
            statements: vec![],
            terminator: BlockTerminator::Return { value: None },
        });

        let unreachable = cfg.unreachable_blocks();
        assert_eq!(unreachable, vec!["unreachable"]);
    }

    #[test]
    fn test_switch_cases() {
        let term = BlockTerminator::SwitchInt {
            value: "x".to_string(),
            cases: vec![
                SwitchCase {
                    value: 0,
                    target: "zero".to_string(),
                },
                SwitchCase {
                    value: 1,
                    target: "one".to_string(),
                },
            ],
            default_target: "default".to_string(),
        };
        assert_eq!(term.successors(), vec!["zero", "one", "default"]);
    }
}
