#![feature(trait_alias)]

use std::error::Error;
use std::marker::PhantomData;

use rand::distributions::uniform::SampleBorrow;
use rand::thread_rng;
use rand::{distributions::uniform::SampleUniform, Rng};
use rand::seq::SliceRandom;
use serde_yaml::Mapping;

use crate::mutations2::local::Compress;
use crate::{mutations, mutations2::{global::{Shift, Swap}, local::{Accelerate, Chaos, Expand, Increment, Loop, Multiply, Reverse, Shuffle, Voidout}, Mutation, MutationKind}};

use super::ioutils::{load_as_bytes_from_path, load_as_bytes_from_url};

pub struct AppCfg {
    pub output_path: String,
    pub output_n: usize,
}

pub fn parse_app_cfg(root_value: &serde_yaml::Value) -> AppCfg {
    let config = root_value
        .get("config").expect("[config] was not present. is required.")
        .as_mapping().expect("[config] must be a map. wasn't.");

    let output = config
        .get("output").expect("[config.output] was not present. is required.")
        .as_mapping().expect("[config.output] must be a map. wasn't.");

    AppCfg {
        output_path: output
            .get("path").expect("[config.output.path] was not present. is required.")
            .as_str().expect("[config.output.path] must be a string. wasn't.").into(),
        output_n: output
            .get("num").expect("[config.output.num] was not present. is required.")
            .as_u64().expect("[config.output.num] must be a positive whole number. wasn't.") as usize,
    }
}

pub enum SourceKind {
    URL(String),
    Path(String),
}

impl SourceKind {
    pub fn perform(&self) -> Vec<u8> {
        match self {
            Self::URL(url) => load_as_bytes_from_url(url)
                .expect(format!("Failed to load file from [{}]", url).as_str()),
            Self::Path(path) => load_as_bytes_from_path(path)
                .expect(format!("Failed to load file from [{}]", path).as_str()),
        }
    }
}

pub fn parse_source(root_value: &serde_yaml::Value) -> SourceKind {
    let source = root_value
        .get("source").expect("[source] was not present. is required.")
        .as_mapping().expect("[source] must be a map. wasn't.");

    let url = source.get("url")
        .map(|val| val.as_str().expect("[source.url] must be a string. wasn't."));
    let file = source.get("file")
        .map(|val| val.as_str().expect("[source.file] must be a string. wasn't."));

    match (url, file) {
        (Some(url), None) => SourceKind::URL(url.into()),
        (None, Some(path)) => SourceKind::Path(path.into()),
        (Some(_), Some(_)) => panic!("found both [source.url] and [source.file]. exactly one must be specified."),
        (None, None) => panic!("didn't find [source.url] or [source.file]. at least one is needed."),
    }
}

pub enum ModeKind {
    AsImage,
    AsBytes,
    AsMemoryMap,
}

pub fn parse_mode(root_value: &serde_yaml::Value) -> ModeKind {
    let mode = root_value
        .get("mode").expect("[mode] was not present. is required.")
        .as_str().expect("[mode] must be a string. wasn't.");

    match mode {
        "image" => ModeKind::AsImage,
        "bytes" => ModeKind::AsBytes,
        "memory_map" => ModeKind::AsMemoryMap,
        _ => panic!("[mode] must be one of the following:\n\t- image\n\t- bytes\n\t- memory_map"),
    }
}

pub fn get_default<'a>(root_value: &'a serde_yaml::Value, key: &str) -> &'a serde_yaml::Value {
    let default = root_value
        .get("defaults").expect(format!("tried to extract [{}] from defaults, but [defaults] couldn't be found.", key).as_str());

    if key.contains("chunksize") {
        default.get(key).or(default.get("chunksize"))
            .expect(format!("tried to extract [chunksize] or [{}] from defaults due to missing setting, but it couldn't be found.", key).as_str())
    } else {
        default.get(key).expect(format!("tried to extract [{}] from defaults due to missing setting, but it couldn't be found.", key).as_str())
    }
}

