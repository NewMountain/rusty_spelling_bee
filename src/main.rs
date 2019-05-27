extern crate rand;


use rand::seq::SliceRandom;
use std::collections::HashSet;
use std::fs::File;

use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::process;
use std::{thread, time};


// Let's create a struct (Think Elm record type)
#[derive(Debug, Clone)]
struct GameStatus {
    game_word: String,
    game_chars: Vec<char>,
    anchor_char: char,
    guessable_words: Vec<String>,
    guessed_words: Vec<String>,
}

fn get_file_contents() -> std::io::Result<String> {
    // Read the local copy of the OSX dictionary
    let file = File::open("./src/assets/words")?;
    // Recommended best practice for space efficiency in Rust
    // https://doc.rust-lang.org/std/fs/struct.File.html
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();

    // Assign the buffer to the contents string
    buf_reader.read_to_string(&mut contents)?;

    // Function requires a result type return
    Ok(contents)
}

fn is_qualifying_word(word: &str) -> bool {
    // Make a copy of the string as we're going to mess with it
    let local_word = word.to_string().to_lowercase();

    // Take the chars
    // mutable as we are sorting them
    let mut chars: Vec<char> = local_word.chars().collect();

    // Mutate the list by sorting it then dedup it
    chars.sort();

    // Dedup the list
    // Lame that you can't dot chain in Rust
    chars.dedup();

    return chars.len() == 7;
}

fn get_distinct_chars(word: &str) -> Vec<char> {
    // Make a copy of the word
    let local_word = word.to_string().to_lowercase();

    // Take the chars
    // mutable as we are sorting them
    let mut chars: Vec<char> = local_word.chars().collect();

    // Mutate the list by sorting it then dedup it
    chars.sort();

    // Dedup the list
    // Lame that you can't dot chain in Rust
    chars.dedup();

    return chars;
}

fn is_guessable_word(word: &str, game_chars: &Vec<char>, anchor_char: &char) -> bool {
    // First create a copy of the lowercase word
    let local_word = word.to_string().to_lowercase();
    // Next get the distinct list of chars in the word
    let mut word_chars: Vec<char> = local_word.chars().collect();
    word_chars.sort();
    word_chars.dedup();

    // Now turn it into a set
    let charset: HashSet<_> = word_chars.iter().clone().collect();

    // Next, turn the game chars into HashSet
    let game_charset: HashSet<_> = game_chars.iter().clone().collect();

    // Check 1, is charset a subset of game_charset
    let check_one = charset.is_subset(&game_charset);

    // Check 2, does charset contain the anchor char?
    let check_two = charset.contains(anchor_char);

    return check_one && check_two;
}

fn get_input(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_goes_into_input_above) => {}
        Err(_no_updates_is_fine) => {}
    }
    // Trim and coerce to lowercase
    input.trim().to_string().to_lowercase()
}

fn print_game(game: &GameStatus) -> String {
    // Local anchor
    let local_anchor = game.anchor_char.to_owned();

    // Get the non-anchor chars
    let charset: HashSet<_> = game.game_chars.iter().cloned().collect();
    let anchor_charset: HashSet<_> = [local_anchor].iter().cloned().collect();

    // Get the non anchor chars
    let non_anchor_chars: HashSet<_> = charset.difference(&anchor_charset).collect();

    // Convert back to a vector
    let char_vec: Vec<&char> = non_anchor_chars.into_iter().collect();

    let board = format!(
        "Your letters are:\n\n {} {}\n{} {} {}\n {} {}",
        char_vec[0], char_vec[1], char_vec[2], local_anchor, char_vec[3], char_vec[4], char_vec[5]
    );

    let guesses = format!(
        "So far you have guessed the following {} words out of {} words possible:\n{:?}",
        game.guessed_words.len(),
        game.guessable_words.len(),
        game.guessed_words
    );

    return format!(
        "\n{}\n\n{}\n\n{}",
        board, guesses, "Please enter your guess, exit() to exit or give_up() to end the game."
    );
}


fn main() {

    // Get the words string from the file system
    let word_string = get_file_contents().unwrap_or("Could not find dictionary file.".to_string());

    // First thing, split by newline
    // Then filter for words less than three chars (we can make this a paramter later)
    // lines returns an iterator which can be collected
    let words: Vec<String> = word_string
        .lines()
        .filter(|w| w.chars().count() >= 3)
        .map(|w| w.to_owned())
        .collect();

    // Cool now create the character words that qualify for the game
    let candidate_words: Vec<&String> = words.iter().filter(|w| is_qualifying_word(w)).collect();

    // Pick one of them at random
    // Select an index at random
    let game_word = match candidate_words.choose(&mut rand::thread_rng()) {
        Some(word) => word.to_owned(),
        None => "",
    };

    // Get the characters of the game_word
    let game_chars: Vec<char> = get_distinct_chars(&game_word);

    // Pick one letter from the game word as the "anchor char"
    // IE the one letter that must occur in the word
    let anchor_char = match game_chars.choose(&mut rand::thread_rng()) {
        Some(character) => character.to_owned(),
        None => 'a',
    };

    // Now let's find all words that are guessable
    // Find all the words that only contain characters within the game chars
    // and contain the anchor char
    let guessable_words: Vec<String> = words
        .iter()
        .filter(|w| is_guessable_word(w, &game_chars, &anchor_char))
        .map(|w| w.to_owned())
        .map(|w| w.to_lowercase())
        .collect();

    // Create a game_struct
    let mut game = GameStatus {
        game_word: game_word.to_owned(),
        game_chars: game_chars,
        anchor_char: anchor_char,
        guessable_words: guessable_words,
        guessed_words: Vec::new(),
    };

    // Main game loop
    loop {
        // Clear the screen
        print!("{}[2J", 27 as char);
        // Make the game string
        let game_string = &print_game(&game);
        // Print the game and collect the user input
        let input: String = get_input(game_string);

        // If the user exits, print
        if input == "exit()" {
            println!("\n\nThank you for playing the game! Hope to see you soon!\n\n");
            process::exit(0);
        }

        // If the user gives up, end the game
        // TODO: Print All possible answers
        if input == "give_up()" {
            println!(
                "Not bad. You got {} out of {}.\n\n The complete list of words was: {:?}\n",
                game.guessed_words.len(),
                game.guessable_words.len(),
                game.guessable_words
            );

            println!("\n\nThank you for playing the game! Hope to see you soon!\n\n");

            process::exit(0);
        }

        // Otherwise, check the input and update the guessed words
        if game.guessable_words.contains(&input) {
            // Clear the screen
            print!("{}[2J", 27 as char);

            // Check that wasn't already guessed
            if game.guessed_words.contains(&input) {
                println!("{} was already guessed. Try again.", input)
            } else {
                println!("Good job! {} was one of the words!", input);
                game.guessed_words.push(input)
            }

            // Just persist the message a bit before restarting
            let one_and_change_secs = time::Duration::from_millis(1500);
            thread::sleep(one_and_change_secs);

        } else {
            // Clear the screen
            print!("{}[2J", 27 as char);
            println!("I'm sorry. {} was not one of the words!", input);

            // Just persist the message a bit before restarting
            let one_and_change_secs = time::Duration::from_millis(1500);
            thread::sleep(one_and_change_secs);
        }
    }
}