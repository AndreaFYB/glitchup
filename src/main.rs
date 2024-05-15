//! A main function. Currently doesn't have anything since work on 
//! a databender hasn't started yet.

mod benders;
mod mutations;
mod loaders;
mod configuration;
mod mutations2;
mod config;
mod preconvert;

use std::{clone, error::Error, io::{Cursor, Read}};

use benders::KaBender;
use config::{ioutils::{load_yaml_file, save_bytes_to_file}, parser::{parse_app_cfg, parse_mutations, parse_source, AppCfg}};
use configuration::Configuration;

use image::{codecs::tiff::{TiffDecoder, TiffEncoder}, DynamicImage, ImageEncoder};
use rand::{rngs::StdRng, Rng, SeedableRng};
use rayon::prelude::*;

use crate::{config::{ioutils::{load_as_image_from_bytes, load_as_mmap_from_path}, parser::{parse_features, Features}}, mutations2::{local::{Expand, Increment}, AreaType, Mutation}, preconvert::generic_preconvert};

fn main2() {
    // Initialises the configuration for the application.
    let conf = Configuration::from_file("Options.toml");

    conf.verify_config();

    // Initialises the mutation map at the start.
    lazy_static::initialize(&benders::MUTMAP);

    // Retrieves some options from the configuration.
    let loops = conf.get("times")
        .and_then(|times| times.as_int())
        .unwrap_or(&1);

    (0..*loops).into_par_iter().for_each(|i| {
        let bender = KaBender::new(&conf, i.to_string());
        bender.run();
    });
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args();
    let yaml = load_yaml_file(args.nth(1)
        .expect("an argument must be passed that links to the configuration file - missing."))?;

    let source = parse_source(&yaml);
    let app_cfg = parse_app_cfg(&yaml);
    let features = parse_features(&yaml);

    let mut rng = StdRng::from_entropy();

    let mut file = source.perform();
    
    // PRE-CONVERT
    let extension = if let Some(pre_convert) = &features.pre_convert {
        file = generic_preconvert(&file, pre_convert.get_image_format().expect("not a valid format"))?;
        &pre_convert.format
    } else {
        &app_cfg.output.extension
    };
    
    let bytesize = file.len();
    println!("file in memory has [{} bytes]", bytesize);

    for i in 0..app_cfg.output.num {
        let new_file_path = format!("{}-{}.{}", app_cfg.output.path, i, extension);

        let mut mutations = parse_mutations(&mut rng, &yaml);
        if features.memory_map {
            save_bytes_to_file(new_file_path.as_str(), &file)?;
            let mut memory_map = load_as_mmap_from_path(&new_file_path)?;
            mutate_bytes(&mut memory_map, &mut rng, &mut mutations)?;
        }

        if features.sequential {
            mutate_bytes(&mut file, &mut rng, &mut mutations)?;
            save_bytes_to_file(&new_file_path, &file)?;
        } else {
            let mut file = file.clone();
            mutate_bytes(&mut file, &mut rng, &mut mutations)?;
            save_bytes_to_file(&new_file_path, &file)?;
        }
    }

    Ok(())
}

fn mutate_bytes(
    bytes: &mut [u8],
    rng: &mut impl Rng,
    mutations: &mut Vec<Box<dyn Mutation>>,
) -> Result<(), Box<dyn Error>> {
    let bytesize = bytes.len();
    for mutation in mutations.iter() {
        // println!("applying mut: {} which acts in the area: {:?}", mutation.get_name(), mutation.get_type());
        match mutation.get_type() {
            AreaType::Global => {
                mutation.bend(bytes)
            },
            AreaType::Local => {
                let chunksize = mutation.get_chunksize();
                let chunkstart = rng.gen_range(0, bytesize - chunksize);

                mutation.bend(
                    &mut bytes[chunkstart..chunkstart+chunksize]);
            }
        }
    };

    Ok(())
}