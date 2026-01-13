use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum PlayerActionType {
    Jump,
    PrimaryUse,
    SecondaryUse,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum IKSolverType {
    Fabrik,
    TwoBone
}
impl Default for IKSolverType {
    fn default() -> Self {
        IKSolverType::Fabrik
    }
}
