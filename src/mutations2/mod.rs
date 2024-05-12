pub mod local;
pub mod global;

#[derive(Debug)]
pub enum AreaType {
    Local, Global,
}

pub trait Mutation: Sync + Send {
    fn get_name(&self) -> String;
    fn get_details(&self) -> String;
    fn bend(&self, to_mutate: &mut [u8]);
    fn get_type(&self) -> AreaType;
    fn get_chunksize(&self) -> usize;
}

pub enum MutationKind {
    Chaos,
    Expand,
    Compress,
    Accelerate,
    Increment,
    Loop,
    Multiply,
    Reverse,
    Shuffle,
    Voidout,
    Swap,
    Shift,
}

impl From<&str> for MutationKind {
    fn from(value: &str) -> Self {
        match value {
            "chaos" => MutationKind::Chaos,
            "expand" => MutationKind::Expand,
            "compress" => MutationKind::Compress,
            "accelerate" => MutationKind::Accelerate,
            "increment" => MutationKind::Increment,
            "loop" => MutationKind::Loop,
            "multiply" => MutationKind::Multiply,
            "reverse" => MutationKind::Reverse,
            "shuffle" => MutationKind::Shuffle,
            "voidout" => MutationKind::Voidout,
            "swap" => MutationKind::Swap,
            "shift" => MutationKind::Shift,
            _ => panic!("[{}] is not a valid mutation.", value),
        }
    }
}