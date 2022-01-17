use itertools::Itertools;
use rand::prelude::SliceRandom;
use rand::rngs::StdRng;
use rand::{thread_rng, SeedableRng};
use std::fs::File;
use std::io;
use std::io::{stdin, BufRead, BufReader};
use std::ops::RangeInclusive;

const WORD_LENGTH: usize = 5;

enum Restrictions {
    AtPosition(char, usize),
    NotAtPosition(char, usize),
    Count(char, RangeInclusive<usize>),
}

fn main() {
    let mut rand = StdRng::seed_from_u64(42);
    let mut words = get_all_5_words();

    while words.len() > 1 {
        println!("Remaining words: {}", words.len());
        let word = words.choose(&mut rand).unwrap();
        println!("Guess: {}", word);
        println!("Is word in list? (enter yes or no)");
        let mut ans = String::new();
        stdin().read_line(&mut ans);
        if ans != "yes\n" {
            continue;
        }

        let restrictions = get_restrictions(word);
        words = update_words_from_restrictions(words, &restrictions);
        println!();
    }

    if let Some(ans) = words.get(0) {
        println!("The answer is {}", ans);
    } else {
        println!("The word is not in our list");
    }
}

fn update_words_from_restrictions(
    words: Vec<String>,
    restrictions: &Vec<Restrictions>,
) -> Vec<String> {
    words
        .into_iter()
        .filter(|word| {
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

fn get_restrictions(word: &str) -> Vec<Restrictions> {
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
    let green_chars: Vec<_> = green_pos
        .iter()
        .map(|i| word.chars().nth(*i).unwrap())
        .collect();

    println!("Enter the position of any yellow characters (enter 'n' when done):");
    let mut yellow_pos = Vec::new();
    request_from_user(&mut yellow_pos);
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

/// Reading from the file './words.txt', return all 5 letter words of the english alphabet in lower case
fn get_all_5_words() -> Vec<String> {
    let file = File::open("words.txt")
        .expect("Could not open file. Please make sure the file 'words.txt' exists in the current directory");
    let reader = BufReader::new(file);

    reader
        .lines()
        .map(|l| l.expect("Could not parse line").to_lowercase())
        .filter(|w| w.len() == WORD_LENGTH.try_into().unwrap())
        .collect()
}
