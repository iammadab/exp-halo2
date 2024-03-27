use std::marker::PhantomData;
use halo2_proofs::{arithmetic::FieldExt, circuit::*, plonk::*};
use halo2_proofs::poly::Rotation;

// TODO: add proper documentation across all

// TODO: add documentation
struct ExpConfig {
    // TODO: figure out if there is a better way to do this:
    //  - maybe don't use two columns, compress to 1
    //  - or bit decomposition for super fast exponentiation
    pub advice: [Column<Advice>; 3],
    pub selector: Selector,
    pub instance: Column<Instance>
}

// TODO: add documentation
struct ExpChip<F: FieldExt> {
    config: ExpConfig,
    _marker: PhantomData<F>
}

impl<F: FieldExt> ExpChip<F> {
    // TODO: is this used?
    fn construct(config: ExpConfig) -> Self {
        Self {
            config,
            _marker: PhantomData
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> ExpConfig {
        // TODO: add better comments here
        // TODO: maybe name this to result column
        let result_column = meta.advice_column();
        let exponent_column= meta.advice_column();
        let base_column = meta.advice_column();
        let sel = meta.selector();
        let instance = meta.instance_column();

        meta.enable_equality(result_column);
        meta.enable_equality(exponent_column);
        meta.enable_equality(base_column);
        meta.enable_equality(sel);
        meta.enable_equality(instance);

        meta.create_gate("exp", |meta| {
            let s = meta.query_selector(sel);
            let prev_running_result = meta.query_advice(result_column, Rotation::cur());
            let current_result = meta.query_advice(result_column, Rotation::next());
            let prev_exp = meta.query_advice(exponent_column, Rotation::cur());
            let current_exp = meta.query_advice(exponent_column, Rotation::next());
            let prev_base = meta.query_advice(base_column, Rotation::cur());
            let curr_base = meta.query_advice(base_column, Rotation::prev());

            vec![
                s * ((prev_running_result * base) - current_result),
                s * ((current_exp - prev_exp) - F::one()),
                curr_base - prev_base
            ]
        });

        ExpConfig {
            advice: [result_column, exponent_column, base_column],
            selector: sel,
            instance
        }
    }
}
