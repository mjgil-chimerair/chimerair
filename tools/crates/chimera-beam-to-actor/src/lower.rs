//! Beam to Actor lowering.
//!
//! Converts BEAM dialect operations to Actor dialect.

use chimera_beam_dialect::ops::BeamOpKind;
use chimera_beam_dialect::{BeamOp, BeamType};
use chimera_beam_effects::{EffectInfo, EffectType};
use serde::{Deserialize, Serialize};

/// Result of lowering operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoweringResult {
    /// Whether lowering succeeded.
    pub success: bool,
    /// Output actor operations (IR representation).
    pub output: Vec<String>,
    /// Effects preserved during lowering.
    pub effects: Vec<EffectInfo>,
    /// Errors if any.
    pub errors: Vec<String>,
}

impl LoweringResult {
    /// Create a successful result.
    pub fn success(output: Vec<String>) -> Self {
        LoweringResult {
            success: true,
            output,
            effects: vec![],
            errors: vec![],
        }
    }

    /// Create a failed result.
    pub fn error(msg: impl Into<String>) -> Self {
        LoweringResult {
            success: false,
            output: vec![],
            effects: vec![],
            errors: vec![msg.into()],
        }
    }

    /// Add an effect.
    pub fn add_effect(&mut self, effect: EffectInfo) {
        self.effects.push(effect);
    }
}

/// Lowerer from BEAM to Actor dialect.
#[derive(Debug, Clone)]
pub struct BeamToActorLowerer {
    /// Enable effect preservation.
    preserve_effects: bool,
    /// Enable strict type checking.
    strict_types: bool,
}

impl BeamToActorLowerer {
    /// Create a new lowerer.
    pub fn new() -> Self {
        BeamToActorLowerer {
            preserve_effects: true,
            strict_types: true,
        }
    }

    /// Create with options.
    pub fn with_options(preserve_effects: bool, strict_types: bool) -> Self {
        BeamToActorLowerer {
            preserve_effects,
            strict_types,
        }
    }

    /// Lower a BEAM operation to Actor.
    pub fn lower_op(&self, op: &BeamOp) -> LoweringResult {
        match op.kind {
            BeamOpKind::Spawn => self.lower_spawn(op),
            BeamOpKind::SpawnLink => self.lower_spawn_link(op),
            BeamOpKind::SpawnMonitor => self.lower_spawn_monitor(op),
            BeamOpKind::Send => self.lower_send(op),
            BeamOpKind::Recv => self.lower_receive(op),
            BeamOpKind::Link => self.lower_link(op),
            BeamOpKind::Unlink => self.lower_unlink(op),
            BeamOpKind::Monitor => self.lower_monitor(op),
            BeamOpKind::Demonitor => self.lower_demonitor(op),
            BeamOpKind::Exit => self.lower_exit(op),
            BeamOpKind::Exit2 => self.lower_exit2(op),
            BeamOpKind::Kill => self.lower_kill(op),
            BeamOpKind::Register => self.lower_register(op),
            BeamOpKind::Unregister => self.lower_unregister(op),
            BeamOpKind::Whereis => self.lower_whereis(op),
            BeamOpKind::SupervisorStart => self.lower_supervisor_start(op),
            _ => LoweringResult::error(format!("unsupported op: {:?}", op.kind)),
        }
    }

    /// Lower spawn.
    fn lower_spawn(&self, op: &BeamOp) -> LoweringResult {
        let mut result = LoweringResult::success(vec![format!(
            "actor.spawn {}:{}",
            op.attributes
                .get(0)
                .map(|t| t.1.clone())
                .unwrap_or_default(),
            op.attributes
                .get(1)
                .map(|t| t.1.clone())
                .unwrap_or_default()
        )]);
        if self.preserve_effects {
            result.add_effect(EffectInfo::at(EffectType::ProcessSpawn, Default::default()));
        }
        result
    }

