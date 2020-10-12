
pub struct FiniteStateMachine
{
    accept_on_first_accepting_state: bool,
    state_count: usize,
    state_transitions: StateTransitionCollection,
    accepting_states: Vec<usize>
}

impl FiniteStateMachine
{
    pub fn new(state_count: usize, state_transitions: StateTransitionCollection, accept_on_first_accepting_state: bool, accepting_states: Vec<usize>) -> FiniteStateMachine
    {
        FiniteStateMachine { accept_on_first_accepting_state, state_count, state_transitions, accepting_states }
    }

    fn is_accepting_state(&self, state: usize) -> bool
    {
        self.accepting_states.iter().any(|s| *s == state)
    }

    pub fn test(&self, input: &[u8]) -> bool
    {
        let mut current_state = 0;
        let mut accepted;

        for current_char in input.iter()
        {
            accepted = false;
            for j in 0..self.state_count
            {
                let current_transitions = self.state_transitions.get_transition(current_state, j);

                for current_transition in current_transitions
                {
                    if (current_transition.accepted_chars)(*current_char)
                    {
                        current_state = current_transition.target_state;
                        if self.accept_on_first_accepting_state && self.is_accepting_state(current_state)
                        {
                            return true;
                        }

                        accepted = true;
                        break;
                    }
                }

                if accepted
                {
                    break;
                }
            }

            if !accepted
            {
                return false;
            }
        }

        self.is_accepting_state(current_state)
    }

    pub fn is_digit(c: u8) -> bool
    {
        c.is_ascii_digit()
    }

    pub fn is_whitespace(c: u8) -> bool
    {
        c == b' ' || c == b'\t' || c == b'\n' || c == b'\r'
    }

    pub fn whitespace_pattern() -> AcceptedCharacterPattern
    {
        Box::new(|c| FiniteStateMachine::is_whitespace(c))
    }

    pub fn literal_pattern(characters: Vec<u8>) -> AcceptedCharacterPattern
    {
        Box::new(move |c|
        {
            for ch in characters.iter()
            {
                if *ch == c
                {
                    return true;
                }
            }
            return false;
        })
    }

    pub fn single_character_pattern(ch: u8) -> AcceptedCharacterPattern
    {
        Box::new(move |c| ch == c)
    }

    pub fn digit_pattern() -> AcceptedCharacterPattern
    {
        Box::new(|c| FiniteStateMachine::is_digit(c))
    }

    pub fn any_character_pattern() -> AcceptedCharacterPattern
    {
        Box::new(|_| true)
    }
}

type AcceptedCharacterPattern = Box<dyn Fn(u8) -> bool + Sync>;

pub struct StateTransitionCollection
{
    transition_count: usize,
    state_transitions: Vec<Vec<StateTransition>>
}

impl StateTransitionCollection
{
    pub fn new(transition_count: usize) -> StateTransitionCollection
    {
        let mut stc = StateTransitionCollection
        {
            transition_count,
            state_transitions: Vec::with_capacity(transition_count * transition_count)
        };

        for _ in 0..transition_count * transition_count
        {
            stc.state_transitions.push(Vec::with_capacity(3));
        }

        stc
    }

    fn get_transition(&self, src: usize, target: usize) -> &Vec<StateTransition>
    {
        &self.state_transitions[src * self.transition_count + target]
    }

    pub fn set_transition(&mut self, src: usize, target: usize, accepted_pattern: AcceptedCharacterPattern)
    {
        &self.state_transitions[src * self.transition_count + target].push(StateTransition
        {
            accepted_chars: accepted_pattern,
            target_state: target
        });
    }
}

struct StateTransition
{
    accepted_chars: AcceptedCharacterPattern,
    target_state: usize
}