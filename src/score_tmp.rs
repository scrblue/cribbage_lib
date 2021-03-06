use crate::deck;
use std::cmp;

// Enum indicating the type of scoring events encountered during the play phase
// Scoring is based on the entire PlayGroup
// TODO Log Nibs and LastCard event as needed in the relevant portion of the main file
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub enum PlayScoreType {
    Nibs,         // Jack as starter card; two points for dealer
    Pair,         // Two cards with the same value; 2pts
    Triple,       // Three cards with the same value; 6pts
    Quadruple,    // Four cards with the same value; 12pts
    Straight(u8), // Three or more cards with sequential values; 1pt per card; variable is length
    Fifteen,      // When the exact value of the PlayGroup's total is 15; 2pts
    ThirtyOne,    // When the exact value of the PlayGroup's toal is 31; 2pts
    LastCard,     // When a player places the last card of a PlayGroup and does not have 31; 1pt
}

// Enum indicating the type of scoring events encoutered in the show phase
// Enum's options contain the cards used to make up each score event
// Allow manual scoring to count triples and quadruples as multiple pairs and to score double runs,
// triple runs, and double double runs with one selection
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub enum ShowScoreType {
    // Any combination of cards which add to 15; two pts
    Fifteen(Vec<deck::Card>),
    // Two cards with the same value; 2pts
    Pair(Vec<deck::Card>),
    // Three cards with the same value; 6pts
    Triple(Vec<deck::Card>),
    // Four cards with same value; 12pts
    Quadruple(Vec<deck::Card>),
    // Three or more cards with sequential values; 1pt per card
    Straight(Vec<deck::Card>),
    // Four cards not counting the starter card of the same suit; 4pts
    FourFlush(Vec<deck::Card>),
    // Five cards of the same suit; 5pts
    FiveFlush(Vec<deck::Card>),
    // Jack in hand which matches suit of starter card; one pt
    Nobs(Vec<deck::Card>),
}

// Enum for indicating whether a score event was made during the play phase or the show phase
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub enum ScoreType {
    Play(PlayScoreType),
    Show(ShowScoreType),
}

// Instance describing a single increase in score
// Used in logs of the game, manual scoring selection/confirmation, and for automatic scoring
// Vectors of ScoreEvents are returned by the scoring functions in this file to represent the
// correct score of each hand or PlayGroup
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct ScoreEvent {
    pub score_type: ScoreType,
    pub player_index: u8,
    pub point_value: u8,
}

