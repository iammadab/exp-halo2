extern crate core;

use halo2_proofs::plonk::Expression;
use halo2_proofs::poly::Rotation;
use halo2_proofs::{arithmetic::FieldExt, circuit::*, plonk::*};
use std::marker::PhantomData;

// TODO: add proper documentation across all

// TODO: add documentation
#[derive(Clone)]
struct ExpConfig {
    // TODO: figure out if there is a better way to do this:
    //  - maybe don't use two columns, compress to 1
    //  - or bit decomposition for super fast exponentiation
    // can reduce it to two columns, use the first element of the result column as the base
    // but how do I get that into meta.create_gate??
    pub advice: [Column<Advice>; 3],
    pub selector: Selector,
    pub instance: Column<Instance>,
}

// TODO: add documentation
struct ExpChip<F: FieldExt> {
    config: ExpConfig,
    _marker: PhantomData<F>,
}

impl<F: FieldExt> ExpChip<F> {
    // TODO: is this used?
    fn construct(config: ExpConfig) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> ExpConfig {
        // TODO: add better comments here
        // TODO: maybe name this to result column
        let result_column = meta.advice_column();
        let exponent_column = meta.advice_column();
        let base_column = meta.advice_column();
        let sel = meta.selector();
        let instance = meta.instance_column();

        meta.enable_equality(result_column);
        meta.enable_equality(exponent_column);
        meta.enable_equality(base_column);
        meta.enable_equality(instance);

        meta.create_gate("exp", |meta| {
            let s = meta.query_selector(sel);
            let prev_running_result = meta.query_advice(result_column, Rotation::cur());
            let current_result = meta.query_advice(result_column, Rotation::next());
            let prev_exp = meta.query_advice(exponent_column, Rotation::cur());
            let curr_exp = meta.query_advice(exponent_column, Rotation::next());
            let prev_base = meta.query_advice(base_column, Rotation::cur());
            let curr_base = meta.query_advice(base_column, Rotation::next());

            vec![
                s.clone() * ((prev_running_result * prev_base.clone()) - current_result),
                s.clone() * ((prev_exp - curr_exp) - Expression::Constant(F::one())),
                s * (prev_base - curr_base),
            ]
        });

        ExpConfig {
            advice: [result_column, exponent_column, base_column],
            selector: sel,
            instance,
        }
    }

    // TODO: refactor this
    fn assign(&self, mut layouter: impl Layouter<F>) -> Result<AssignedCell<F, F>, Error> {
        // what does one do here?
        // going to be using a single region
        // TODO: look into the hacknode use of regions, see how they handle overlapping elements
        layouter.assign_region(
            || "exp_region",
            |mut region| {
                // first row
                self.config.selector.enable(&mut region, 0)?;
                // copy the base into the first result column cell
                let mut result_cell = region.assign_advice_from_instance(
                    || "result_start",
                    self.config.instance,
                    0,
                    self.config.advice[0],
                    0,
                )?;
                // copy the exponent into the first exponent column cell
                let mut exp_cell = region.assign_advice_from_instance(
                    || "exp_start",
                    self.config.instance,
                    1,
                    self.config.advice[1],
                    0,
                )?;
                // copy the base into the first base_column cell
                let mut base_cell = region.assign_advice_from_instance(
                    || "base_start",
                    self.config.instance,
                    0,
                    self.config.advice[2],
                    0,
                )?;

                // continuously update the rows until some exponent is 1
                // what do we condition the for loop on
                // what do we do in the for loop?
                let mut i = 1;
                while let Some(value) = exp_cell.value() {
                    if value == &F::one() {
                        break;
                    }

                    if value != &F::from(2) {
                        self.config.selector.enable(&mut region, i)?;
                    }

                    let next_result = *result_cell.value().unwrap() * *base_cell.value().unwrap();
                    let next_exp = *exp_cell.value().unwrap() - F::one();
                    let next_base_cell = *base_cell.value().unwrap() + F::zero();

                    // update the table
                    result_cell = region.assign_advice(
                        || "next_result",
                        self.config.advice[0],
                        i,
                        || Ok(next_result),
                    )?;
                    exp_cell = region.assign_advice(
                        || "next_exp",
                        self.config.advice[1],
                        i,
                        || Ok(next_exp),
                    )?;
                    base_cell = region.assign_advice(
                        || "next_base",
                        self.config.advice[2],
                        i,
                        || Ok(next_base_cell),
                    )?;

                    i += 1;
                }

                Ok(result_cell)
            },
        )
    }

    fn expose_public(
        &self,
        mut layouter: impl Layouter<F>,
        cell: &AssignedCell<F, F>,
        instance_column_row: usize,
    ) -> Result<(), Error> {
        layouter.constrain_instance(cell.cell(), self.config.instance, instance_column_row)
    }
}

#[derive(Default)]
struct ExpCircuit<F> {
    _marker: PhantomData<F>,
}

impl<F: FieldExt> Circuit<F> for ExpCircuit<F> {
    type Config = ExpConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        ExpChip::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let chip = ExpChip::construct(config);
        let result = chip.assign(layouter.namespace(|| "exp circuit"))?;
        chip.expose_public(layouter.namespace(|| "boundary-constraint"), &result, 2)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::ExpCircuit;
    use halo2_proofs::dev::MockProver;
    use halo2_proofs::pasta::Fp;

    #[test]
    fn test_exp_circuit() {
        let k = 4;
        let public_inputs = vec![Fp::from(3), Fp::from(4), Fp::from(81)];
        let circuit = ExpCircuit::<Fp>::default();
        let prover = MockProver::run(k, &circuit, vec![public_inputs]).unwrap();
        prover.assert_satisfied();
    }
}
