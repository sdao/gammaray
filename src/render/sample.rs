use core::Vec;

#[derive(Clone)]
pub struct Sample {
    pub accum: Vec,
    pub num_samples: usize,
}

impl Sample {
    pub fn zero() -> Sample {
        Sample {accum: Vec::zero(), num_samples: 0}
    }
}
