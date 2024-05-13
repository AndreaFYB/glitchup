use rand::Rng;
use rand_xorshift::XorShiftRng;
use rand_core::{SeedableRng, RngCore};
use rand::seq::SliceRandom;

use rayon::iter::IntoParallelRefMutIterator;
use rayon::iter::ParallelIterator;

use moveslice::Moveslice;

use std::sync::{Arc, Mutex};

fn index_boundary(data: &[u8]) -> (usize, usize) {
    (data.len()/10, data.len())
}

fn generate_index(data: &[u8]) -> usize {
    let (min, max) = index_boundary(data);
    rand::thread_rng().gen_range(min, max)
}

macro_rules! get_opt_as {
    (int, $cfg:ident, $configname:tt, $value:tt, $type:ty) => {
        {
             $cfg.generate_int_from_option($configname, $value)
                .map(|option| option as $type)
                .ok_or(MutationError::new(
                    format!("Expected '{}' to be under '{} or globally as a valid integer, or range.", $value, $configname)
                ))
        }
    };
    (float, $cfg:ident, $configname:tt, $value:tt, $type:ty) => {
        {
            $cfg.generate_float_from_option($configname, $value)
               .map(|option| option as $type)
               .ok_or(MutationError::new(
                   format!("Expected '{}' to be under '{} or globally as a valid float, or range.", $configname, $value)
               ))
       }
    };
}

macro_rules! index_range {
    ($data:ident, $chunksize:ident) => {
        {
            let len = $data.len();
            let start = generate_index($data);
            let end = if $chunksize + start > len {len} else {$chunksize + start};
            start..end
        }
    };
}

macro_rules! slice_mut {
    ($data:ident, $chunksize:ident) => {
        $data.get_mut(index_range!($data, $chunksize))
    };
}

pub struct MutationError {
    pub error: String
}

impl MutationError {
    fn new(error: String) -> Self {
        MutationError { error }
    }
}

pub fn chaos(data: &mut [u8], cfg: &crate::Configuration) -> Result<String, MutationError> {
    let name = "ChaosConfig";

    let iterations = get_opt_as!(int, cfg, name, "iterations", usize)?;
    let chunksize = get_opt_as!(int, cfg, name, "chunksize", usize)?;

    let xrng = Arc::new(Mutex::new(XorShiftRng::from_rng(rand::thread_rng())
        .map_err(|err| MutationError::new(err.to_string()))?));


    for _ in 0..iterations {
        if let Some(slice) = slice_mut!(data, chunksize) {
            slice.par_iter_mut().for_each(|chr| *chr = xrng.lock().unwrap().next_u32() as u8);
        }
    }

    Ok(format!("CHS_it={}_ch={}", iterations, chunksize))
}

pub fn compress(data: &mut [u8], cfg: &crate::Configuration) -> Result<String, MutationError> {
    let name = "CompressConfig";

    let iterations = get_opt_as!(int, cfg, name, "iterations", usize)?;
    let chunksize = get_opt_as!(int, cfg, name, "chunksize", usize)?;
    let compress_by = get_opt_as!(int, cfg, name, "compress_by", usize)?;

    for _ in 0..iterations {
        if let Some(slice) = slice_mut!(data, chunksize) {
            let mut c_index = 0; // index of byte to use

                let mut sclone = vec![0; slice.len()];
                sclone[..].clone_from_slice(slice);
                let slen = sclone.len();

                for chr in slice.iter_mut() {
                    *chr = sclone[c_index];
                    c_index += compress_by;

                    if c_index >= slen {
                        break;
                    }
                }
        }
    }

    Ok(format!("CMP_it={}_ch={}", iterations, chunksize))
}

pub fn gradient(data: &mut [u8], cfg: &crate::Configuration) -> Result<String, MutationError> {
    let name = "GradientConfig";

    let iterations = get_opt_as!(int, cfg, name, "iterations", usize)?;
    let chunksize = get_opt_as!(int, cfg, name, "chunksize", usize)?;
    let accelerate_by = get_opt_as!(int, cfg, name, "accelerate_by", usize)?;
    let accelerate_in = get_opt_as!(int, cfg, name, "accelerate_in", usize)?;

    for _ in 0..iterations {
        let mut n = accelerate_by;
        let mut i = 0;

        if let Some(slice) = slice_mut!(data, chunksize) {
            for chr in slice.iter_mut() {
                *chr = ((*chr as usize + n) % 256) as u8;
                i += 1;

                if i >= accelerate_in {
                    i = 0;
                    n += accelerate_by;
                };
            }
        }
    }

    Ok(format!("GRT_it={}_ch={}_by={}_in={}",
        iterations, chunksize, accelerate_by, accelerate_in))
}

