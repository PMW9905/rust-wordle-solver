use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, stdin};
use std::fs::File;
use std::process;
use std::io::Write;

const WORD_LEN: usize = 5;
const NUM_GUESS_MATCH_PERMUTATIONS: i32 = 243;
const BAR_CHAR_LENGTH: i32 = 20;

#[derive(Hash, Debug, Copy, Clone, PartialEq, Eq)]
enum LetterColor {
    Gray,
    Yellow,
    Green,
}

type GuessMatch = [LetterColor; WORD_LEN];

const WINNING_GUESS: GuessMatch = [LetterColor::Green; WORD_LEN];

fn print_progress_bar(numer: i32, denom: i32) {

    let fraction_loaded: f32 = (numer as f32) / (denom as f32);

    let bars_loaded: i32 = (fraction_loaded * (BAR_CHAR_LENGTH as f32)) as i32 ; 
    let bars_unloaded: i32 = BAR_CHAR_LENGTH - bars_loaded;

    print!("\r[");

    for _ in 0..bars_loaded {
        print!("=");
    }

    for _ in 0..bars_unloaded {
        print!(" ");
    }

    print!("]");
    let percent_loaded: i32 = (fraction_loaded * 100.0) as i32;

    print!(" {}% {} of {}", percent_loaded, numer, denom);
    if numer == denom {
        println!("")
    }
    std::io::stdout().flush().unwrap_or_default();
}

fn read_words_from_file(filepath: &str) -> Result<HashSet<String>, std::io::Error> {
    let file = File::open(filepath)?;
    let reader = BufReader::new(file);
    
    let mut words = HashSet::new();

    for line in reader.lines() {
        let word = line.unwrap_or_default();
        if !word.is_empty() {
            words.insert(word);
        }
    }   

    Ok(words)
}

fn calc_guessmatch(guess_word: &String, actual_word: &String) -> GuessMatch {
    let mut guess_letter_count: [i32; 26] = [0; 26];
    let mut actual_letter_count: [i32; 26] = [0; 26];
    let mut guess_match: GuessMatch = [LetterColor::Gray; WORD_LEN];
   
    for letter in actual_word.chars() {
        actual_letter_count[(letter as u8 - b'a') as usize]+=1;
    }

    for (index, guess_letter) in guess_word.chars().enumerate() {
        let actual_letter = actual_word.chars().nth(index).unwrap_or_default();

        let guess_letter_key = (guess_letter as u8 - b'a') as usize;

        guess_letter_count[guess_letter_key]+=1;

        if actual_letter == guess_letter {
            guess_match[index] = LetterColor::Green
        } else if actual_letter_count[guess_letter_key] >= guess_letter_count[guess_letter_key] {
            guess_match[index] = LetterColor::Yellow
        } 
    }
    
    return guess_match
}

fn filter_for_words_with_same_guessmatch(chosen_word: &String, resulting_guess: &GuessMatch, words: &HashSet<String>) -> HashSet<String> {
    let mut new_words: HashSet<String> = HashSet::new();

    for word in words.iter() {
        let guess_match = calc_guessmatch(chosen_word, &word);
        if guess_match == *resulting_guess {
            new_words.insert(word.clone());
        }
    }

    return new_words;
}

fn get_best_word(words: &HashSet<String>, all_words: &HashSet<String>) -> (String, f32) {

    println!("Word pool reduced to {}", words.len());

    let mut best_word: String = String::new();
    let mut best_std: f32 = 999.0;

    let mut words_per_pattern: HashMap<GuessMatch, i32> = HashMap::new();

    for (index, guess_word) in all_words.iter().enumerate() {
        for actual_word in words.iter() {
            // compare the guess word against the 'real' word
            // do this by computing the GuessMatch
            let guess_match: GuessMatch = calc_guessmatch(&guess_word, &actual_word);
            // add that guess to the respective HashMap
            match words_per_pattern.get(&guess_match) {
                Some(times_seen) => {
                   words_per_pattern.insert(guess_match, times_seen+1); 
                },
                None => {
                    words_per_pattern.insert(guess_match, 1);
                }
            }
        } 
        // after all of that, take the std of the hashmap.
        let mean: f32 = words.len() as f32 / NUM_GUESS_MATCH_PERMUTATIONS as f32; 
        let variance: f32 = words_per_pattern.values().map(|&x| (x as f32-mean).powi(2)).sum::<f32>() / NUM_GUESS_MATCH_PERMUTATIONS as f32;
        let mut std: f32 = variance.sqrt();

        if words_per_pattern.contains_key(&WINNING_GUESS) {
            // If there is a winning possibility, we want to give a slight advantage 
            // proportional to the number of words left
            std = std - (1.0/words.len() as f32);
        }
        // if it beats the current best std, then replace it. 
        if std < best_std {
            best_word = guess_word.clone();
            best_std = std.clone();
        }
        // clear the map
        words_per_pattern.clear();

        let numerator: i32 = i32::try_from(index.clone()).unwrap_or_default() + 1; 
        let denominator: i32 = all_words.len().try_into().unwrap_or_default();

        print_progress_bar(numerator, denominator); 
    }    


    (best_word, best_std)
}

fn main() {

    println!("Reading in words.txt");
    let mut words = match read_words_from_file("words.txt") {
        Ok(words) => words,
        Err(error) => {
            println!("An error occured when trying to read in words.txt: {}", error);
            process::exit(1);
        }
    };

    println!("Words: {}", words.len());
    //println!("Calculating best word...");
    //let best_std: f32;
    //let best_word: String;

    //(best_word, best_std) = get_best_word(&words, &words);
    //println!("\nResults: {}, {}", best_word, best_std);

    
    let mut is_still_guessing: bool = true;
    let mut next_word_suggestion = "rales".to_string(); 
    let all_words: HashSet<String> = words.clone();

    println!("WORDLE SOLVER");
    println!("I'll tell you what word to play. You'll reply back with the color match result.");
    println!("Green tiles are \'g\'. Yellow tiles are \'y\'. Miss tiles are \'-\'");
    println!("For example, a match result with the first two as green, the 3rd as yellow,");
    println!("and the others as blank, would look like:");
    println!("ggy--");
    println!("\n\nReady to play?\n");
    

    while is_still_guessing {

        println!("You should play the word \'{}\'.",next_word_suggestion);
        println!("What was your color match result?\n");

        let mut guess_result = String::new();

        match stdin().read_line(&mut guess_result) {
            Ok(input_size) => {
                input_size
            },
            Err(err) => {
                println!("Failed to grab input. Err: {}", err);
                process::exit(1);
            }
        }; 

        guess_result = guess_result.trim().to_string();         

        if guess_result.len() != WORD_LEN {
            println!("Input must be {} characters long. Length was {} Try again...", WORD_LEN, guess_result.len());
            continue;
        } 

        let mut guess_match: GuessMatch = [LetterColor::Gray; WORD_LEN];

        for (index, letter) in guess_result.chars().enumerate() {
            guess_match[index] = match letter {
                'g' => LetterColor::Green, 
                'y' => LetterColor::Yellow, 
                '-' => LetterColor::Gray, 
                _   => LetterColor::Gray, 
            };
        }

        if guess_match == [LetterColor::Green; WORD_LEN] {
            println!("Yippie! You got it!");
            is_still_guessing = false;
        } else {
            words = filter_for_words_with_same_guessmatch(&next_word_suggestion, &guess_match, &words);
            (next_word_suggestion, _) = get_best_word(&words, &all_words); 
        }
    }
}

