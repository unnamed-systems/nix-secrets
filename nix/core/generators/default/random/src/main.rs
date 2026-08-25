use eyre::{Context, OptionExt, bail};
use rand::{rng, seq::IndexedRandom};
use std::{
    io::{BufWriter, Write},
    iter::repeat_with,
};

fn main() -> eyre::Result<()> {
    let mut length = None;
    let mut count = None;
    let mut charset_str = None;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--length" => {
                length = Some(args.next().ok_or_eyre("Missing length")?.parse::<usize>()?);
            }
            "--count" => {
                count = Some(args.next().ok_or_eyre("Missing count")?.parse::<usize>()?);
            }
            "--charset" => {
                charset_str = Some(args.next().ok_or_eyre("Missing charset")?);
            }
            _ => bail!("Unknown argument provided"),
        }
    }

    let length = length.ok_or_eyre("Missing required flag: --length")?;
    let count = count.ok_or_eyre("Missing required flag: --count")?;
    let charset_raw = charset_str.ok_or_eyre("Missing required flag: --charset")?;

    if charset_raw.is_empty() {
        bail!("Empty charset provided");
    };

    let charset: Vec<char> = charset_raw.chars().collect();

    let mut rng = rng();
    let mut stdout = BufWriter::new(std::io::stdout());

    for _ in 0..count {
        let value: String = repeat_with(|| charset.choose(&mut rng))
            .take(length)
            .flatten()
            .collect();
        writeln!(stdout, "{value}").expect("Failed to write to stdout");
    }

    stdout.flush().wrap_err("Failed to flush stdout")?;
    Ok(())
}