pub fn increase(data: &mut [u8], cfg: &crate::Configuration) -> Result<String, MutationError> {
    let name = "IncreaseConfig";

    let iterations = get_opt_as!(int, cfg, name, "iterations", usize)?;
    let chunksize = get_opt_as!(int, cfg, name, "chunksize", usize)?;
    let increase_by = get_opt_as!(int, cfg, name, "increase_by", usize)?;

    for _ in 0..iterations {
        if let Some(slice) = slice_mut!(data, chunksize) {
            for chr in slice.iter_mut() {
                *chr = ((*chr as usize + increase_by) % 256) as u8;
            }
        }
    }

    Ok(format!("INC_it={}_ch={}_by={}",
        iterations, chunksize, increase_by))
}

pub fn loops(data: &mut [u8], cfg: &crate::Configuration) -> Result<String, MutationError> {
    let name = "LoopsConfig";

    // Options
    let iterations = get_opt_as!(int, cfg, name, "iterations", usize)?;
    let chunksize = get_opt_as!(int, cfg, name, "chunksize", usize)?;
    let mut loops = get_opt_as!(int, cfg, name, "loops", usize)?;

    // Extra variables needed
    let (index_min, index_max) = index_boundary(data);
    let len = data.len();

    // Update loops
    // Below is formula explaining why this code is needed.
    // The assumption is that the lowest index should be smaller than 
    // the maximum index subtracted by (chunksize-loops).
    // MIN < MAX-(CH*LP)
    // CH*LP < MAX-MIN
    // LP < (MAX-MIN)/CH
    let min_safe_loops = (index_max-index_min)/chunksize;
    loops = loops.min(min_safe_loops);

    for _ in 0..iterations {
        let index = rand::thread_rng()
            .gen_range(index_min, index_max-(chunksize*loops));

            // Get whole file to allow circular access
            if let Some(slice) = data.get_mut(0..) {
                // Loop for (chunksize) times...
                for _ in 0..chunksize {
                    // Internally loop (loop) times...
                    for rep in 1..=loops {
                        // Get the index of the character to modify
                        let modind = 
                            if index + chunksize * rep < index_max{
                                index + chunksize * rep
                            }
                            else {
                                ((index + chunksize * rep) % (index_max-index_min)) + index_min
                            };

                        // Shows important info before panic - for catching bugs.
                        if index > len || modind > len {
                            eprintln!("Diagnostics before panic.");
                            eprintln!("index:{}, min/max:{}/{}, modind:{}, chsize:{}, in+ch+tp:{} % max {} + min {}",
                             index, index_min, index_max, modind, chunksize, index + chunksize * rep,
                             (index + chunksize * rep) % index_max, ((index + chunksize * rep) % index_max) + index_min);
                            panic!("Out of bounds error. If you see this, please contact the developer.");
                        }
                            
                        // "Repeat" current byte across other byte.
                        slice[modind] = slice[index];
                    }
                }
            }
    }

    Ok(format!("LPS_it={}_ch={}_lps={}",
        iterations, chunksize, loops))
}

pub fn multiply(data: &mut [u8], cfg: &crate::Configuration) -> Result<String, MutationError> {
    let name = "MultiplyConfig";

    let iterations = get_opt_as!(int, cfg, name, "iterations", usize)?;
    let chunksize = get_opt_as!(int, cfg, name, "chunksize", usize)?;
    let multiply_by = get_opt_as!(float, cfg, name, "multiply_by", f64)?;

    for _ in 0..iterations {
        if let Some(slice) = slice_mut!(data, chunksize) {
            for chr in slice.iter_mut() {
                *chr = ((*chr as f64 * multiply_by) as usize % 256) as u8;
            }
        }
    }

    Ok(format!("MUL_it={}_ch={}_by={}",
        iterations, chunksize, multiply_by))
}

pub fn reverse(data: &mut [u8], cfg: &crate::Configuration) -> Result<String, MutationError> {
    let name = "ReverseConfig";

    let iterations = get_opt_as!(int, cfg, name, "iterations", usize)?;
    let chunksize = get_opt_as!(int, cfg, name, "chunksize", usize)?;

    for _ in 0..iterations {
        if let Some(slice) = slice_mut!(data, chunksize) {
            slice.reverse();
        }
    }

    Ok(format!("RVR_it={}_ch={}",
        iterations, chunksize))
}

