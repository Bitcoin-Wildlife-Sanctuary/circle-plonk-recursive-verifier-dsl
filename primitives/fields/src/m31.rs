use circle_plonk_dsl_constraint_system::var::{AllocVar, AllocationMode, Var};
use circle_plonk_dsl_constraint_system::ConstraintSystemRef;
use num_traits::{One, Zero};
use std::ops::{Add, Mul, Neg, Sub};
use stwo_prover::core::fields::m31::M31;

#[derive(Debug, Clone)]
pub struct M31Var {
    pub cs: ConstraintSystemRef,
    pub value: M31,
    pub variable: usize,
}

impl Var for M31Var {
    type Value = M31;

    fn cs(&self) -> ConstraintSystemRef {
        self.cs.clone()
    }
}

impl AllocVar for M31Var {
    fn new_variables(cs: &ConstraintSystemRef, value: &Self::Value, mode: AllocationMode) -> Self {
        if mode != AllocationMode::Constant {
            Self {
                cs: cs.clone(),
                value: *value,
                variable: cs.new_m31(*value, mode),
            }
        } else {
            Self::new_constant(cs, value)
        }
    }

    fn new_constant(cs: &ConstraintSystemRef, value: &Self::Value) -> Self {
        if value.is_zero() {
            return Self::zero(&cs);
        }
        if value.is_one() {
            return Self::one(&cs);
        }

        let exist = cs.get_cache(format!("m31 {}", value.0));
        if let Some(variable) = exist {
            Self {
                cs: cs.clone(),
                value: *value,
                variable,
            }
        } else {
            let res = Self {
                cs: cs.clone(),
                value: *value,
                variable: cs.new_m31(*value, AllocationMode::Constant),
            };
            cs.set_cache(format!("m31 {}", value.0), res.variable);
            res
        }
    }
}

impl Add<&M31Var> for &M31Var {
    type Output = M31Var;

    fn add(self, rhs: &M31Var) -> M31Var {
        let cs = self.cs.and(&rhs.cs);
        let value = self.value + rhs.value;
        let variable = cs.add(self.variable, rhs.variable);

        M31Var {
            cs,
            value,
            variable,
        }
    }
}

impl Sub<&M31Var> for &M31Var {
    type Output = M31Var;

    fn sub(self, rhs: &M31Var) -> M31Var {
        self + &(-rhs)
    }
}

impl Neg for &M31Var {
    type Output = M31Var;

    fn neg(self) -> M31Var {
        let value = -self.value;
        let variable = self.cs.mul_constant(self.variable, M31::one().neg());

        M31Var {
            cs: self.cs.clone(),
            value,
            variable,
        }
    }
}

impl Mul<&M31Var> for &M31Var {
    type Output = M31Var;

    fn mul(self, rhs: &M31Var) -> M31Var {
        let cs = self.cs.and(&rhs.cs);
        let value = self.value * rhs.value;
        let variable = cs.mul(self.variable, rhs.variable);

        M31Var {
            cs,
            value,
            variable,
        }
    }
}

impl M31Var {
    pub fn zero(cs: &ConstraintSystemRef) -> M31Var {
        M31Var {
            cs: cs.clone(),
            value: M31::zero(),
            variable: 0,
        }
    }

    pub fn one(cs: &ConstraintSystemRef) -> M31Var {
        M31Var {
            cs: cs.clone(),
            value: M31::one(),
            variable: 1,
        }
    }

    pub fn equalverify(&self, rhs: &M31Var) {
        assert_eq!(self.value, rhs.value);
        let cs = self.cs.and(&rhs.cs);
        cs.insert_gate(self.variable, 0, rhs.variable, M31::one());
    }

    pub fn inv(&self) -> M31Var {
        let cs = self.cs.clone();

        let value = self.value.inverse();
        let res = M31Var::new_witness(&cs, &value);
        cs.insert_gate(self.variable, res.variable, 1, M31::zero());

        res
    }

    pub fn mul_constant(&self, constant: M31) -> M31Var {
        let cs = self.cs();
        let value = self.value * constant;
        let variable = cs.mul_constant(self.variable, constant);

        M31Var {
            cs,
            value,
            variable,
        }
    }

    pub fn is_eq(&self, rhs: &M31Var) -> M31Var {
        (self - rhs).is_zero()
    }

    pub fn is_zero(&self) -> M31Var {
        let cs = self.cs();
        let inv = M31Var::new_witness(&self.cs, &{
            if self.value.is_zero() {
                M31::zero()
            } else {
                self.value.inverse()
            }
        });
        let out = &(self * &inv).neg() + &M31Var::one(&cs);
        cs.insert_gate(self.variable, out.variable, 0, M31::zero());

        out
    }
}
