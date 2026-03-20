use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Beacon<'a> {
    pub name: &'a str,
    pub count: u32,
    pub delta: i32,
    pub power: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogError<'a> {
    Malformed(&'a str),
    InvalidName(&'a str),
    DuplicateName(&'a str),
    Overflow(&'a str),
}

pub fn parse_beacon_log<'a>(lines: &'a [&'a str]) -> Result<Vec<Beacon<'a>>, LogError<'a>> {
    let mut seen: HashSet<&'a str> = HashSet::new();

    lines
        .iter()
        .copied()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let mut parts = line.split('|');
            let name = parts.next().ok_or(LogError::Malformed(line))?;
            let count_str = parts.next().ok_or(LogError::Malformed(line))?;
            let delta_str = parts.next().ok_or(LogError::Malformed(line))?;

            if parts.next().is_some() {
                return Err(LogError::Malformed(line));
            }

            if name.is_empty() || !name.bytes().all(|b| b.is_ascii_lowercase() || b == b'-') {
                return Err(LogError::InvalidName(name));
            }

            if !seen.insert(name) {
                return Err(LogError::DuplicateName(name));
            }

            let count: u32 = count_str.parse().map_err(|_| LogError::Malformed(line))?;
            let delta: i32 = delta_str.parse().map_err(|_| LogError::Malformed(line))?;

            let magnitude = delta.unsigned_abs();
            let power = count
                .checked_mul(magnitude)
                .ok_or(LogError::Overflow(line))?;

            Ok(Beacon {
                name,
                count,
                delta,
                power,
            })
        })
        .collect()
}
