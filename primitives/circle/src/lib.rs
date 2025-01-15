use circle_plonk_dsl_channel::ChannelVar;
use circle_plonk_dsl_constraint_system::dvar::{AllocVar, AllocationMode, DVar};
use circle_plonk_dsl_constraint_system::ConstraintSystemRef;
use circle_plonk_dsl_fields::{M31Var, QM31Var};
use std::ops::{Add, Neg};
use stwo_prover::core::circle::CirclePoint;
use stwo_prover::core::fields::qm31::SecureField;

#[derive(Clone)]
pub struct CirclePointQM31Var {
    pub x: QM31Var,
    pub y: QM31Var,
}

impl DVar for CirclePointQM31Var {
    type Value = CirclePoint<SecureField>;

    fn cs(&self) -> ConstraintSystemRef {
        self.x.cs().and(&self.y.cs())
    }
}

impl AllocVar for CirclePointQM31Var {
    fn new_variables(cs: &ConstraintSystemRef, value: &Self::Value, mode: AllocationMode) -> Self {
        let x = QM31Var::new_variables(cs, &value.x, mode);
        let y = QM31Var::new_variables(cs, &value.y, mode);
        Self { x, y }
    }
}

impl CirclePointQM31Var {
    pub fn from_t(t: &QM31Var) -> Self {
        let cs = t.cs();

        let t_doubled = t + t;
        let t_squared = t * t;

        let t_squared_plus_1 = t_squared.add(&M31Var::one(&cs));
        let t_squared_plus_1_inverse = t_squared_plus_1.inv();

        let one_minus_t_squared_minus = t_squared.neg().add(&M31Var::one(&cs));

        let x = &one_minus_t_squared_minus * &t_squared_plus_1_inverse;
        let y = &t_doubled * &t_squared_plus_1_inverse;

        Self { x, y }
    }

    pub fn from_channel(channel: &mut ChannelVar) -> Self {
        let [t, _] = channel.get_felts();
        Self::from_t(&t)
    }
}