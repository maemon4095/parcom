pub mod vec_control;

pub enum Response<T, E> {
    Advance(T),
    Cancel(T, E),
    Finish(T),
}
