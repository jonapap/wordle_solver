use std::io;
use std::io::{stdin, BufRead};
use std::ops::RangeInclusive;

use itertools::Itertools;

const WORD_LENGTH: usize = 5;
static WORD_LIST: &'static str = include_str!("../words.txt");

/// Represents the current restrictions imposed on the word. This can be used to filter down the
/// word list until the desired word is found.
enum Restrictions {
    /// Character MUST be at a position
    AtPosition(char, usize),
    /// Character MUST NOT be at a position
    NotAtPosition(char, usize),
    /// The number of occurrences of a character must be within the range
    Count(char, RangeInclusive<usize>),
}

type WordInfo<'a> = (f64, &'a str);

fn main() {
    let words = get_all_5_words();
    let mut words = rank_words(words);

    while words.len() > 1 {
        if words.len() > 10 {
            println!("Remaining words: {}", words.len());
        } else {
            println!(
                "Remaining words: {} ({})",
                words.len(),
                words.iter().map(|w| w.1).join(", ")
            );
        }
        let word = words
            .iter()
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .unwrap();
        println!("Guess: {} ({})", word.1, word.0);

        println!("Is word in list? (enter yes or no)");
        let mut ans = String::new();
        stdin()
            .read_line(&mut ans)
            .expect("Could not read from stdin!");

        if ans == "yes\n" {
            let restrictions = get_restrictions_from_user(word.1);
            words = update_words_from_restrictions(words, &restrictions);
            println!();
        } else {
            let word_to_delete = word.clone();
            words.retain(|x| *x != word_to_delete);
        }
    }

    if let Some(ans) = words.get(0) {
        println!("The answer is {}", ans.1);
    } else {
        println!("The word is not in our list");
    }
}

fn rank_word(word: &str, words: &Vec<&str>) -> f64 {
    let length = words.len();
    words // Iterate through each possible guess
        .iter()
        .map(|w2| get_pattern_from_guess(w2, word)) // Get the Wordle pattern corresponding to the guess as a number
        .counts() // Count the occurrence of each pattern
        .iter()
        .map(|(_, count)| (*count as f64) / (length as f64)) // Get the probability of each pattern happening
        .map(|p| p * f64::log2(p.recip())) // This line and calculate the expected information (entropy) for w1
        .sum()
}

fn rank_words(words: Vec<&str>) -> Vec<WordInfo> {
    let length = words.len();
    words
        .iter()
        .map(|w1| (rank_word(w1, &words), *w1))
        .collect()
}

/// Returns a number from the answer and the guess. This number is unique for each Wordle pattern.
///
/// The number is more or less calculated by assuming the pattern is in base 3 and converting it
/// to base 10.
fn get_pattern_from_guess(answer: &str, guess: &str) -> usize {
    let (yellow_pos, green_pos) = get_pos_from_guess(answer, guess);

    let mut pattern: usize = 0;
    for i in yellow_pos {
        pattern += 2 * 3_usize.pow(i as u32);
    }
    for i in green_pos {
        pattern += 1 * 3_usize.pow(i as u32);
    }

    pattern
}

fn get_pos_from_guess(answer: &str, guess: &str) -> (Vec<usize>, Vec<usize>) {
    assert_eq!(answer.len(), guess.len());

    let mut added_chars = Vec::new();
    let mut yellow_pos = Vec::new();
    let mut green_pos = Vec::new();

    for (pos, char) in guess.chars().enumerate() {
        if answer.chars().nth(pos).unwrap() == char {
            green_pos.push(pos);
            added_chars.push(char);
        }
    }
    for (pos, char) in guess.chars().enumerate() {
        if answer.chars().contains(&char)
            && !green_pos.contains(&pos)
            && added_chars.iter().filter(|x| **x == char).count()
                < answer.chars().filter(|x| *x == char).count()
        {
            yellow_pos.push(pos);
            added_chars.push(char);
        }
    }

    (yellow_pos, green_pos)
}

fn get_restrictions_from_guess(answer: &str, guess: &str) -> Vec<Restrictions> {
    let (yellow_pos, green_pos) = get_pos_from_guess(answer, guess);

    convert_pos_to_restrictions(guess, green_pos, yellow_pos)
}

#[test]
fn run_on_all_words() {
    let word_list = get_all_5_words();
    let word_info = rank_words(word_list.clone());

    let tries: Vec<u32> = word_list
        .iter()
        .map(|answer| {
            let mut words = word_info.clone();

            let mut guesses = 0;
            while words.len() > 1 {
                let guess = words
                    .iter()
                    .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
                    .unwrap();

                let restrictions = get_restrictions_from_guess(answer, guess.1);
                words = update_words_from_restrictions(words, &restrictions);

                guesses += 1;
            }

            words.get(0).expect("Word does not exist in the list?");

            guesses
        })
        .collect();

    println!("Max: {}", tries.iter().max().unwrap());
    println!("Low: {}", tries.iter().min().unwrap());
    println!(
        "Average: {}",
        tries.iter().sum::<u32>() as f64 / tries.len() as f64
    );
    println!("Above 5: {}", tries.iter().filter(|x| **x > 5).count());
}