pub fn get_optional_default<'a>(root_value: &'a serde_yaml::Value, key: &str) -> Option<&'a serde_yaml::Value> {
    let default = root_value
        .get("defaults").expect(format!("tried to extract [{}] from defaults, but [defaults] couldn't be found.", key).as_str());

    if key.contains("chunksize") {
        default.get(key).or(default.get("chunksize"))
    } else {
        default.get(key)
    }
}

pub fn parse_mutations<'a, 'b>(rng: &mut impl Rng, root_value: &serde_yaml::Value) -> Vec<Box<dyn Mutation>> where
    Chaos: Mutation,
    Expand: Mutation,
    Accelerate: Mutation,
    Increment: Mutation,
    Loop: Mutation,
    Multiply: Mutation,
    Reverse: Mutation,
    Shuffle: Mutation,
{
    let mutations = root_value
        .get("mutations").expect("[mutations] was not present - is required.")
        .as_sequence().expect("[mutations] must be a list - wasn't.");

    mutations.iter().enumerate().map(
        |(i, mutation)| {
            mutation.as_mapping().expect(format!("[mutations.{i}] must be a map - wasn't.").as_str())
        }
    ).map(
        |mutation| {
            let keys = mutation.keys().len();
            if keys != 1 {
                panic!("only one key [the mutation name] is accepted by mutation - found {}.", keys);
            }
            parse_mutation(rng, mutation, root_value)
        }
    ).collect::<Vec<_>>()
}

pub fn parse_mutation<'a, 'b>(rng: &mut impl Rng, mutation: &Mapping, root_value: &serde_yaml::Value) -> Box<dyn Mutation> where
    Chaos: Mutation,
    Expand: Mutation,
    Accelerate: Mutation,
    Increment: Mutation,
    Loop: Mutation,
    Multiply: Mutation,
    Reverse: Mutation,
    Shuffle: Mutation,
    Shift: Mutation,
    Swap: Mutation,
    Voidout: Mutation,
{
    // getting the only key - which should be the mutation name.
    let kind: MutationKind = mutation.keys().next().unwrap().as_str()
        .expect("an effect must start with its name as a string.").into();

    match kind {
        MutationKind::Chaos => Box::new(parse_chaos(rng, mutation, root_value)),
        MutationKind::Expand => Box::new(parse_expand(rng, mutation, root_value)),
        MutationKind::Compress => Box::new(parse_compress(rng, mutation, root_value)),
        MutationKind::Accelerate => Box::new(parse_accelerate(rng, mutation, root_value)),
        MutationKind::Increment => Box::new(parse_increment(rng, mutation, root_value)),
        MutationKind::Loop => Box::new(parse_loop(rng, mutation, root_value)),
        MutationKind::Multiply => Box::new(parse_multiply(rng, mutation, root_value)),
        MutationKind::Reverse => Box::new(parse_reverse(rng, mutation, root_value)),
        MutationKind::Shuffle => Box::new(parse_shuffle(rng, mutation, root_value)),
        MutationKind::Shift => Box::new(parse_shift(rng, mutation, root_value)),
        MutationKind::Swap => Box::new(parse_swap(rng, mutation, root_value)),
        MutationKind::Voidout => Box::new(parse_voidout(rng, mutation, root_value)),
    }
}

fn parse_chaos(rng: &mut impl Rng, mutation: &Mapping, root_value: &serde_yaml::Value) -> Chaos {
    let mutation = mutation.get("chaos").unwrap()
        .as_mapping().expect("[compress] was found but it wasn't a mapping.");

    let chunksize = 
        extract_u64_param(
            rng, mutation, root_value,
            "chaos", "chunksize");

    Chaos { chunksize: chunksize as usize }
}

fn parse_expand(rng: &mut impl Rng, mutation: &Mapping, root_value: &serde_yaml::Value) -> Expand {
    let mutation = mutation.get("expand").unwrap()
        .as_mapping().expect("[expand] was found but it wasn't a mapping.");

    let by = 
        extract_u64_param(
            rng, mutation, root_value,
            "expand", "by");

    let chunksize = 
        extract_u64_param(
            rng, mutation, root_value,
            "expand", "chunksize");

    Expand {
        by: by as f64,
        chunksize: chunksize as usize,
    }
}

