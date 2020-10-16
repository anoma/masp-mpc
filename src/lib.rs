pub mod bridge {
    //! Bridges two incompatible bellman APIs
    //! based on phase2-bn254 code

    use bellman_ce::pairing::ff::{Field, PrimeField, PrimeFieldRepr};
    use std::fmt::Write;
    use std::marker::PhantomData;

    // based on https://github.com/matter-labs/ff/blob/c54018e79bd4ee59cd6e9720364e3682b34135e1/src/lib.rs
    fn convert_scalar<Scalar: ff::PrimeField, E: bellman_ce::pairing::Engine>(x: Scalar) -> E::Fr {
        let mut value = String::new();
        write!(&mut value, "{:?}", x).expect("Scalar write failed");

        let value = &value[2..];
        let mut buf = hex::decode(&value)
            .map_err(|_| format!("could not decode hex: {}", value))
            .unwrap();
        let mut repr = <E::Fr as PrimeField>::Repr::default();
        let required_length = repr.as_ref().len() * 8;
        buf.reverse();
        buf.resize(required_length, 0);

        repr.read_le(&buf[..])
            .map_err(|e| format!("could not read {}: {}", value, &e))
            .unwrap();

        E::Fr::from_repr(repr)
            .map_err(|e| format!("could not convert into prime field: {}: {}", value, &e))
            .expect("Scalar -> Fr failed")
    }

    struct BridgeConstraintSystem<
        'a,
        E: bellman_ce::pairing::Engine,
        CS: bellman_ce::ConstraintSystem<E>,
    > {
        pub cs: &'a mut CS,
        pub inputs: Vec<bellman_ce::Variable>,
        pub aux: Vec<bellman_ce::Variable>,
        pub _engine: PhantomData<E>,
    }

    impl<E: bellman_ce::pairing::Engine, CS: bellman_ce::ConstraintSystem<E>>
        BridgeConstraintSystem<'_, E, CS>
    {
        fn bridge(&self, var: bellman::Variable) -> bellman_ce::Variable {
            match var.get_unchecked() {
                bellman::Index::Input(id) => self.inputs[id],
                bellman::Index::Aux(id) => self.aux[id],
            }
        }
    }

    impl<
            Scalar: ff::PrimeField, // + bellman_ce::pairing::ff::PrimeField + bellman_ce::pairing::ff::SqrtField,
            E: bellman_ce::pairing::Engine, // + bellman_ce::pairing::ff::ScalarEngine<Fr = Scalar>,
            CS: bellman_ce::ConstraintSystem<E>,
        > bellman::ConstraintSystem<Scalar> for BridgeConstraintSystem<'_, E, CS>
    {
        type Root = Self;

        fn alloc<F, A, AR>(
            &mut self,
            annotation: A,
            _: F,
        ) -> Result<bellman::Variable, bellman::SynthesisError>
        where
            F: FnOnce() -> Result<Scalar, bellman::SynthesisError>,
            A: FnOnce() -> AR,
            AR: Into<String>,
        {
            let index = self.aux.len();
            let var = bellman::Variable::new_unchecked(bellman::Index::Aux(index));
            //let val = f()?;
            let ce_var = self
                .cs
                .alloc(annotation, || Ok(E::Fr::one()))
                .expect("Bad bellman_ce alloc_input");
            self.aux.push(ce_var);

            Ok(var)
        }

        fn alloc_input<F, A, AR>(
            &mut self,
            annotation: A,
            _: F,
        ) -> Result<bellman::Variable, bellman::SynthesisError>
        where
            F: FnOnce() -> Result<Scalar, bellman::SynthesisError>,
            A: FnOnce() -> AR,
            AR: Into<String>,
        {
            let index = self.inputs.len();
            let var = bellman::Variable::new_unchecked(bellman::Index::Input(index));
            //let val = f().expect("Bad assignment");
            let ce_var = self
                .cs
                .alloc_input(annotation, || Ok(E::Fr::one()))
                .expect("Bad bellman_ce alloc_input");
            self.inputs.push(ce_var);

            Ok(var)
        }

        fn enforce<A, AR, LA, LB, LC>(&mut self, annotation: A, a: LA, b: LB, c: LC)
        where
            A: FnOnce() -> AR,
            AR: Into<String>,
            LA: FnOnce(bellman::LinearCombination<Scalar>) -> bellman::LinearCombination<Scalar>,
            LB: FnOnce(bellman::LinearCombination<Scalar>) -> bellman::LinearCombination<Scalar>,
            LC: FnOnce(bellman::LinearCombination<Scalar>) -> bellman::LinearCombination<Scalar>,
        {
            let convert_lc =  //<Scalar: ff::PrimeField, E: bellman_ce::pairing::Engine>(
            |lc: bellman::LinearCombination<Scalar>|
         -> bellman_ce::LinearCombination<E> {
            //let make_lc = |lc_data: Vec<(usize, E::Fr)>|
            lc.as_ref().iter().fold(
                bellman_ce::LinearCombination::<E>::zero(),
                |lc: bellman_ce::LinearCombination<E>, (variable, coeff)| {
                    lc + (convert_scalar::<Scalar, E>(*coeff), self.bridge(*variable))
                },
            )
        };

            let a = convert_lc(a(bellman::LinearCombination::zero()));
            let b = convert_lc(b(bellman::LinearCombination::zero()));
            let c = convert_lc(c(bellman::LinearCombination::zero()));

            self.cs.enforce(annotation, |_| a, |_| b, |_| c);
        }

        fn push_namespace<NR, N>(&mut self, _: N)
        where
            NR: Into<String>,
            N: FnOnce() -> NR,
        {
            // Do nothing; we don't care about namespaces in this context.
        }

        fn pop_namespace(&mut self) {
            // Do nothing; we don't care about namespaces in this context.
        }

        fn get_root(&mut self) -> &mut Self::Root {
            self
        }
    }

    pub struct BridgeCircuit<Scalar: ff::PrimeField, C: bellman::Circuit<Scalar>> {
        pub circuit: C,
        pub _scalar: PhantomData<Scalar>,
    }

    impl<E: bellman_ce::pairing::Engine, Scalar: ff::PrimeField, C: bellman::Circuit<Scalar>>
        bellman_ce::Circuit<E> for BridgeCircuit<Scalar, C>
    {
        fn synthesize<CS: bellman_ce::ConstraintSystem<E>>(
            self,
            cs: &mut CS,
        ) -> Result<(), bellman_ce::SynthesisError> {
            let mut bcs = BridgeConstraintSystem::<E, CS> {
                cs,
                inputs: vec![],
                aux: vec![],
                _engine: PhantomData::<E>,
            };
            bcs.inputs.push(bellman_ce::Variable::new_unchecked(
                bellman_ce::Index::Input(0),
            ));

            Ok(self
                .circuit
                .synthesize(&mut bcs)
                .expect("Failed to synthesize"))
        }
    }
}

pub mod test;