/// Filter the list of words based on the given restrictions
fn update_words_from_restrictions<'a, 'b>(
    words: Vec<WordInfo<'a>>,
    restrictions: &'b Vec<Restrictions>,
) -> Vec<WordInfo<'a>> {
    words
        .into_iter()
        .filter(|(_, word)| {
            for r in restrictions {
                match r {
                    Restrictions::NotAtPosition(char, pos) => {
                        if word.chars().nth(*pos).unwrap() == *char {
                            return false;
                        }
                    }
                    Restrictions::AtPosition(char, pos) => {
                        if word.chars().nth(*pos).unwrap() != *char {
                            return false;
                        }
                    }
                    Restrictions::Count(char, range) => {
                        let char_count = word.chars().filter(|x| *x == *char).count();
                        if !range.contains(&char_count) {
                            return false;
                        }
                    }
                }
            }

            return true;
        })
        .collect()
}

/// Ask the user to get the restrictions
fn get_restrictions_from_user(word: &str) -> Vec<Restrictions> {
    let request_from_user = |list: &mut Vec<usize>| {
        for i in io::stdin().lock().lines() {
            let input = i.unwrap();
            let position = input.parse::<usize>();
            if let Ok(p) = position {
                if !(1..=WORD_LENGTH).contains(&p) {
                    println!(
                        "Please make sure the number is between 1 and {} (inclusive)",
                        WORD_LENGTH
                    );
                } else {
                    list.push(p - 1);
                }
            } else if input == "n" {
                break;
            } else {
                println!("Please enter a number");
            }
        }
    };
    println!("Enter the position of any green characters (enter 'n' when done):");
    let mut green_pos = Vec::new();
    request_from_user(&mut green_pos);

    println!("Enter the position of any yellow characters (enter 'n' when done):");
    let mut yellow_pos = Vec::new();
    request_from_user(&mut yellow_pos);

    convert_pos_to_restrictions(word, green_pos, yellow_pos)
}

fn convert_pos_to_restrictions(
    word: &str,
    green_pos: Vec<usize>,
    yellow_pos: Vec<usize>,
) -> Vec<Restrictions> {
    let green_chars: Vec<_> = green_pos
        .iter()
        .map(|i| word.chars().nth(*i).unwrap())
        .collect();

    let yellow_chars: Vec<_> = yellow_pos
        .iter()
        .map(|i| word.chars().nth(*i).unwrap())
        .collect();

    // The remaining characters are then gray
    let gray_chars: Vec<_> = (0..word.len().try_into().unwrap())
        .filter(|i| !(yellow_pos.contains(i) || green_pos.contains(i)))
        .map(|i| word.chars().nth(i).unwrap())
        .collect();

    let mut restrictions = Vec::new();

    // Combine the info from all the positions to get a count restrictions
    for c in word.chars().unique() {
        let is_gray = gray_chars.contains(&c);
        let count_green = green_chars.iter().filter(|x| **x == c).count();
        let count_yellow = yellow_chars.iter().filter(|x| **x == c).count();

        let lower_limit = count_green + count_yellow;
        let upper_limit = if is_gray {
            0.max(lower_limit) // If we have some gray + other colors, we know the exact limit
        } else {
            WORD_LENGTH // Else, just put it at the max word length
        };

        restrictions.push(Restrictions::Count(c, lower_limit..=upper_limit));
    }

    // Simply map the green characters to a position restriction
    restrictions.extend(
        green_pos
            .into_iter()
            .map(|p| Restrictions::AtPosition(word.chars().nth(p).unwrap(), p)),
    );
    // Similar with the yellow characters
    restrictions.extend(
        yellow_pos
            .into_iter()
            .map(|p| Restrictions::NotAtPosition(word.chars().nth(p).unwrap(), p)),
    );

    restrictions
}

/// Reads from the file './words.txt', and returns all 5 letter words of the english alphabet in lower case.
fn get_all_5_words() -> Vec<&'static str> {
    // let file = File::open("words.txt")
    //     .expect("Could not open file. Please make sure the file 'words.txt' exists in the current directory");
    // let reader = BufReader::new(WORD_LIST);

    WORD_LIST
        .lines()
        // .map(|l| l.expect("Could not parse line").to_lowercase())
        .filter(|w| w.len() == WORD_LENGTH.try_into().unwrap())
        .collect()
}
