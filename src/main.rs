use std::collections::HashMap;
use std::env;
use std::fs::File;
use rand::{Rng, prelude::ThreadRng, seq::SliceRandom};
use serde::Deserialize;

static mut DEBUG: bool = false;

/// Prints a debug message if the program is in debug mode
/// 
/// # Arguments
/// 
/// * `message` - The message to print
/// 
/// # Example
/// 
/// ```
/// // Will only print with debug mode enabled
/// debug!("Hello world!");
/// ```
macro_rules! debug {
    ($($arg:tt)*) => {
        if unsafe { DEBUG } {
            eprint!("[DEBUG]   ");
            eprintln!($($arg)*);
        }
    }
}

/// Handles the processing of launch arguments passed in the command line
/// 
/// # Arguments
/// 
/// * `args` - The arguments passed in the command line
/// 
/// * `word_count` - The number of words to generate
/// 
/// * `path` - The path to the config file
/// 
/// # Returns
/// 
/// `bool` - Whether the program should continue running
fn handle_launch_args(args: Vec<String>, word_count: &mut i32, path: &mut String, affixes: &mut Vec<String>) -> bool {
    if args.len() > 1 {
        // help message
        if args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
            println!("Usage: ./namesmith [-n <word_count>] [-d] [-p <path>]");
            println!("\t-n\tnumber of words to generate");
            println!("\t-d\tenable debug mode");
            println!("\t-p\tpath to config file");
            println!("\t-a\ta list of affixed syllables in IPA");
            println!("\t-v\tdisplay the current version");
            println!("\t-h\tdisplay this help message");
            return false;
        }

        // number of words to generate
        if args.contains(&"-n".to_string()) || args.contains(&"--number".to_string()) {
            let index = args.iter().position(|x| x == "-n").unwrap();
            *word_count = args[index + 1].parse::<i32>().unwrap();
        }

        // enable debug mode
        if args.contains(&"-d".to_string()) || args.contains(&"--debug".to_string()) {
            unsafe {
                DEBUG = true;
            }
        }

        // path to config file
        if args.contains(&"-p".to_string()) || args.contains(&"--path".to_string()) {
            let index = args.iter().position(|x| x == "-p").unwrap();
            *path = args[index + 1].clone();
        }

        // affixes
        if args.contains(&"-a".to_string()) || args.contains(&"--affixes".to_string()) {
            let index = args.iter().position(|x| x == "-a").unwrap();
            *affixes = args[index + 1].clone().replace("\"", "").replace("'", "").split(",").map(|x| x.to_string()).collect();
            debug!("Affixes: {:?}", affixes);
        }

        // version
        if args.contains(&"-v".to_string()) || args.contains(&"--version".to_string()) {
            println!("namesmith v{}", env!("CARGO_PKG_VERSION"));
            return false;
        }
    }

    return true;
}

/// Handles the processing of the config file
/// 
/// # Arguments
/// 
/// * `path` - The path to the config file
/// 
/// # Returns
/// 
/// The configuration loaded from the config file and two mutable `vec`s of `String`s
/// 
/// # Example
/// 
/// ```
/// // Will load the config file and return the config and two mutable `vec`s of `String`s
/// let (cfg, onsets, codas) = handle_config_file("config.json");
/// ```
fn handle_config(mut path: String) -> (Config, Vec<String>, Vec<String>) {
    if path == String::new() {
        path = "./english.json".to_owned();
    }
    let config: Config = serde_json::from_reader(File::open(&path).unwrap()).unwrap();
    let mut codas = config.codas.clone();
    let mut onsets = config.onsets.clone();
    if onsets.len() == 1 && onsets[0] == "@" {
        onsets = config.consonants.clone();
    }
    if codas.len() == 1 && codas[0] == "@" {
        codas = config.consonants.clone();
    }
    (config, codas, onsets)
}

/// Generates a random syllable
/// 
/// # Arguments
/// 
/// * `syllable` - The syllable to generate
/// 
/// * `config` - The configuration loaded from the config file
/// 
/// * `rng` - The random number generator
/// 
/// * `word` - The word to generate the syllable for
/// 
/// * `vowel_index` - Where the vowel is located in the syllable
/// 
/// * `onsets` - A Vec of possible onsets to use
/// 
/// * `codas` - A Vec of possible codas to use
/// 
/// # Returns
/// 
/// `String` - The generated syllable
/// 
/// # Example
/// 
/// ```
/// // Will generate a random syllable
/// generate_syllable(syllable_out, &config, &mut rng, &word, 0, &onsets, &codas);
/// ```
fn build_syllable(structure: &String, config: &Config, rng: &mut ThreadRng, word: &mut Vec<String>, onsets: &Vec<String>, codas: &Vec<String>) {
    let mut syllable: Vec<String> = vec![];
    // find the location of the vowel in the syllable
    let vowel_index = structure.to_lowercase().find("v").unwrap();
    // for each character in the syllable structure
    for index in 0..structure.len() {
        // if the letter is a vowel
        if structure.chars().nth(index).unwrap() == 'v' {
            // choose a random vowel
            let vowel = config.vowels.choose(rng).unwrap().to_owned();
            debug!("vowel:\t{}", vowel);

            syllable.push(vowel.to_string());
        }
        else {
            debug!("index:\t{}", index);

            // if before v:
            if index < vowel_index {
                // choose a random onset
                let onset = onsets.choose(rng).unwrap();
                debug!("onset:\t{}", onset);

                // insert the chosen onset before the vowel
                if syllable.len() == 0 {
                    syllable.push(onset.to_string());
                }
                else {
                    syllable.insert(0, onset.to_string());
                }
            }
            else {
                // choose a random coda
                let coda = codas.choose(rng).unwrap();
                debug!("coda:\t{}", coda);
                syllable.push(coda.to_string());
            }
        }

        word.append(&mut syllable);
    }
}

