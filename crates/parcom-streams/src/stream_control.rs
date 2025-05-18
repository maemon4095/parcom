pub mod vec_control;
pub mod vecdeque_control;

pub enum Response<T, E> {
    Advance(T),
    Cancel(T, E),
    Finish(T),
}