fn parse_compress(rng: &mut impl Rng, mutation: &Mapping, root_value: &serde_yaml::Value) -> Compress {
    let mutation = mutation.get("compress").unwrap()
        .as_mapping().expect("[compress] was found but it wasn't a mapping.");

    let by = 
        extract_u64_param(
            rng, mutation, root_value,
            "compress", "by");

    let chunksize = 
        extract_u64_param(
            rng, mutation, root_value,
            "compress", "chunksize");

    Compress {
        by: by as f64,
        chunksize: chunksize as usize,
    }
}

fn parse_accelerate(rng: &mut impl Rng, mutation: &Mapping, root_value: &serde_yaml::Value) -> Accelerate {
    let mutation = mutation.get("accelerate").unwrap()
        .as_mapping().expect("[accelerate] was found but it wasn't a mapping.");

    let from = 
        extract_u64_param(
            rng, mutation, root_value,
            "accelerate", "from");

    let by = 
        extract_u64_param(
            rng, mutation, root_value,
            "accelerate", "by");

    let after = 
        extract_u64_param(
            rng, mutation, root_value,
            "accelerate", "after");

    let chunksize = 
        extract_u64_param(
            rng, mutation, root_value,
            "reverse", "chunksize");

    Accelerate {
        from: from as usize,
        by: by as usize,
        after: after as usize,
        chunksize: chunksize as usize,
    }
}

fn parse_increment(rng: &mut impl Rng, mutation: &Mapping, root_value: &serde_yaml::Value) -> Increment {
    let mutation = mutation.get("increment").unwrap()
        .as_mapping().expect("[increment] was found but it wasn't a mapping.");

    let by = 
        extract_u64_param(
            rng, mutation, root_value,
            "increment", "by");

    let chunksize = 
        extract_u64_param(
            rng, mutation, root_value,
            "increment", "chunksize");

    Increment { 
        by: by as usize,
        chunksize: chunksize as usize,
    }
}

fn parse_loop(rng: &mut impl Rng, mutation: &Mapping, root_value: &serde_yaml::Value) -> Loop {
    let mutation = mutation.get("loop").unwrap()
        .as_mapping().expect("[loop] was found but it wasn't a mapping.");

    let chunksize = 
        extract_u64_param(
            rng, mutation, root_value,
            "loop", "chunksize");

    let loopsize = 
        extract_u64_param(
            rng, mutation, root_value,
            "loop", "loopsize");

    Loop { chunksize: chunksize as usize, loopsize: loopsize as usize }
}

fn parse_multiply(rng: &mut impl Rng, mutation: &Mapping, root_value: &serde_yaml::Value) -> Multiply {
    let mutation = mutation.get("multiply").unwrap()
        .as_mapping().expect("[multiply] was found but it wasn't a mapping.");

    let by = 
        extract_f64_param(
            rng, mutation, root_value,
            "multiply", "by");

    let chunksize = 
        extract_u64_param(
            rng, mutation, root_value,
            "multiply", "chunksize");

    Multiply { by, chunksize: chunksize as usize }
}

fn parse_reverse(rng: &mut impl Rng, mutation: &Mapping, root_value: &serde_yaml::Value) -> Reverse {
    let mutation = mutation.get("reverse").unwrap()
        .as_mapping().expect("[reverse] was found but it wasn't a mapping.");

    let chunksize = 
        extract_u64_param(
            rng, mutation, root_value,
            "reverse", "chunksize");

    Reverse { chunksize: chunksize as usize }
}

fn parse_shuffle(rng: &mut impl Rng, mutation: &Mapping, root_value: &serde_yaml::Value) -> Shuffle {
    let mutation = mutation.get("shuffle").unwrap()
        .as_mapping().expect("[shuffle] was found but it wasn't a mapping.");

    let chunksize = 
        extract_u64_param(
            rng, mutation, root_value,
            "shuffle", "chunksize");

    Shuffle { chunksize: chunksize as usize }
}