// Scores a given hand of five cards
pub fn score_hand(index: u8, mut hand: Vec<deck::Card>, starter: deck::Card) -> Vec<ScoreEvent> {
    let mut found_scores: Vec<ScoreEvent> = Vec::new();

    // Variables tracking the runs of four and five such that runs are not
    let mut num_runs_four_or_higher = 0;
    let mut cards_of_runs_four_or_higher: Vec<Vec<deck::Card>> = Vec::new();

    // Variable tracking the card value of any triples or quadruples such as to allow the scoring
    // system to remove redundant couples and triples
    let mut value_triple_or_quadruple: Option<deck::CardValue> = None;
    let mut max_tuple_length = 0;

    // Create every combination of the five cards with a binary counter
    let mut combinations: Vec<Vec<deck::Card>> = Vec::new();
    let mut card_active = [false; 6];
    let mut wait_one = false;
    while !card_active[5] {
        let mut current_combination: Vec<deck::Card> = Vec::new();

        if card_active[4] {
            current_combination.push(starter);
        }
        if card_active[3] {
            current_combination.push(hand[3]);
        }
        if card_active[2] {
            current_combination.push(hand[2]);
        }
        if card_active[1] {
            current_combination.push(hand[1]);
        }
        if card_active[0] {
            current_combination.push(hand[0]);
        }

        if current_combination.len() >= 2 {
            combinations.push(current_combination);
        }

        if !card_active[5]
            && card_active[4]
            && card_active[3]
            && card_active[2]
            && card_active[1]
            && card_active[0]
        {
            card_active[5] = true;
            card_active[4] = false;
            card_active[3] = false;
            card_active[2] = false;
            card_active[1] = false;
            card_active[0] = false;
        }

        if !card_active[4] && card_active[3] && card_active[2] && card_active[1] && card_active[0] {
            card_active[4] = true;
            card_active[3] = false;
            card_active[2] = false;
            card_active[1] = false;
            card_active[0] = false;
        }

        if !card_active[3] && card_active[2] && card_active[1] && card_active[0] {
            card_active[3] = true;
            card_active[2] = false;
            card_active[1] = false;
            card_active[0] = false;
        }

        if !card_active[2] && card_active[1] && card_active[0] {
            card_active[2] = true;
            card_active[1] = false;
            card_active[0] = false;
        }

        if !card_active[1] && card_active[0] {
            card_active[1] = true;
            card_active[0] = false;
        }

        if !card_active[0] {
            if !wait_one {
                card_active[0] = true;
                wait_one = true;
            } else {
                wait_one = false;
            }
        }
    }

    for combination in combinations {
        // Sorts the hand so that equivalent score events will be equal
        let mut combination = combination.clone();
        combination.sort();
        // Flag for whether the current combination is a tuple; checks for pairs, triples, and
        // quadruples; flag is set to false when there is a value that does not match the value of
        // the first card of the combination
        let mut is_tuple = true;

        // Array of booleans representing whether a given card value is in the combination; if
        // there is a continuous group the length of the combination (such as not to count a
        // straight several times, then a straight is present
        let mut is_present = [false; 13];

        // Value representing the sum of the given combination such as to check whether or not
        // a combination adds to 15
        let mut sum = 0;

        for card in &combination {
            // For every card in the combination, add the value to the sum, determine if the
            // combination is still a tuple thus far, and if a given value is present in the
            // combination
            if card.value != combination[0].value {
                is_tuple = false;
            }
            sum += deck::return_play_value(*card);
            is_present[deck::return_value(*card) as usize - 1] = true;
        }

        // If all the cards in the combination had equal values, push the appropriate ScoreEvent to
        // found_scores and update the max_tuple_length and value_triple_or_quadruple as
        // appropriate
        if is_tuple {
            if combination.len() == 2 {
                if max_tuple_length < 2 {
                    max_tuple_length = 2;
                }
                found_scores.push(ScoreEvent {
                    player_index: index,
                    point_value: 2,
                    score_type: ScoreType::Show(ShowScoreType::Pair(combination.to_vec())),
                });
            } else {
                value_triple_or_quadruple = Some(combination[0].value);
                if combination.len() == 3 {
                    if max_tuple_length < 3 {
                        max_tuple_length = 3;
                    }
                    found_scores.push(ScoreEvent {
                        player_index: index,
                        point_value: 6,
                        score_type: ScoreType::Show(ShowScoreType::Triple(combination.to_vec())),
                    });
                } else if combination.len() == 4 {
                    if max_tuple_length < 4 {
                        max_tuple_length = 4;
                    }
                    found_scores.push(ScoreEvent {
                        player_index: index,
                        point_value: 12,
                        score_type: ScoreType::Show(ShowScoreType::Quadruple(combination.to_vec())),
                    });
                }
            }
        }
        // If the maximum number of consecutive values equals the number of cards in the
        // combination, then there is a straight
        let mut max_num_consecutive_values = 0;
        let mut num_consecutive_values = 0;
        for element in &is_present {
            if *element {
                num_consecutive_values += 1;
                if num_consecutive_values > max_num_consecutive_values {
                    max_num_consecutive_values = num_consecutive_values;
                }
            } else {
                num_consecutive_values = 0;
            }
        }
        if max_num_consecutive_values == combination.len() && combination.len() >= 3 {
            if combination.len() >= 4 {
                num_runs_four_or_higher += 1;
                cards_of_runs_four_or_higher.push(combination.to_vec());
            }
            found_scores.push(ScoreEvent {
                player_index: index,
                point_value: combination.len() as u8,
                score_type: ScoreType::Show(ShowScoreType::Straight(combination.to_vec())),
            });
        }

        // If the sum is 15
        if sum == 15 {
            found_scores.push(ScoreEvent {
                player_index: index,
                point_value: 2,
                score_type: ScoreType::Show(ShowScoreType::Fifteen(combination.to_vec())),
            });
        }
    }

    // Eliminates redundant pairs and triples from the found_scores
    if max_tuple_length == 4 {
        // All pairs and triples when there is a quadruple will be redundant
        found_scores.retain({
            |score| match &score.score_type {
                ScoreType::Show(ShowScoreType::Pair(cards)) => false,
                ScoreType::Show(ShowScoreType::Triple(cards)) => false,
                _ => true,
            }
        });
    } else if max_tuple_length == 3 {
        // Three out of the four possible pairs will be redundant when there is a triple but no
        // quadruple
        found_scores.retain({
            |score| match &score.score_type {
                ScoreType::Show(ShowScoreType::Pair(cards)) => {
                    if cards[0].value == value_triple_or_quadruple.unwrap() {
                        false
                    } else {
                        true
                    }
                }
                _ => true,
            }
        });
    }

    // Checks for multiple counting eg. a run of four and a run of three with the same cards;
    // there may be one run of five and one or two runs of four

    if num_runs_four_or_higher > 0 {
        // For each run in the runs of four or higher, henceforth the big runs; there can be at most two big runs
        for big_run in cards_of_runs_four_or_higher {
            // For each score event that is a run
            found_scores.retain({
                |score| match &score.score_type {
                    // If the ScoreType is a run
                    ScoreType::Show(ShowScoreType::Straight(small_run)) => {
                        let mut large_contains_small = true;
                        // For every card in the run of equal or smaller length, henceforth the
                        // small run
                        for card_of_small in small_run {
                            let mut is_current_card_in_larger_run = false;
                            // For every card in the big run, check if the card of the small run is
                            // contained and set the flag accordingly
                            for card_of_big in &big_run {
                                if *card_of_big == *card_of_small {
                                    is_current_card_in_larger_run = true;
                                }
                            }
                            // If any card in the smaller run is not contained in the bigger, then
                            // the large run does not contain the small run
                            if !is_current_card_in_larger_run || big_run == *small_run {
                                large_contains_small = false;
                            }
                        }

                        // Retain the score if the large run does not contain the small score
                        !large_contains_small
                    }
                    // If the ScoreType is not a run, retain the score
                    _ => true,
                }
            });
        }
    }

    // Checks for nobs
    for card in &hand {
        if card.value == deck::CardValue::Jack && card.suit == starter.suit {
            let mut nobs: Vec<deck::Card> = Vec::new();
            nobs.push(*card);
            nobs.push(starter);
            nobs.sort();
            found_scores.push(ScoreEvent {
                player_index: index,
                point_value: 1,
                score_type: ScoreType::Show(ShowScoreType::Nobs(nobs)),
            });
        }
    }

    // Checks for flushes
    // num_matching_cards will equal 4 if the hand contins a flush
    let suit: deck::CardSuit = hand[0].suit;
    let mut num_matching_cards: u8 = 0;
    for card in &hand {
        if card.suit == suit {
            num_matching_cards += 1;
        }
    }

    // If the hand contains a flush, check if the starter matches the suit too
    if num_matching_cards == 4 && starter.suit == suit {
        num_matching_cards += 1;
    }

    // If the hand contains a flush of 4 or 5, push the relevant ScoreEvents
    if num_matching_cards == 4 {
        hand.sort();
        found_scores.push(ScoreEvent {
            player_index: index,
            point_value: 4,
            score_type: ScoreType::Show(ShowScoreType::FourFlush(hand)),
        });
    } else if num_matching_cards == 5 {
        let mut all_cards: Vec<deck::Card> = Vec::new();

        for card in hand {
            all_cards.push(card);
        }
        all_cards.push(starter);
        all_cards.sort();

        found_scores.push(ScoreEvent {
            player_index: index,
            point_value: 5,
            score_type: ScoreType::Show(ShowScoreType::FiveFlush(all_cards)),
        });
    }

    found_scores.sort();

    found_scores
}
