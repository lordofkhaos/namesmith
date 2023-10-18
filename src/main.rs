use std::{collections::HashMap, path::Path};
use std::env;
use std::fs::File;
use rand::{Rng, prelude::ThreadRng, seq::SliceRandom};
use serde::Deserialize;

static mut DEBUG: bool = false;

// TODO: improve performance by pre-generating all possible syllables in a file, only updating them when the config file changes

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
            println!("Usage: ./namesmith [-n <num_of_words>] [-d] [-p <path>]");
            println!("\t-n\tnumber of words to generate");
            println!("\t-d\tenable debug mode");
            println!("\t-p\tpath to config file");
            println!("\t-a\ta list of affixed syllables in as phonemes (e.g. \"-ən,+pri\")");
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
    } else {
        println!("Usage: ./namesmith [-n <word_count>] [-d] [-p <path>]");
        return false;
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
    path = path.trim().to_string();
    debug!("path:\t{}", path);
    // first, test to see if the file exists
    // if the file does not exist, return empty objects to keep a happy system
    if !Path::new(&path).exists() {
        let _c = Config {
            consonants: vec![],
            onsets: vec![],
            codas: vec![],
            vowels: vec![],
            stressed: 0,
            romanization: HashMap::new(),
            structures: vec![],
            max_syllable_count: 0,
        };
        return (_c, vec![], vec![]);
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

fn wrap_sound(sound: String) -> String {
    format!("[{}]", sound)
}

/// Generates a random syllable
/// 
/// # Arguments
/// 
/// * `structure` - The structure of the syllable (e.g. "cvc")
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
/// build_syllable(syllable_out, &config, &mut rng, &word, 0, &onsets, &codas);
/// ```
fn build_syllable(structure: &String, config: &Config, rng: &mut ThreadRng, word: &mut Vec<String>, onsets: &Vec<String>, codas: &Vec<String>) {
    let mut syllable: Vec<String> = vec![];
    // find the location of the vowel in the syllable
    let vowel_index = structure.to_lowercase().find("v").unwrap();
    // for each character in the syllable structure
    for index in 0..structure.len() {
        debug!("index:\t{}\tsyllable:\t{:?}", index, syllable);
        // if the letter is a vowel
        if structure.chars().nth(index).unwrap() == 'v' {
            // choose a random vowel
            let vowel = config.vowels.choose(rng).unwrap().to_owned();
            debug!("vowel:\t{}", vowel);

            syllable.push(wrap_sound(vowel.to_string()));
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
                    syllable.push(wrap_sound(onset.to_string()));
                }
                else {
                    syllable.insert(0, wrap_sound(onset.to_string()));
                }
            }
            else {
                // choose a random coda
                let coda = codas.choose(rng).unwrap();
                debug!("coda:\t{}", coda);
                syllable.push(wrap_sound(coda.to_string()));
            }
        }

        debug!("syllable:\t{}", syllable.concat());
    }
    word.push(syllable.concat());
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
        let syllable_structure = config.structures.choose(&mut rng).unwrap();
        debug!("structure:\t{}", syllable_structure);

        // for each letter in the syllable
        build_syllable(&syllable_structure.to_lowercase(), config, &mut rng, &mut word, onsets, codas);

        // unless it's the last syllable, add a syllable marker
        if i != syllable_count - 1 {
            word.push(" ".to_owned());
        }
    }
    if affixes.len() > 0 {
        let mut prefixed = false;
        let mut suffixed = false;
        for affix in affixes {
            if affix.starts_with("+") {
                prefixed = true;
            } else if affix.starts_with("-") {
                suffixed = true;
            }

            if prefixed && suffixed {
                break; // no need to go any further
            }
        }

        if prefixed {
            let mut copy = affixes.to_vec();
            let mut real_affix = "".to_owned();
            // choose a random suffix
            let mut affix = "".to_owned();
            while !affix.starts_with("+") {
                affix = copy.choose(&mut rng).unwrap().to_owned();
                // remove the chosen affix from the list
                debug!("PREFIX: ----- affix: '{}'", affix);
                let index = copy.iter().position(|x| x == &affix).unwrap();
                debug!("index: {}, copy.length(): {}", index, copy.len());
                copy.remove(index);
                
            }
            affix = affix.replace("+", "");
            debug!("--- affix chars: \"{}\"", affix.split("").collect::<Vec<&str>>().join("   "));
            let mut temp: Vec<u32> = vec![];
            for c in affix.chars() {
                temp.push(c as u32);
            }
            debug!("--- affix chars: \"{}\"", temp.into_iter().map(|x| x.to_string()).collect::<Vec<String>>().join("   "));


            for (i, c) in affix.chars().enumerate() {
                // check if this is a diphthong:
                // proper error handling:
                let res = affix.chars().nth(i + 1);
                match res {
                    // within the bounds of the affix
                    Some(x) => {
                        // if the next character is a tie bar, the next three characters form a diphthong
                        if x as u32 == 865 { // check for a tie bar
                            debug!("---- tie bar found, adding [");
                            real_affix.push_str(&format!("[{}", c));
                            continue;
                        }

                        if c as u32 == 865 { // check for a tie bar
                            debug!("---- tie bar found, adding tie bar");
                            real_affix.push(c);
                            continue;
                        }

                        // else, it's not a diphthong
                        real_affix.push_str(&format!("[{}]", c));
                    },
                    None => {

                        if i != 0 {
                            // if the character before this one was a tie bar, this character and the two previous form a diphthong
                            if affix.chars().nth(i - 1).unwrap() as u32 == 865 { // check for a tie bar
                                debug!("---- tie bar found, adding ]");
                                real_affix.push_str(&format!("{}]", c));
                                continue;
                            }
                        }

                        // else, it's not a diphthong
                        real_affix.push_str(&format!("[{}]", c));
                    }
                }
                
            }

            word.insert(0, real_affix.replace("+", ""));
            // add a syllable marker
            word.insert(1, " ".to_owned());
        } 
        if suffixed {
            let mut copy = affixes.to_vec();
            let mut real_affix = "".to_owned();
            // choose a random suffix
            let mut affix = "".to_owned();
            while !affix.starts_with("-") {
                affix = copy.choose(&mut rng).unwrap().to_owned();
                debug!("SUFFIX: ----- affix: {}", affix);
                let index = copy.iter().position(|x| x == &affix).unwrap();
                debug!("index: {}, copy.length(): {}", index, copy.len());
                copy.remove(index);
            }
            affix = affix.replace("-", "");
            debug!("---- affix chars: \"{}\"", affix.split("").collect::<Vec<&str>>().join("   "));
            let mut temp: Vec<u32> = vec![];
            for c in affix.chars() {
                temp.push(c as u32);
            }
            debug!("---- affix chars: \"{}\"", temp.into_iter().map(|x| x.to_string()).collect::<Vec<String>>().join("   "));


            for (i, c) in affix.chars().enumerate() {
                // check if this is a diphthong:
                // proper error handling:
                let res = affix.chars().nth(i + 1);
                match res {
                    // within the bounds of the affix
                    Some(x) => {
                        // if the next character is a tie bar, the next three characters form a diphthong
                        if x as u32 == 865 { // check for a tie bar
                            debug!("---- tie bar found, adding [");
                            real_affix.push_str(&format!("[{}", c));
                            continue;
                        }

                        if c as u32 == 865 { // check for a tie bar
                            debug!("---- tie bar found, adding tie bar");
                            real_affix.push(c);
                            continue;
                        }

                        // else, it's not a diphthong
                        real_affix.push_str(&format!("[{}]", c));
                    },
                    None => {

                        if i != 0 {
                            // if the character before this one was a tie bar, this character and the two previous form a diphthong
                            if affix.chars().nth(i - 1).unwrap() as u32 == 865 { // check for a tie bar
                                debug!("---- tie bar found, adding ]");
                                real_affix.push_str(&format!("{}]", c));
                                continue;
                            }
                        }

                        // else, it's not a diphthong
                        real_affix.push_str(&format!("[{}]", c));
                    }
                }
                
            }

            // make sure there's a syllable marker
            if word[word.len() - 1] != " " {
                word.push(" ".to_owned());
            }

            // add the affix
            word.push(real_affix.replace("-", ""));
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
    debug!("-- word: {:?}", word);
    let ipa_word = word.join("");
    let mut clone = ipa_word.clone();
    // sort the hashmap by length of the key
    for (key, value) in config.romanization.iter() {
        // replace the key with the value
        clone = clone.replace(format!("[{}]", key).as_str(), value);
    }
    let romanized_word = clone.replace("'", "").replace(" ", ""); //.replace("[", "").replace("]", "");
    (ipa_word.replace("[", "").replace("]", ""), romanized_word.replace("[", "").replace("]", ""))
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
    // if the config file is empty, yell at end user and exit
    if config.consonants.len() == 0 || config.vowels.len() == 0 {
        println!("Error: Config file is empty or does not exist");
        return;
    }

    // for each word
    for _ in 0..word_count {
        let word = create_word(&config, &onsets, &codas, &affixes);
        // join the word
        let (ipa_word, romanized_word) = create_final_str(word, &config);
        // print romanized word
        println!("{} /{}/", romanized_word, ipa_word);
    }
}
