use halo2_proofs::{arithmetic::FieldExt, circuit::*, plonk::*};

// TODO: add documentation
struct ExpConfig {
    pub advice: Column<Advice>,
    pub selector: Selector,
    pub instance: Column<Instance>
}
