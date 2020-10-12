
use super::obj::finite_state_machine::FiniteStateMachine;

pub enum ReadIndexResult
{
    Ok(i32, usize),
    Error,
    Finished
}

// tries to read the first integer, and advance until the first whitespace character
pub fn read_int(input: &[u8], start_index: usize) -> ReadIndexResult
{
    let mut index = start_index;
    let end_index = input.len();

    while index < end_index && FiniteStateMachine::is_whitespace(input[index])
    {
        index += 1;
    }

    if index > end_index || index == end_index
    {
        return ReadIndexResult::Finished;
    }

    let number_start_index = index;

    while index < end_index && (FiniteStateMachine::is_digit(input[index]) || input[index] == b'-')
    {
        index += 1;
    }

    let number_end_index = index;

    while index < end_index && !FiniteStateMachine::is_whitespace(input[index])
    {
        index += 1;
    }

    let pattern_end_index = index;

    match std::str::from_utf8(&input[number_start_index..number_end_index])
    {
        Ok(st) => match st.parse::<i32>()
        {
            Ok(idx) => ReadIndexResult::Ok(idx, pattern_end_index - start_index),
            Err(_) => ReadIndexResult::Error
        },
        Err(_) => ReadIndexResult::Error
    }
}

// reads the string from the first non-whitespace character until the first whitespace character
// then tries to convert it to a float
pub fn read_float(input: &[u8], start_index: usize) -> Option<(f32, usize)>
{
    let mut index = start_index;
    let end_index = input.len();

    while index < end_index && FiniteStateMachine::is_whitespace(input[index])
    {
        index += 1;
    }

    let number_start_index = index;

    while index < end_index && !FiniteStateMachine::is_whitespace(input[index])
    {
        index += 1;
    }

    let number_end_index = index;

    match std::str::from_utf8(&input[number_start_index..number_end_index])
    {
        Ok(st) => st.parse::<f32>().ok().map(|val| (val, number_end_index - start_index)),
        Err(_) => None
    }
}