use clap::Parser;
use std::{
    fs::{self},
    io::Result,
    path::PathBuf,
    process::exit,
    sync::Arc,
    thread,
};

#[derive(Parser)]
struct Options {
    #[arg(short, long)]
    character: char,
    #[arg(short, long, value_name = "FILE")]
    input_file_path: PathBuf,
    #[arg(long, default_value_t = false)]
    ignorecase: bool,
}

fn count_worker(
    content: &String,
    search_char: char,
    start_pos: usize,
    read_count: usize,
    ignorecae: bool,
) -> usize {
    content
        .chars()
        .skip(start_pos)
        .take(read_count)
        .filter(|c| {
            if !ignorecae {
                *c == search_char
            } else {
                c.to_ascii_lowercase() == search_char.to_ascii_lowercase()
            }
        })
        .count()
}

fn main() -> Result<()> {
    let options = Options::parse();

    if !options.input_file_path.exists() {
        eprintln!(
            "ERROR: Input file \"{}\" doesn't exists",
            options.input_file_path.to_str().unwrap()
        );
        exit(1);
    }
    if !options.input_file_path.is_file() {
        eprintln!(
            "ERROR: Input file \"{}\" is not a file",
            options.input_file_path.to_str().unwrap()
        );
        exit(1);
    }

    let content = Arc::new(fs::read_to_string(options.input_file_path)?);
    let content_len = content.len();
    let max_thread_count = thread::available_parallelism()?.get() - 1;

    if content_len == 0 || content_len <= max_thread_count {
        println!(
            "{}",
            content.chars().filter(|c| *c == options.character).count()
        );
    } else {
        let chunk_size = content_len / max_thread_count;
        let leftover_size = content_len % max_thread_count;

        let result: usize = thread::scope(|s| {
            let handles: Vec<_> = (0..max_thread_count)
                .into_iter()
                .map(|i| {
                    let content = Arc::clone(&content);
                    let chunk_start = i * chunk_size;
                    let mut read_count = chunk_size;

                    if i == max_thread_count - 1 {
                        read_count += leftover_size;
                    }

                    s.spawn(move || {
                        count_worker(
                            &content,
                            options.character,
                            chunk_start,
                            read_count,
                            options.ignorecase,
                        )
                    })
                })
                .collect();

            handles.into_iter().map(|h| h.join().unwrap()).sum()
        });

        println!("{result}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(text: &str) -> String {
        text.to_string()
    }

    #[test]
    fn counts_all_occurrences_in_full_range() {
        let content = s("banana");
        assert_eq!(count_worker(&content, 'a', 0, content.len(), false), 3);
        assert_eq!(count_worker(&content, 'n', 0, content.len(), false), 2);
        assert_eq!(count_worker(&content, 'b', 0, content.len(), false), 1);
    }

    #[test]
    fn returns_zero_when_char_is_absent() {
        let content = s("banana");
        assert_eq!(count_worker(&content, 'z', 0, content.len(), false), 0);
    }

    #[test]
    fn returns_zero_for_empty_content() {
        let content = s("");
        assert_eq!(count_worker(&content, 'a', 0, 10, false), 0);
        assert_eq!(count_worker(&content, 'a', 0, 10, true), 0);
    }

    #[test]
    fn returns_zero_for_zero_read_count() {
        let content = s("aaaa");
        assert_eq!(count_worker(&content, 'a', 0, 0, false), 0);
    }

    #[test]
    fn respects_start_pos_and_read_count() {
        // indices: a0 b1 a2 b3 a4 b5
        let content = s("ababab");
        assert_eq!(count_worker(&content, 'a', 1, 3, false), 1); // "bab"
        assert_eq!(count_worker(&content, 'a', 2, 3, false), 2); // "aba"
        assert_eq!(count_worker(&content, 'b', 5, 1, false), 1); // "b"
    }

    #[test]
    fn start_pos_past_end_returns_zero() {
        let content = s("aaa");
        assert_eq!(count_worker(&content, 'a', 3, 5, false), 0);
        assert_eq!(count_worker(&content, 'a', 100, 5, false), 0);
    }

    #[test]
    fn read_count_past_end_is_clamped() {
        let content = s("aaa");
        assert_eq!(count_worker(&content, 'a', 1, 100, false), 2);
    }

    #[test]
    fn case_sensitive_by_default() {
        let content = s("aAaA");
        assert_eq!(count_worker(&content, 'a', 0, content.len(), false), 2);
        assert_eq!(count_worker(&content, 'A', 0, content.len(), false), 2);
    }

    #[test]
    fn ignorecase_matches_both_cases() {
        let content = s("aAaA");
        assert_eq!(count_worker(&content, 'a', 0, content.len(), true), 4);
        assert_eq!(count_worker(&content, 'A', 0, content.len(), true), 4);
    }

    #[test]
    fn ignorecase_does_not_affect_non_letters() {
        let content = s("1!1!");
        assert_eq!(count_worker(&content, '1', 0, content.len(), true), 2);
        assert_eq!(count_worker(&content, '!', 0, content.len(), true), 2);
    }

    #[test]
    fn ignorecase_only_folds_ascii() {
        // to_ascii_lowercase leaves non-ASCII letters untouched.
        let content = s("éÉ");
        assert_eq!(count_worker(&content, 'é', 0, 2, true), 1);
        assert_eq!(count_worker(&content, 'É', 0, 2, true), 1);
    }

    #[test]
    fn positions_are_char_based_not_byte_based() {
        // 'ž' and 'đ' are 2 bytes each in UTF-8, so byte and char indices differ.
        let content = s("žađa");
        assert_eq!(count_worker(&content, 'a', 1, 1, false), 1); // "a"
        assert_eq!(count_worker(&content, 'đ', 2, 1, false), 1); // "đ"
        assert_eq!(count_worker(&content, 'a', 0, 4, false), 2);
    }

    #[test]
    fn counts_multibyte_search_char() {
        let content = s("čačča");
        assert_eq!(count_worker(&content, 'č', 0, 5, false), 3);
    }

    #[test]
    fn counts_whitespace_and_newlines() {
        let content = s("a b\nc d\n");
        assert_eq!(count_worker(&content, ' ', 0, content.len(), false), 2);
        assert_eq!(count_worker(&content, '\n', 0, content.len(), false), 2);
    }

    #[test]
    fn adjacent_chunks_sum_to_full_count() {
        let content = s("The quick brown fox jumps over the lazy dog. Oh, and again: the dog.");
        let len = content.chars().count();
        let expected = count_worker(&content, 'o', 0, len, false);

        for chunks in 1..=len {
            let chunk_size = len / chunks;
            let leftover = len % chunks;
            let total: usize = (0..chunks)
                .map(|i| {
                    let mut read_count = chunk_size;
                    if i == chunks - 1 {
                        read_count += leftover;
                    }
                    count_worker(&content, 'o', i * chunk_size, read_count, false)
                })
                .sum();
            assert_eq!(total, expected, "mismatch with {chunks} chunks");
        }
    }
}
