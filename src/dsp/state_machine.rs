use std::marker::PhantomData;

pub struct StateMachine<C, S: State<C> + Default> {
    current_state: S,
    phantom: PhantomData<C>,
}

pub trait State<C> {
    fn next_state(&mut self, context: C) -> Self;
}

impl<C, S: State<C> + Default> Default for StateMachine<C, S> {
    fn default() -> Self {
        Self {
            current_state: S::default(),
            phantom: PhantomData,
        }
    }
}

impl<C, S: State<C> + Default> StateMachine<C, S> {
    pub fn next(&mut self, context: C) -> &S {
        self.current_state = self.current_state.next_state(context);
        &self.current_state
    }
}
