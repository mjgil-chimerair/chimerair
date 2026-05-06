//! Zig control flow modeling.

use super::operations::ZigOp;
use serde::{Deserialize, Serialize};

/// A basic block in the control flow graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Block ID
    pub id: u64,
    /// Optional label
    pub label: String,
    /// Instructions in this block
    pub instructions: Vec<super::operations::ZigInstruction>,
    /// Terminator instruction
    pub terminator: Option<super::operations::ZigInstruction>,
    /// Successor block IDs
    pub successors: Vec<u64>,
}

impl Block {
    /// Create a new block
    pub fn new(id: u64) -> Self {
        Self {
            id,
            label: String::new(),
            instructions: Vec::new(),
            terminator: None,
            successors: Vec::new(),
        }
    }

    /// Set the block label
    pub fn set_label(&mut self, label: String) {
        self.label = label;
    }

    /// Add an instruction to this block
    pub fn add_instruction(&mut self, inst: super::operations::ZigInstruction) {
        self.instructions.push(inst);
    }

    /// Set the terminator
    pub fn set_terminator(&mut self, term: super::operations::ZigInstruction) {
        self.terminator = Some(term);
    }

    /// Add a successor
    pub fn add_successor(&mut self, succ: u64) {
        self.successors.push(succ);
    }

    /// Check if this block has a terminator
    pub fn has_terminator(&self) -> bool {
        self.terminator.is_some()
    }

    /// Get the terminator op
    pub fn terminator_op(&self) -> Option<&ZigOp> {
        self.terminator.as_ref().map(|t| &t.op)
    }
}

/// Block ID type alias
pub type BlockId = u64;

/// Control flow graph for a function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlFlowGraph {
    /// Entry block ID
    pub entry: BlockId,
    /// All blocks indexed by ID
    blocks: Vec<Block>,
    /// Block ID lookup
    block_ids: Vec<BlockId>,
}

impl ControlFlowGraph {
    /// Create a new CFG
    pub fn new(entry: BlockId) -> Self {
        Self {
            entry,
            blocks: Vec::new(),
            block_ids: Vec::new(),
        }
    }

    /// Add a block
    pub fn add_block(&mut self, block: Block) {
        self.block_ids.push(block.id);
        self.blocks.push(block);
    }

    /// Get a block by ID
    pub fn get_block(&self, id: BlockId) -> Option<&Block> {
        self.blocks.iter().find(|b| b.id == id)
    }

    /// Get a block by ID mutably
    pub fn get_block_mut(&mut self, id: BlockId) -> Option<&mut Block> {
        self.blocks.iter_mut().find(|b| b.id == id)
    }

    /// Get the entry block
    pub fn entry_block(&self) -> Option<&Block> {
        self.get_block(self.entry)
    }

    /// Number of blocks
    pub fn num_blocks(&self) -> usize {
        self.blocks.len()
    }

    /// Check if a block exists
    pub fn has_block(&self, id: BlockId) -> bool {
        self.block_ids.contains(&id)
    }

    /// Get all block IDs
    pub fn block_ids(&self) -> &[BlockId] {
        &self.block_ids
    }

