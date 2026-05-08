//! MLIR emitter for Chimera dialect output.

use std::fmt::Write;

/// MLIR emitter for converting Zig dialect to MLIR text format
#[derive(Debug, Clone)]
pub struct MlirEmitter {
    /// Output buffer
    buf: String,
    /// Current indentation level
    indent: usize,
}

impl MlirEmitter {
    /// Create a new emitter
    pub fn new() -> Self {
        Self {
            buf: String::new(),
            indent: 0,
        }
    }

    /// Emit a module header
    pub fn emit_module_header(&mut self, name: &str, source_lang: &str) {
        writeln!(
            self.buf,
            "module @{} attributes {{ chimera.source_lang = \"{}\" }} {{",
            name, source_lang
        )
        .unwrap();
        self.indent += 1;
    }

    /// Emit module footer
    pub fn emit_module_footer(&mut self) {
        self.indent -= 1;
        writeln!(self.buf, "}}").unwrap();
    }

    /// Emit a function
    pub fn emit_function(&mut self, name: &str, type_sig: &str, is_exported: bool) {
        let export_attr = if is_exported { " {chimera.export}" } else { "" };
        writeln!(
            self.buf,
            "{}func.func @{}{}{} {{",
            "  ".repeat(self.indent),
            name,
            type_sig,
            export_attr
        )
        .unwrap();
        self.indent += 1;
    }

    /// Emit function footer
    pub fn emit_function_footer(&mut self) {
        self.indent -= 1;
        writeln!(self.buf, "{}func.return", "  ".repeat(self.indent)).unwrap();
        writeln!(self.buf, "}}").unwrap();
    }

    /// Emit an operation with result
    pub fn emit_op_with_result(
        &mut self,
        op: &str,
        operands: &[&str],
        result_type: &str,
        res_name: &str,
    ) {
        let ops_str = if operands.is_empty() {
            String::new()
        } else {
            format!(" {}", operands.join(", "))
        };
        writeln!(
            self.buf,
            "{}%{} = {} {}{} : {}",
            "  ".repeat(self.indent),
            res_name,
            op,
            ops_str,
            if operands.is_empty() { "" } else { " : " },
            result_type
        )
        .unwrap();
    }

    /// Emit a simple operation (no result)
    pub fn emit_op(&mut self, op: &str, operands: &[&str], op_type: &str) {
        let ops_str = if operands.is_empty() {
            String::new()
        } else {
            format!(" {}", operands.join(", "))
        };
        writeln!(
            self.buf,
            "{}{} {}{}",
            "  ".repeat(self.indent),
            op,
            ops_str,
            if operands.is_empty() { "" } else { " : " }
        )
        .unwrap();
    }

    /// Emit a constant
    pub fn emit_constant(&mut self, name: &str, value: &str, result_type: &str) {
        writeln!(
            self.buf,
            "{}%{} = arith.constant {} : {}",
            "  ".repeat(self.indent),
            name,
            value,
            result_type
        )
        .unwrap();
    }

    /// Emit a return
    pub fn emit_return(&mut self, value: Option<&str>, result_type: &str) {
        if let Some(v) = value {
            writeln!(
                self.buf,
                "{}return %{} : {}",
                "  ".repeat(self.indent),
                v,
                result_type
            )
            .unwrap();
        } else {
            writeln!(self.buf, "{}return", "  ".repeat(self.indent)).unwrap();
        }
    }

    /// Emit a branch
    pub fn emit_branch(&mut self, target: &str, operands: &[&str]) {
        let ops_str = if operands.is_empty() {
            String::new()
        } else {
            format!("({})", operands.join(", "))
        };
        writeln!(
            self.buf,
            "{}cf.br ^bb{}{}",
            "  ".repeat(self.indent),
            target,
            ops_str
        )
        .unwrap();
    }

    /// Emit a conditional branch
    pub fn emit_cond_branch(&mut self, cond: &str, true_target: &str, false_target: &str) {
        writeln!(
            self.buf,
            "{}cf.cond_br %{}, ^bb{}, ^bb{}",
            "  ".repeat(self.indent),
            cond,
            true_target,
            false_target
        )
        .unwrap();
    }

