use bellman::{Circuit, ConstraintSystem, SynthesisError};
use bls12_381::Scalar;

struct TestCircuit {
    x: Option<Scalar>,
}
impl Circuit<Scalar> for TestCircuit {
    fn synthesize<CS: ConstraintSystem<Scalar>>(self, cs: &mut CS) -> Result<(), SynthesisError> {
        let mut x_value = self.x;
        let mut x = cs.alloc(|| "x", || x_value.ok_or(SynthesisError::AssignmentMissing))?;

        cs.enforce(|| "x = x^2", |lc| lc + x, |lc| lc + x, |lc| lc + x);

        Ok(())
    }
}

#[test]
fn test_test_circuit() {
    use bellman::gadgets::test::*;
    let mut cs = TestConstraintSystem::new();

    let instance = TestCircuit {
        x: Some(Scalar::one()),
    };

    instance.synthesize(&mut cs).unwrap();

    assert!(cs.is_satisfied());
}
