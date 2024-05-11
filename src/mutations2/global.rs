use moveslice::Moveslice;
use rand::{thread_rng, Rng};

use super::{AreaType, Mutation};

pub struct Swap {
    pub chunksize: usize,
}

impl Mutation for Swap {
    fn get_name(&self) -> String {
        "swap".into()
    }

    fn get_details(&self) -> String {
        "[none]".into()
    }

    fn bend(&self, to_mutate: &mut [u8]) {
        let mut rng = thread_rng();
        let split_index = rng.gen_range(self.chunksize, to_mutate.as_mut().len());
        let (l, r) = to_mutate.as_mut().split_at_mut(split_index);

        let slice_1_idx = rng.gen_range(0, l.len() - self.chunksize);
        let slice_2_idx = rng.gen_range(0, r.len() - self.chunksize);

        let slice_1 = &mut l[slice_1_idx..slice_1_idx+self.chunksize];
        let slice_2 = &mut r[slice_2_idx..slice_2_idx+self.chunksize];
        slice_1.swap_with_slice(slice_2);
    }

    fn get_type(&self) -> AreaType {
        AreaType::Global
    }

    fn get_chunksize(&self) -> usize {
        self.chunksize
    }
}

pub struct Shift {
    pub from: usize,
    pub by: usize,
    pub chunksize: usize,
}

impl Mutation for Shift {
    fn get_name(&self) -> String {
        "shift".into()
    }

    fn get_details(&self) -> String {
        format!("[from={} by={}]", self.from, self.by)
    }

    fn bend(&self, to_mutate: &mut [u8]) {
        to_mutate.as_mut().moveslice(self.from..self.from+self.chunksize, self.from+self.by);
    }

    fn get_type(&self) -> AreaType {
        AreaType::Global
    }

    fn get_chunksize(&self) -> usize {
        self.chunksize
    }
}