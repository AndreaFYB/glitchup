use rand::{seq::SliceRandom, thread_rng, Rng};

use super::{AreaType, Mutation};

pub struct Chaos {
    pub chunksize: usize,
}

impl Mutation for Chaos {
    fn get_name(&self) -> String {
        "chaos".into()
    }

    fn get_details(&self) -> String {
        "[none]".into()
    }

    fn bend(&self, to_mutate: &mut [u8]) {
        let mut rng = thread_rng();
        for byte in to_mutate.as_mut().iter_mut() {
            *byte = rng.gen()
        }
    }

    fn get_type(&self) -> AreaType {
        AreaType::Local
    }

    fn get_chunksize(&self) -> usize {
        self.chunksize
    }
}

pub struct Compress {
    pub by: usize,
    pub chunksize: usize,
}

impl Mutation for Compress {
    fn get_name(&self) -> String {
        "compress".into()
    }

    fn get_details(&self) -> String {
        format!("[by={}]", self.by)
    }

    fn bend(&self, to_mutate: &mut [u8]) {
        let length = to_mutate.as_mut().len();
        let mut replacement = vec![0; length];

        for idx in 0..length {
            replacement[idx] = to_mutate.as_mut()[idx / self.by];
        }

        to_mutate.as_mut().swap_with_slice(&mut replacement);
    }

    fn get_type(&self) -> AreaType {
        AreaType::Local
    }

    fn get_chunksize(&self) -> usize {
        self.chunksize
    }
}

pub struct Accelerate {
    pub from: usize,
    pub by: usize,
    pub after: usize,
    pub chunksize: usize,
}

impl Mutation for Accelerate {
    fn get_name(&self) -> String {
        "accelerate".into()
    }

    fn get_details(&self) -> String {
        format!("[by={} after={}]", self.by, self.after)
    }

    fn bend(&self, to_mutate: &mut [u8]) {
        let increment = self.from;
        for (i, byte) in to_mutate.as_mut().iter_mut().enumerate() {
            *byte = (((*byte as usize) + self.from + (self.by * (i/self.after))) % 256) as u8;
        }
    }

    fn get_type(&self) -> AreaType {
        AreaType::Local
    }

    fn get_chunksize(&self) -> usize {
        self.chunksize
    }
}

pub struct Increment {
    pub by: usize,
    pub chunksize: usize,
}

impl Mutation for Increment {
    fn get_name(&self) -> String {
        "increment".into()
    }

    fn get_details(&self) -> String {
        format!("[by={}]", self.by)
    }

    fn bend(&self, to_mutate: &mut [u8]) {
        for byte in to_mutate.as_mut().iter_mut() {
            *byte = ((*byte as usize + self.by) % 256) as u8
        }
    }

    fn get_type(&self) -> AreaType {
        AreaType::Local
    }

    fn get_chunksize(&self) -> usize {
        self.chunksize
    }
}

pub struct Loop {
    pub loopsize: usize,
    pub chunksize: usize,
}

impl Mutation for Loop {
    fn get_name(&self) -> String {
        "loop".into()
    }

    fn get_details(&self) -> String {
        format!("[loopsize={}]", self.loopsize)
    }

    fn bend(&self, to_mutate: &mut [u8]) {
        let (l, r) = to_mutate.as_mut().split_at_mut(self.loopsize);
        for (i, byte) in r.as_mut().iter_mut().enumerate() {
            *byte = l[i % l.len()]
        }
    }

    fn get_type(&self) -> AreaType {
        AreaType::Local
    }

    fn get_chunksize(&self) -> usize {
        self.chunksize
    }
}

pub struct Multiply {
    pub by: f64,
    pub chunksize: usize,
}

impl Mutation for Multiply {
    fn get_name(&self) -> String {
        "multiply".into()
    }

    fn get_details(&self) -> String {
        format!("[by={}]", self.by)
    }

    fn bend(&self, to_mutate: &mut [u8]) {
        let mut rng = thread_rng();
        for byte in to_mutate.as_mut().iter_mut() {
            *byte = ((((*byte as f64) * self.by) as usize) % 256) as u8
        }
    }

    fn get_type(&self) -> AreaType {
        AreaType::Local
    }

    fn get_chunksize(&self) -> usize {
        self.chunksize
    }
}

pub struct Reverse {
    pub chunksize: usize,
}

impl Mutation for Reverse {
    fn get_name(&self) -> String {
        "reverse".into()
    }

    fn get_details(&self) -> String {
        "[none]".into()
    }

    fn bend(&self, to_mutate: &mut [u8]) {
        to_mutate.as_mut().reverse();
    }

    fn get_type(&self) -> AreaType {
        AreaType::Local
    }

    fn get_chunksize(&self) -> usize {
        self.chunksize
    }
}

pub struct Shuffle {
    pub chunksize: usize,
}

impl Mutation for Shuffle {
    fn get_name(&self) -> String {
        "shuffle".into()
    }

    fn get_details(&self) -> String {
        "[none]".into()
    }

    fn bend(&self, to_mutate: &mut [u8]) {
        to_mutate.as_mut().shuffle(&mut thread_rng())
    }

    fn get_type(&self) -> AreaType {
        AreaType::Local
    }

    fn get_chunksize(&self) -> usize {
        self.chunksize
    }
}

pub struct Voidout {
    pub chunksize: usize,
}

impl Mutation for Voidout {
    fn get_name(&self) -> String {
        "voidout".into()
    }

    fn get_details(&self) -> String {
        "[none]".into()
    }

    fn bend(&self, to_mutate: &mut [u8]) {
        for byte in to_mutate.as_mut().iter_mut() {
            *byte = 0
        }
    }

    fn get_type(&self) -> AreaType {
        AreaType::Local
    }

    fn get_chunksize(&self) -> usize {
        self.chunksize
    }
}