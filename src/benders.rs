use super::{loaders::Loader};

use memmap2::MmapMut;

use super::mutations::*;

use super::configuration::Configuration;

use std::collections::HashMap;

use rayon::prelude::*;
use lazy_static::lazy_static;

type Mut = fn(&mut [u8], &Configuration) -> Result<String, MutationError>;
type Muts = Vec<Mut>;

// Initialise all mutations in here.
lazy_static! {
    pub static ref MUTMAP: HashMap<String, Mut> = {
        let mut map = HashMap::new();
        let mutmap: Vec<(String, Mut)> = 
        vec![
            ("Void".into()     , void),
            ("Chaos".into()    , chaos),
            ("Loops".into()    , loops),
            ("Reverse".into()  , reverse),
            ("Shift".into()    , shift),
            ("Shuffle".into()  , shuffle),
            ("Swap".into()     , swap),
            ("Increase".into() , increase),
            ("Gradient".into() , gradient),
            ("Multiply".into() , multiply),
            ("Compress".into() , compress),
            // Add more mutations here.
        ];

        for (k,v) in mutmap.into_iter() {
            map.insert(k, v);
        }

        map
    };
}

/// A main controller of the databender.
/// 
/// Manages the file handling, data storage, and controls mutations.
pub struct KaBender<'a> {
    pub seed: String,
    outdir: String,
    extension: String,
    output: String,
    pub config: &'a Configuration,
}

impl<'a> KaBender<'a> {
    /// Creates a new KaBender from the configuration.
    pub fn new(configuration: &'a Configuration, seed: String) -> Self {
        println!("Initialising bender...");
        let mut new : KaBender = KaBender {
            seed: seed,
            config : configuration,
            extension : String::new(),
            output : String::new(),
            outdir : String::new(),
        };

        new.setup_file_data();
        new
    }

    /// Executes the bender.
    /// 
    /// Performs all mutation combinations using the configuration loaded.
    pub fn run(mut self) {
        // Retrieves mutations.
        let mutations = self.config.get_mutations();

        // Generates a file for each list of mutations
        let filelist : Vec<MmapMut> = self.init_file_n(mutations.len());

        // Retrieves all mutations from hashmap using the file.
        let mutations : Vec<Muts> = mutations.par_iter().map(|combo| {
            combo.iter().map(|mut_str| {
                MUTMAP.get(*mut_str).cloned().unwrap()
            }).collect()
        }).collect();

        // Pairs each memory-mapped file to a list of mutations.
        let mut mut_map : Vec<(Muts, MmapMut)> = mutations
            .into_iter()
            .zip(filelist
                .into_iter())
            .collect();

        // Performs the mutations in parallel
        mut_map
            .par_iter_mut()
            .enumerate()
            .for_each(|(index, (mutation_combo, map))| {
                let mut log = Vec::new();

                let results: Result<Vec<_>, _> = mutation_combo.into_iter().map(|mutation| {
                    match mutation(map, self.config) {
                        Ok(mutation) => Ok(log.push(mutation)),
                        Err(error) => {
                            eprintln!("{}", error.error);
                            Loader::remove_file(&format!("{}temp{}SEED={}.{}", self.outdir, index, self.seed, self.extension))
                        },
                    }
                }).collect();

                if results.is_ok() {
                    self.flush(index, log);
                }
            });
    }

    /// Initialises multiple memory mapped copies of a file.
    /// 
    /// * `n` - Number of files to initialize
    fn init_file_n(&mut self, n: usize) -> Vec<MmapMut> {
        println!("Initialising file...");

        (0..n)
            .into_par_iter()
            .map(|index| {
                Loader::init_file_mut(
                    self.config.get_inputfile(),
                    format!("{}temp{}SEED={}.{}",
                        self.outdir,
                        index,
                        self.seed,
                        self.extension
                    ).as_str()
                ).unwrap()
            }).collect()
    }

    /// Sets up the data of the file, such as the input, output, extension, and path.
    fn setup_file_data(&mut self) {
        use std::path::Path;
        use std::ffi::OsStr;

        let input = self.config.get_inputfile();

        // Sets output name to custom name, or input if not specified.
        let output: &str = self.config.get("outputfile")
            .and_then(|v| v.as_str())
            .map_or(input.clone(), |s| s.as_str());

        let path = Path::new(&output);

        // Extracts the extension from the filename
        self.extension = String::from(path
            .extension()
            .and_then(OsStr::to_str)
            .unwrap()
            .clone());

        // Extracts the output directory.
        // In X/Y.../Z.EXT, this extracts X/Y.../
        self.outdir = path.parent().and_then(Path::to_str).map_or(String::new(), |text| {
            if text == "" {
                String::new()
            } else {
                format!("{}/", text)
            }
        });

        // Represents the full path, apart from the extension.
        // In X/Y/.../Z.EXT, this extracts X/Y.../Z
        self.output = format!(
            "{}{}",
            self.outdir,
            path.file_stem().and_then(OsStr::to_str).unwrap().clone(),
        );
    }

    /// Renames the temporary file that was mutated to its supposed output file.
    /// 
    /// * `iter` - The iteration. Used to rename the right mutated file.
    /// * `log` - The log of mutations applied to the file. Used to embed mutation data into the filename itself.
    fn flush(&self, iter: usize, log: Vec<String>){
        let mut temp_muts = log.join("---");
        if temp_muts.len() > 200 {
            temp_muts.truncate(200);
            println!("Truncating mutation name due to length...");
        }

        // Generates an output name
        let genoutput = format!("{name}__{muts}.{ext}",
            name = self.output.clone(),
            muts = temp_muts,
            ext = self.extension.clone(),
        );

        let temporaryname = format!("{}temp{}SEED={}.{}", self.outdir, iter, self.seed, self.extension);

        // Renames temporary file to actual output name
        let result = Loader::rename_file(&temporaryname, &genoutput);

        if let Err(err) = result {
            println!("\n{:-^80}\nSomething went wrong while renaming the file from \n{} to {}\n{}\n{:-^80}", "ERROR",
             temporaryname, genoutput, err.to_string(), "")
        }
    }
}
