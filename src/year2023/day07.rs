use std::cmp::Ordering;
use std::collections::HashMap;
use std::io;

use crate::errors::invalid_input;
use crate::parse::lines;
use crate::part::Part;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
enum Card {
    Joker,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Card {
    fn from_char(ch: char, are_jokers_enabled: bool) -> io::Result<Self> {
        match ch {
            '2' => Ok(Self::Two),
            '3' => Ok(Self::Three),
            '4' => Ok(Self::Four),
            '5' => Ok(Self::Five),
            '6' => Ok(Self::Six),
            '7' => Ok(Self::Seven),
            '8' => Ok(Self::Eight),
            '9' => Ok(Self::Nine),
            'T' => Ok(Self::Ten),
            'J' => Ok(if are_jokers_enabled {
                Self::Joker
            } else {
                Self::Jack
            }),
            'Q' => Ok(Self::Queen),
            'K' => Ok(Self::King),
            'A' => Ok(Self::Ace),
            _ => Err(invalid_input("Unknown card")),
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
enum HandType {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

const HAND_SIZE: usize = 5;

fn calculate_hand_type(cards: &[Card; HAND_SIZE]) -> HandType {
    let mut card_map = HashMap::<Card, usize>::new();
    for card in cards {
        *card_map.entry(*card).or_insert(0) += 1;
    }

    let num_jokers = card_map.remove(&Card::Joker).unwrap_or(0);

    if card_map.len() <= 1 {
        return HandType::FiveOfAKind;
    }

    let mut counts = card_map
        .into_iter()
        .map(|(_key, val)| val)
        .collect::<Vec<_>>();
    counts.sort();

    let most_popular_count = counts[counts.len() - 1];
    assert!(most_popular_count >= 1);
    assert!(most_popular_count + num_jokers <= 4);

    if most_popular_count + num_jokers == 4 {
        return HandType::FourOfAKind;
    }

    if counts.len() == 2 {
        assert!(most_popular_count + num_jokers == 3);
        return HandType::FullHouse;
    }

    assert!(counts.len() >= 3);
    assert!(most_popular_count + num_jokers <= 3);

    if most_popular_count + num_jokers == 3 {
        return HandType::ThreeOfAKind;
    }

    assert!(most_popular_count + num_jokers <= 2);
    assert!(num_jokers <= 1);

    let second_most_popular_count = counts[counts.len() - 2];
    if most_popular_count == 2 && second_most_popular_count == 2 {
        return HandType::TwoPair;
    }

    if most_popular_count + num_jokers >= 2 {
        return HandType::OnePair;
    }

    assert!(num_jokers == 0);
    return HandType::HighCard;
}

#[derive(PartialEq, Eq)]
struct Hand {
    cards: [Card; HAND_SIZE],
    bid: i64,
    hand_type: HandType,
}

impl Hand {
    fn from_line(line: &str, are_jokers_enabled: bool) -> io::Result<Self> {
        let [cards_str, bid_str] =
            line.split_whitespace().collect::<Vec<_>>()[..]
        else {
            return Err(invalid_input("Expected cards and bid"));
        };

        let cards = cards_str
            .chars()
            .map(|ch| Card::from_char(ch, are_jokers_enabled))
            .collect::<io::Result<Vec<_>>>()?
            .try_into()
            .map_err(|_ignored| invalid_input("Expected 5 cards"))?;

        let bid = bid_str.parse::<i64>().map_err(invalid_input)?;

        let hand_type = calculate_hand_type(&cards);

        Ok(Self {
            cards,
            bid,
            hand_type,
        })
    }
}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.hand_type, self.cards, self.bid).cmp(&(
            other.hand_type,
            other.cards,
            other.bid,
        ))
    }
}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut hands = lines(reader)?
        .map(|line| Hand::from_line(&line, part == Part::Part2))
        .collect::<io::Result<Vec<_>>>()?;
    hands.sort();

    let result: i64 = hands
        .into_iter()
        .enumerate()
        .map(|(index, hand)| ((index + 1) as i64) * hand.bid)
        .sum();
    println!("{result}");

    Ok(())
}