pub fn shift(data: &mut [u8], cfg: &crate::Configuration) -> Result<String, MutationError> {
    let name = "ShiftConfig";

    let (index_min, index_max) = index_boundary(data);
    let new_max = index_max - index_min;

    let iterations = get_opt_as!(int, cfg, name, "iterations", usize)?;
    let chunksize = get_opt_as!(int, cfg, name, "chunksize", usize)?;

    for _ in 0..iterations {
        let index = rand::thread_rng().gen_range(0, new_max);
        let m_index = rand::thread_rng().gen_range(0, new_max - chunksize);

        if let Some(mut slice) = data.get_mut(index_min..index_max) {
            let max_i = if chunksize+index > slice.len() {slice.len()} else {chunksize+index};
                let result = slice.try_moveslice(
                    index..max_i,
                    m_index
                );

                if let Err(res) = result {
                    eprintln!("Shifting failed. Moveslice returned error: {:?}", res);
                };
        }
    }

    Ok(format!("SFT_it={}_ch={}",
        iterations, chunksize))
}

pub fn shuffle(data: &mut [u8], cfg: &crate::Configuration) -> Result<String, MutationError> {
    let name = "ShuffleConfig";

    let iterations = get_opt_as!(int, cfg, name, "iterations", usize)?;
    let chunksize = get_opt_as!(int, cfg, name, "chunksize", usize)?;

    for _ in 0..iterations {
        if let Some(slice) = slice_mut!(data, chunksize) {
            slice.shuffle(&mut rand::thread_rng());
        }
    }

    Ok(format!("SHF_it={}_ch={}",
        iterations, chunksize))
}

pub fn swap(data: &mut [u8], cfg: &crate::Configuration) -> Result<String, MutationError> {
    let name = "SwapConfig";

    // Options
    let iterations = get_opt_as!(int, cfg, name, "iterations", usize)?;
    let chunksize = get_opt_as!(int, cfg, name, "chunksize", usize)?;

    // Validation check
    if chunksize >= (0.49 * data.len() as f64) as usize {
        println!();
        return Err(MutationError::new("Cannot perform Swap - Chunksize is too large.".into()));
    }

    // Extra variables needed
    let mut rng = rand::thread_rng();
    let (index_min, index_max) = index_boundary(data);
    let sl = data.get_mut(index_min..index_max).unwrap();
    let len = sl.len();

    // Actual mutation
    for _ in 0..iterations {
        let splitdex = rng.gen_range(chunksize, (index_max-index_min)-chunksize);

        let (left, right) = sl.split_at_mut(splitdex);

        let index1 = rng.gen_range(0, splitdex - chunksize);
        let index2 = rng.gen_range(0, len - splitdex - chunksize);
        let (llen, rlen) = {(left.len(), right.len())};

        let slice1 = left.get_mut(index1..index1+chunksize);
        let slice2 = right.get_mut(index2..index2+chunksize);

        if slice1.is_none() || slice2.is_none() {
            eprintln!("Diagnostics before panic.");
            eprintln!("i1r:{}, i2r:{}", splitdex - chunksize, len - splitdex - chunksize);
            eprintln!("flen:{}, dex:{}, ch:{}", len, splitdex, chunksize);
            eprintln!("len:{}, i1a/i1b:{}/{}", llen, index1, index1+chunksize);
            eprintln!("len:{}, i2a/i2b:{}/{}", rlen, index2, index2+chunksize);
            panic!("Out of bounds error. If you see this, please contact the developer.");
        }
        slice1.unwrap().swap_with_slice(slice2.unwrap());
    }

    Ok(format!("SWP_it={}_ch={}",
        iterations, chunksize))
}

pub fn void(data: &mut [u8], cfg: &crate::Configuration) -> Result<String, MutationError> {
    let name = "VoidConfig";

    let iterations = get_opt_as!(int, cfg, name, "iterations", usize)?;
    let chunksize = get_opt_as!(int, cfg, name, "chunksize", usize)?;

    for _ in 0..iterations {
        if let Some(slice) = slice_mut!(data, chunksize) {
            for chr in slice.iter_mut() {
                *chr = b'0';
            }
            slice.par_iter_mut().for_each(|chr| *chr = b'0');
        }
    }

    Ok(format!("VOID_it={}_ch={}",
        iterations, chunksize))
}