    /// Emit an addition
    pub fn emit_add(&mut self, result: &str, lhs: &str, rhs: &str, result_type: &str) {
        writeln!(
            self.buf,
            "{}%{} = arith.addi %{}, %{} : {}",
            "  ".repeat(self.indent),
            result,
            lhs,
            rhs,
            result_type
        )
        .unwrap();
    }

    /// Emit a load
    pub fn emit_load(&mut self, result: &str, addr: &str, result_type: &str) {
        writeln!(
            self.buf,
            "{}%{} = memref.load %{} : {}",
            "  ".repeat(self.indent),
            result,
            addr,
            result_type
        )
        .unwrap();
    }

    /// Emit a store
    pub fn emit_store(&mut self, value: &str, addr: &str, result_type: &str) {
        writeln!(
            self.buf,
            "{}memref.store %{}, %{} : {}",
            "  ".repeat(self.indent),
            value,
            addr,
            result_type
        )
        .unwrap();
    }

    /// Emit a function call
    pub fn emit_call(
        &mut self,
        result: Option<&str>,
        callee: &str,
        args: &[&str],
        result_type: Option<&str>,
    ) {
        let res_str = result.map(|r| format!("%{} = ", r)).unwrap_or_default();
        let args_str = if args.is_empty() {
            String::new()
        } else {
            format!("({})", args.join(", "))
        };
        let type_str = result_type.map(|t| format!(" : {}", t)).unwrap_or_default();
        writeln!(
            self.buf,
            "{}{}func.call @{}{}{}",
            "  ".repeat(self.indent),
            res_str,
            callee,
            args_str,
            type_str
        )
        .unwrap();
    }

    /// Get the emitted output
    pub fn output(&self) -> &str {
        &self.buf
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.buf.clear();
        self.indent = 0;
    }
}

impl Default for MlirEmitter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emitter_creation() {
        let emitter = MlirEmitter::new();
        assert!(emitter.output().is_empty());
    }

    #[test]
    fn test_emit_module_header() {
        let mut emitter = MlirEmitter::new();
        emitter.emit_module_header("test", "zig");
        let output = emitter.output();
        assert!(output.contains("module @test"));
        assert!(output.contains("chimera.source_lang = \"zig\""));
    }

    #[test]
    fn test_emit_constant() {
        let mut emitter = MlirEmitter::new();
        emitter.emit_constant("c42", "42", "i32");
        assert!(emitter.output().contains("arith.constant 42 : i32"));
    }

    #[test]
    fn test_emit_add() {
        let mut emitter = MlirEmitter::new();
        emitter.emit_add("result", "lhs", "rhs", "i32");
        assert!(emitter.output().contains("arith.addi %lhs, %rhs : i32"));
    }

    #[test]
    fn test_emit_function() {
        let mut emitter = MlirEmitter::new();
        emitter.emit_function("add", "(i32, i32) -> i32", false);
        assert!(emitter.output().contains("func.func @add"));
    }

    #[test]
    fn test_emit_exported_function() {
        let mut emitter = MlirEmitter::new();
        emitter.emit_function("exported_fn", "(i64) -> i64", true);
        let output = emitter.output();
        assert!(output.contains("func.func @exported_fn"));
        assert!(output.contains("chimera.export"));
    }

    #[test]
    fn test_emit_call() {
        let mut emitter = MlirEmitter::new();
        emitter.emit_call(Some("result"), "might_fail", &["val"], Some("i64"));
        assert!(emitter.output().contains("func.call @might_fail"));
    }

    #[test]
    fn test_full_module_output() {
        let mut emitter = MlirEmitter::new();
        emitter.emit_module_header("test_module", "zig");
        emitter.emit_constant("c1", "1", "i32");
        emitter.emit_constant("c2", "2", "i32");
        emitter.emit_add("result", "c1", "c2", "i32");
        let output = emitter.output();
        assert!(output.contains("module @test_module"));
        assert!(output.contains("arith.constant"));
        assert!(output.contains("arith.addi"));
    }
}