/// Generates a word from multiple or one syllables
/// 
/// # Arguments
/// 
/// * `config` - The configuration loaded from the config file
/// 
/// * `onsets` - A Vec of possible onsets to use
/// 
/// * `codas` - A Vec of possible codas to use
/// 
/// # Returns
/// 
/// `Vec<String>` - The generated word as a Vec of Strings to account for dipthongs
fn create_word(config: &Config, onsets: &Vec<String>, codas: &Vec<String>, affixes: &Vec<String>) -> Vec<String> {
    let mut word: Vec<String> = vec![];
    let mut rng = rand::thread_rng();
    let syllable_count = rng.gen_range(1..config.max_syllable_count + 1);
    debug!("syllable_count:\t{}", syllable_count);
    // build the syllables
    for i in 0..syllable_count {
        // indicate syllable is stressed
        if i == config.stressed || i == syllable_count - (config.stressed * -1) || syllable_count == 1 {
            word.push("'".to_owned());
        }
        // choose a syllable
        let syllable = config.structures.choose(&mut rng).unwrap();
        debug!("syllable:\t{}", syllable);

        // for each letter in the syllable
        build_syllable(syllable, config, &mut rng, &mut word, onsets, codas);

        // unless it's the last syllable, add a syllable marker
        if i != syllable_count - 1 {
            word.push("•".to_owned());
        }
    }
    if affixes.len() > 0 {
        // should there be a prefix?
        let prefixed = rng.gen_bool(0.5);
        // should there be a suffix?
        let suffixed = rng.gen_bool(0.5);

        if prefixed {
            // choose a random affix or none
            let affix = affixes.choose(&mut rng).unwrap();
            if affix.starts_with("+") && prefixed {
                word.insert(0, affix.to_owned().replace("+", ""));
                // add a syllable marker
                word.insert(1, "•".to_owned());
            }
        } else if suffixed {
            // choose a random suffix or none
            let affix = affixes.choose(&mut rng).unwrap();
            // make sure there's a syllable marker
            if word[word.len() - 1] != "•" {
                word.push("•".to_owned());
            }

            // add the affix
            if affix.starts_with("-") {
                word.push(affix.to_owned().replace("-", ""));
            }
        }
    }
    
    word
}

/// Builds the output string from the raw word and romanized version
/// 
/// # Arguments
/// 
/// * `word` - The raw word to build the output string from
/// 
/// * `config` - The configuration loaded from the config file
/// 
/// # Returns
/// 
/// (`String`, `String) - The final ipa string and romanized string
fn create_final_str(word: Vec<String>, config: &Config) -> (String, String) {
    let ipa_word = word.join("");
    let mut clone = ipa_word.clone();
    // sort the hashmap by length of the key
    let mut sorted_map: Vec<(&String, &String)> = config.romanization.iter().collect();
    sorted_map.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
    for (key, value) in sorted_map {
        // replace the key with the value
        clone = clone.replace(key, value);
    }
    let romanized_word = clone.replace("'", "").replace("•", "");
    (ipa_word, romanized_word)
}

#[derive(Deserialize, Debug)]
/// The configuration file as represented by a struct
struct Config {
    /// Any possible consonant sound
    pub consonants: Vec<String>,
    /// Any consonant possibly used as an onset
    pub onsets: Vec<String>,
    /// Any consonant possibly used as a coda
    pub codas: Vec<String>,
    /// Any possible vowel sound
    pub vowels: Vec<String>,
    /// Which syllable most commonly takes the stress
    pub stressed: i32,
    /// A map of ipa sounds to romanized characters as according to the config file
    pub romanization: HashMap<String, String>,
    /// Any possible syllable structure (e.g. "cv", "cvc", etc.)
    pub structures: Vec<String>,
    /// The maximum number of syllables in a word
    pub max_syllable_count: i32,
}

fn main() {
    // number of words to generate
    let mut word_count = 5;
    debug!("word_count:\t{}", word_count);
    // very basic launch argument handling
    let args: Vec<String> = env::args().collect();
    let mut path = String::new();
    let mut affixes: Vec<String> = vec![];
    if !handle_launch_args(args, &mut word_count, &mut path, &mut affixes) {
        return;
    }

    let (config, codas, onsets) = handle_config(path);

    // for each word
    for _ in 0..word_count {
        let word = create_word(&config, &onsets, &codas, &affixes);
        // join the word
        let (ipa_word, romanized_word) = create_final_str(word, &config);
        // print romanized word
        println!("{} ({})", romanized_word, ipa_word);
    }
}