    /// Iteratate over blocks
    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    /// Check if CFG is well-formed
    pub fn is_well_formed(&self) -> bool {
        // Entry block must exist
        if !self.has_block(self.entry) {
            return false;
        }

        // Each successor must refer to an existing block
        for block in &self.blocks {
            for &succ in &block.successors {
                if !self.has_block(succ) {
                    return false;
                }
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operations::{ZigInstruction, ZigOp};

    #[test]
    fn test_block_creation() {
        let block = Block::new(0);
        assert_eq!(block.id, 0);
        assert!(!block.has_terminator());
    }

    #[test]
    fn test_block_with_instruction() {
        let mut block = Block::new(0);
        block.set_label("entry".to_string());

        let inst = ZigInstruction::new(1, ZigOp::Add);
        block.add_instruction(inst);
        assert_eq!(block.instructions.len(), 1);
    }

    #[test]
    fn test_block_with_terminator() {
        let mut block = Block::new(0);
        let term = ZigInstruction::new(2, ZigOp::Br);
        block.set_terminator(term);
        assert!(block.has_terminator());
        assert!(matches!(block.terminator_op(), Some(ZigOp::Br)));
    }

    #[test]
    fn test_cfg_creation() {
        let cfg = ControlFlowGraph::new(0);
        assert_eq!(cfg.entry, 0);
        assert_eq!(cfg.num_blocks(), 0);
    }

    #[test]
    fn test_cfg_add_block() {
        let mut cfg = ControlFlowGraph::new(0);
        let block = Block::new(0);
        cfg.add_block(block);
        assert_eq!(cfg.num_blocks(), 1);
        assert!(cfg.has_block(0));
    }

    #[test]
    fn test_cfg_get_block() {
        let mut cfg = ControlFlowGraph::new(0);
        let block = Block::new(0);
        cfg.add_block(block);

        let found = cfg.get_block(0);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, 0);
    }

    #[test]
    fn test_cfg_well_formed() {
        let mut cfg = ControlFlowGraph::new(0);
        let mut block = Block::new(0);
        block.add_successor(1);
        cfg.add_block(block);

        // Should be well-formed since successor block doesn't exist yet
        assert!(!cfg.is_well_formed());

        // Add the successor
        let block2 = Block::new(1);
        cfg.add_block(block2);
        assert!(cfg.is_well_formed());
    }

    #[test]
    fn test_loop_cfg_with_break_continue() {
        // Build a loop: entry -> header -> body -> header (loop back)
        //                header -> exit (break)
        //                body -> continue_target -> header
        let mut cfg = ControlFlowGraph::new(0);

        // Entry block
        let mut entry = Block::new(0);
        entry.add_instruction(ZigInstruction::new(1, ZigOp::Add));
        entry.set_terminator(ZigInstruction::new(2, ZigOp::Br));
        entry.add_successor(1); // to header
        cfg.add_block(entry);

        // Header block (loop condition)
        let mut header = Block::new(1);
        header.add_instruction(ZigInstruction::new(3, ZigOp::Slt));
        header.set_terminator(ZigInstruction::new(4, ZigOp::BrCond));
        header.add_successor(2); // body
        header.add_successor(3); // exit
        cfg.add_block(header);

        // Loop body
        let mut body = Block::new(2);
        body.add_instruction(ZigInstruction::new(5, ZigOp::Load));
        body.set_terminator(ZigInstruction::new(6, ZigOp::Br));
        body.add_successor(4); // continue target
        cfg.add_block(body);

        // Continue target (increment)
        let mut continue_target = Block::new(4);
        continue_target.add_instruction(ZigInstruction::new(7, ZigOp::Add));
        continue_target.set_terminator(ZigInstruction::new(8, ZigOp::Br));
        continue_target.add_successor(1); // back to header
        cfg.add_block(continue_target);

        // Exit block
        let mut exit = Block::new(3);
        exit.set_terminator(ZigInstruction::new(9, ZigOp::RetVoid));
        cfg.add_block(exit);

        assert!(cfg.is_well_formed());
        assert_eq!(cfg.num_blocks(), 5);
    }

    #[test]
    fn test_switch_cfg() {
        // Build switch: entry -> switch -> case0, case1, case2, default, merge
        let mut cfg = ControlFlowGraph::new(0);

        // Entry with switch
        let mut entry = Block::new(0);
        entry.add_instruction(ZigInstruction::new(1, ZigOp::Load));
        entry.set_terminator(ZigInstruction::new(2, ZigOp::Switch));
        entry.add_successor(1); // case0
        entry.add_successor(2); // case1
        entry.add_successor(3); // case2
        entry.add_successor(4); // default
        cfg.add_block(entry);

        // Case 0
        let mut case0 = Block::new(1);
        case0.set_terminator(ZigInstruction::new(3, ZigOp::Br));
        case0.add_successor(5); // merge
        cfg.add_block(case0);

        // Case 1
        let mut case1 = Block::new(2);
        case1.set_terminator(ZigInstruction::new(4, ZigOp::Br));
        case1.add_successor(5); // merge
        cfg.add_block(case1);

        // Case 2
        let mut case2 = Block::new(3);
        case2.set_terminator(ZigInstruction::new(5, ZigOp::Br));
        case2.add_successor(5); // merge
        cfg.add_block(case2);

        // Default
        let mut default = Block::new(4);
        default.set_terminator(ZigInstruction::new(6, ZigOp::Br));
        default.add_successor(5); // merge
        cfg.add_block(default);

        // Merge block
        let mut merge = Block::new(5);
        merge.set_terminator(ZigInstruction::new(7, ZigOp::RetVoid));
        cfg.add_block(merge);

        assert!(cfg.is_well_formed());
        assert_eq!(cfg.num_blocks(), 6);
    }

    #[test]
    fn test_try_error_cfg() {
        // Build error handling: entry -> try -> catch -> merge
        let mut cfg = ControlFlowGraph::new(0);

        // Entry block
        let mut entry = Block::new(0);
        entry.add_instruction(ZigInstruction::new(1, ZigOp::Call));
        entry.set_terminator(ZigInstruction::new(2, ZigOp::Invoke));
        entry.add_successor(1); // success
        entry.add_successor(2); // error/catch
        cfg.add_block(entry);

        // Success path
        let mut success = Block::new(1);
        success.set_terminator(ZigInstruction::new(3, ZigOp::RetVoid));
        cfg.add_block(success);

        // Error/Catch block
        let mut catch = Block::new(2);
        catch.add_instruction(ZigInstruction::new(4, ZigOp::IsErr));
        catch.set_terminator(ZigInstruction::new(5, ZigOp::Br));
        catch.add_successor(3); // merge
        cfg.add_block(catch);

        // Merge block
        let mut merge = Block::new(3);
        merge.set_terminator(ZigInstruction::new(6, ZigOp::RetVoid));
        cfg.add_block(merge);

        assert!(cfg.is_well_formed());
        assert_eq!(cfg.num_blocks(), 4);
    }

    #[test]
    fn test_defer_cfg() {
        // Build defer ordering: entry -> body -> defer_a -> defer_b -> return
        let mut cfg = ControlFlowGraph::new(0);

        // Entry
        let mut entry = Block::new(0);
        entry.add_instruction(ZigInstruction::new(1, ZigOp::Alloca));
        entry.set_terminator(ZigInstruction::new(2, ZigOp::Br));
        entry.add_successor(1);
        cfg.add_block(entry);

        // Body
        let mut body = Block::new(1);
        body.add_instruction(ZigInstruction::new(3, ZigOp::Store));
        body.set_terminator(ZigInstruction::new(4, ZigOp::Br));
        body.add_successor(2); // to defer_a (LIFO - last defer runs first)
        cfg.add_block(body);

        // Defer A (runs second)
        let mut defer_a = Block::new(2);
        defer_a.add_instruction(ZigInstruction::new(5, ZigOp::Load));
        defer_a.set_terminator(ZigInstruction::new(6, ZigOp::Br));
        defer_a.add_successor(3);
        cfg.add_block(defer_a);

        // Defer B (runs first - LIFO)
        let mut defer_b = Block::new(3);
        defer_b.add_instruction(ZigInstruction::new(7, ZigOp::Load));
        defer_b.set_terminator(ZigInstruction::new(8, ZigOp::Br));
        defer_b.add_successor(4);
        cfg.add_block(defer_b);

        // Return block
        let mut ret = Block::new(4);
        ret.set_terminator(ZigInstruction::new(9, ZigOp::RetVoid));
        cfg.add_block(ret);

        assert!(cfg.is_well_formed());
        assert_eq!(cfg.num_blocks(), 5);
    }

    #[test]
    fn test_unreachable_block() {
        let mut cfg = ControlFlowGraph::new(0);

        let mut entry = Block::new(0);
        entry.set_terminator(ZigInstruction::new(1, ZigOp::Unreachable));
        cfg.add_block(entry);

        assert!(cfg.is_well_formed());
    }

    #[test]
    fn test_break_continue_blocks() {
        let mut cfg = ControlFlowGraph::new(0);

        // Build nested loop with break and continue
        let mut outer_header = Block::new(0);
        outer_header.set_terminator(ZigInstruction::new(1, ZigOp::BrCond));
        outer_header.add_successor(1); // inner
        outer_header.add_successor(4); // outer exit
        cfg.add_block(outer_header);

        let mut inner = Block::new(1);
        inner.add_instruction(ZigInstruction::new(2, ZigOp::Add));
        inner.set_terminator(ZigInstruction::new(3, ZigOp::BrCond));
        inner.add_successor(1); // continue to inner header
        inner.add_successor(2); // break to outer exit
        cfg.add_block(inner);

        let mut inner_body = Block::new(2);
        inner_body.set_terminator(ZigInstruction::new(4, ZigOp::Br));
        inner_body.add_successor(1); // continue
        cfg.add_block(inner_body);

        let mut outer_exit = Block::new(4);
        outer_exit.set_terminator(ZigInstruction::new(5, ZigOp::RetVoid));
        cfg.add_block(outer_exit);

        assert!(cfg.is_well_formed());
        assert_eq!(cfg.num_blocks(), 4);
    }

    #[test]
    fn test_terminator_ops() {
        assert!(ZigOp::Ret.is_terminator());
        assert!(ZigOp::RetVoid.is_terminator());
        assert!(ZigOp::Br.is_terminator());
        assert!(ZigOp::BrCond.is_terminator());
        assert!(ZigOp::Switch.is_terminator());
        assert!(ZigOp::Unreachable.is_terminator());
        assert!(ZigOp::Invoke.is_terminator());
        assert!(!ZigOp::Add.is_terminator());
        assert!(!ZigOp::Load.is_terminator());
        assert!(!ZigOp::Store.is_terminator());
    }
}