fn parse_shift(rng: &mut impl Rng, mutation: &Mapping, root_value: &serde_yaml::Value) -> Shift {
    let mutation = mutation.get("shift").unwrap()
        .as_mapping().expect("[shift] was found but it wasn't a mapping.");

    // let from = 
    //     extract_u64_param(
    //         rng, mutation, root_value,
    //         "shift", "from");

    let by = 
        extract_u64_param(
            rng, mutation, root_value,
            "shift", "by");

    let chunksize = 
        extract_u64_param(
            rng, mutation, root_value,
            "shift", "chunksize");

    Shift {
        // from: from as usize,
        by: by as usize,
        chunksize: chunksize as usize,
    }
}

fn parse_swap(rng: &mut impl Rng, mutation: &Mapping, root_value: &serde_yaml::Value) -> Swap {
    let mutation = mutation.get("swap").unwrap()
        .as_mapping().expect("[swap] was found but it wasn't a mapping.");

    let chunk_1 = 
        extract_u64_param(
            rng, mutation, root_value,
            "swap", "chunk_1");

    let chunk_2 = 
        extract_u64_param(
            rng, mutation, root_value,
            "swap", "chunk_2");

    let chunksize = 
        extract_u64_param(
            rng, mutation, root_value,
            "swap", "chunksize");

    Swap {
        chunksize: chunksize as usize,
    }
}

fn parse_voidout(rng: &mut impl Rng, mutation: &Mapping, root_value: &serde_yaml::Value) -> Voidout {
    let mutation = mutation.get("voidout").unwrap()
        .as_mapping().expect("[voidout] was found but it wasn't a mapping.");

    let chunksize = 
        extract_u64_param(
            rng, mutation, root_value,
            "voidout", "chunksize");

    Voidout { chunksize: chunksize as usize }
}

fn check_param_exists(root_value: &serde_yaml::Value, mutation: &serde_yaml::Mapping, mutation_name: &str, property_name: &str) -> bool {
    let default_key = format!("{}.{}", mutation_name, property_name);
    match mutation.get(property_name) {
        Some(_) => true,
        None => get_optional_default(root_value, &default_key).is_some()
    }
}

macro_rules! param_extractor {
    ($parse_type:ty, $fn_name:ident, $parser:ident) => {
        fn $fn_name(
            rng: &mut impl Rng,
            mutation: &Mapping,
            root_value: &serde_yaml::Value,
            mutation_name: &str,
            property_name: &str,
        ) -> $parse_type
        {
            let default_key = format!("{}.{}", mutation_name, property_name);
        
            match mutation.get(property_name) {
                Some(val) => $parser(rng, val, &default_key),
                None => {
                    let default = get_default(root_value, &default_key);
                    $parser(rng, default, format!("defaults.{}", default_key).as_str())
                }
            }
        }
    };
}

macro_rules! param_parser {
    ($parse_type:ty, $fn_name:ident, $type_description:expr, $($cast_method:tt)*) => {
        fn $fn_name(
            rng: &mut impl Rng,
            param: &serde_yaml::Value,
            property_path: &str,
        ) -> $parse_type
        {
            if let Some(exact) = param.$($cast_method)* {
                exact
            } else if let Some(range) = param.as_mapping() {
                let min = range.get("min")
                    .expect(format!("expected [{}.min] due to mapping - not present.", property_path).as_str());
                let max = range.get("max")
                    .expect(format!("expected [{}.max] due to mapping - not present.", property_path).as_str());
        
                let min = min.$($cast_method)*.expect(format!("[{}.min] must be {} - wasn't.", property_path, $type_description).as_str());
                let max = max.$($cast_method)*.expect(format!("[{}.max] must be {} - wasn't.", property_path, $type_description).as_str());
        
                rng.gen_range(min, max)
            } else if let Some(options) = param.as_sequence() {
                let picked = options.choose(rng).unwrap();
                
                picked.$($cast_method)*.expect(format!("[{}] must have elements who are {}. weren't.", property_path, $type_description).as_str())
            } else {
                todo!()
            }
        }
    };
}

param_parser!(u64, parse_u64_param, "a positive integer", as_u64());
param_extractor!(u64, extract_u64_param, parse_u64_param);

param_parser!(f64, parse_f64_param, "a number", as_f64());
param_extractor!(f64, extract_f64_param, parse_f64_param);