    /// Lower spawn_link.
    fn lower_spawn_link(&self, op: &BeamOp) -> LoweringResult {
        let mut result = LoweringResult::success(vec![format!(
            "actor.spawn_link {}:{}",
            op.attributes
                .get(0)
                .map(|t| t.1.clone())
                .unwrap_or_default(),
            op.attributes
                .get(1)
                .map(|t| t.1.clone())
                .unwrap_or_default()
        )]);
        if self.preserve_effects {
            result.add_effect(EffectInfo::at(EffectType::ProcessSpawn, Default::default()));
            result.add_effect(EffectInfo::at(EffectType::ProcessLink, Default::default()));
        }
        result
    }

    /// Lower spawn_monitor.
    fn lower_spawn_monitor(&self, op: &BeamOp) -> LoweringResult {
        let mut result = LoweringResult::success(vec![format!(
            "actor.spawn_monitor {}:{}",
            op.attributes
                .get(0)
                .map(|t| t.1.clone())
                .unwrap_or_default(),
            op.attributes
                .get(1)
                .map(|t| t.1.clone())
                .unwrap_or_default()
        )]);
        if self.preserve_effects {
            result.add_effect(EffectInfo::at(EffectType::ProcessSpawn, Default::default()));
            result.add_effect(EffectInfo::at(
                EffectType::ProcessMonitor,
                Default::default(),
            ));
        }
        result
    }

    /// Lower send.
    fn lower_send(&self, op: &BeamOp) -> LoweringResult {
        let mut result = LoweringResult::success(vec!["actor.send".to_string()]);
        if self.preserve_effects {
            result.add_effect(EffectInfo::at(EffectType::MessageSend, Default::default()));
        }
        result
    }

    /// Lower receive.
    fn lower_receive(&self, op: &BeamOp) -> LoweringResult {
        let mut result = LoweringResult::success(vec!["actor.receive".to_string()]);
        if self.preserve_effects {
            result.add_effect(EffectInfo::at(
                EffectType::MessageReceive,
                Default::default(),
            ));
        }
        result
    }

    /// Lower link.
    fn lower_link(&self, op: &BeamOp) -> LoweringResult {
        let mut result = LoweringResult::success(vec!["actor.link".to_string()]);
        if self.preserve_effects {
            result.add_effect(EffectInfo::at(EffectType::ProcessLink, Default::default()));
        }
        result
    }

    /// Lower unlink.
    fn lower_unlink(&self, _op: &BeamOp) -> LoweringResult {
        LoweringResult::success(vec!["actor.unlink".to_string()])
    }

    /// Lower monitor.
    fn lower_monitor(&self, op: &BeamOp) -> LoweringResult {
        let mut result = LoweringResult::success(vec!["actor.monitor".to_string()]);
        if self.preserve_effects {
            result.add_effect(EffectInfo::at(
                EffectType::ProcessMonitor,
                Default::default(),
            ));
        }
        result
    }

    /// Lower demonitor.
    fn lower_demonitor(&self, _op: &BeamOp) -> LoweringResult {
        LoweringResult::success(vec!["actor.demonitor".to_string()])
    }

    /// Lower exit.
    fn lower_exit(&self, op: &BeamOp) -> LoweringResult {
        let mut result = LoweringResult::success(vec!["actor.exit".to_string()]);
        if self.preserve_effects {
            result.add_effect(EffectInfo::at(EffectType::ProcessExit, Default::default()));
        }
        result
    }

    /// Lower exit2.
    fn lower_exit2(&self, op: &BeamOp) -> LoweringResult {
        let mut result = LoweringResult::success(vec!["actor.exit2".to_string()]);
        if self.preserve_effects {
            result.add_effect(EffectInfo::at(EffectType::ProcessExit, Default::default()));
        }
        result
    }

    /// Lower kill.
    fn lower_kill(&self, op: &BeamOp) -> LoweringResult {
        let mut result = LoweringResult::success(vec!["actor.kill".to_string()]);
        if self.preserve_effects {
            result.add_effect(EffectInfo::at(EffectType::ProcessExit, Default::default()));
        }
        result
    }

    /// Lower register.
    fn lower_register(&self, _op: &BeamOp) -> LoweringResult {
        let mut result = LoweringResult::success(vec!["actor.register".to_string()]);
        if self.preserve_effects {
            result.add_effect(EffectInfo::at(EffectType::Registry, Default::default()));
        }
        result
    }

    /// Lower unregister.
    fn lower_unregister(&self, _op: &BeamOp) -> LoweringResult {
        let mut result = LoweringResult::success(vec!["actor.unregister".to_string()]);
        if self.preserve_effects {
            result.add_effect(EffectInfo::at(EffectType::Registry, Default::default()));
        }
        result
    }

    /// Lower whereis.
    fn lower_whereis(&self, _op: &BeamOp) -> LoweringResult {
        LoweringResult::success(vec!["actor.whereis".to_string()])
    }

    /// Lower supervisor_start.
    fn lower_supervisor_start(&self, op: &BeamOp) -> LoweringResult {
        LoweringResult::success(vec!["actor.supervisor_start".to_string()])
    }
}

impl Default for BeamToActorLowerer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lower_spawn() {
        let lowerer = BeamToActorLowerer::new();
        let op = BeamOp::spawn("mod".to_string(), "fun".to_string());
        let result = lowerer.lower_op(&op);
        assert!(result.success);
        assert_eq!(result.output.len(), 1);
        assert!(result.effects.len() >= 1);
    }

    #[test]
    fn test_lower_spawn_link() {
        let lowerer = BeamToActorLowerer::new();
        let op = BeamOp::spawn_link("mod".to_string(), "fun".to_string());
        let result = lowerer.lower_op(&op);
        assert!(result.success);
        assert!(result.effects.len() >= 2); // spawn + link
    }

    #[test]
    fn test_lower_send() {
        let lowerer = BeamToActorLowerer::new();
        let op = BeamOp::send(BeamType::pid(), BeamType::atom());
        let result = lowerer.lower_op(&op);
        assert!(result.success);
    }

    #[test]
    fn test_lower_receive() {
        let lowerer = BeamToActorLowerer::new();
        let op = BeamOp::receive(3);
        let result = lowerer.lower_op(&op);
        assert!(result.success);
    }

    #[test]
    fn test_lower_link() {
        let lowerer = BeamToActorLowerer::new();
        let op = BeamOp::link(BeamType::pid());
        let result = lowerer.lower_op(&op);
        assert!(result.success);
    }

    #[test]
    fn test_lower_monitor() {
        let lowerer = BeamToActorLowerer::new();
        let op = BeamOp::monitor(BeamType::pid());
        let result = lowerer.lower_op(&op);
        assert!(result.success);
    }

    #[test]
    fn test_lower_exit() {
        let lowerer = BeamToActorLowerer::new();
        let op = BeamOp::exit(BeamType::atom());
        let result = lowerer.lower_op(&op);
        assert!(result.success);
    }

    #[test]
    fn test_lower_register() {
        let lowerer = BeamToActorLowerer::new();
        let op = BeamOp::register(BeamType::atom(), BeamType::pid());
        let result = lowerer.lower_op(&op);
        assert!(result.success);
    }

    #[test]
    fn test_lower_unsupported() {
        let lowerer = BeamToActorLowerer::new();
        let op = BeamOp {
            kind: BeamOpKind::Now,
            name: "beam.now".to_string(),
            inputs: vec![],
            outputs: vec![],
            attributes: vec![],
            regions: 0,
        };
        let result = lowerer.lower_op(&op);
        assert!(!result.success);
    }

    #[test]
    fn test_lowering_result_success() {
        let result = LoweringResult::success(vec!["op1".to_string(), "op2".to_string()]);
        assert!(result.success);
        assert_eq!(result.output.len(), 2);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_lowering_result_error() {
        let result = LoweringResult::error("test error");
        assert!(!result.success);
        assert!(result.output.is_empty());
        assert_eq!(result.errors.len(), 1);
    }
}
